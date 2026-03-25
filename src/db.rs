use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::metrics::{DataPoint, Granularity, Sample};

const TABLES: &[(&str, i64)] = &[
    ("samples_1s", 60),
    ("samples_5s", 1800),
    ("samples_30s", 14400),
    ("samples_5m", 86400),
    ("samples_15m", 604800),
];

const COLUMNS: &str = "ts, cpu_percent, cpu_temp, ram_percent, swap_percent, \
    gpu_percent, gpu_temp, vram_percent, \
    disk_read_bytes, disk_write_bytes, net_rx_bytes, net_tx_bytes";

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open (or create) the SQLite database at %LOCALAPPDATA%/LocalMonitor/localmonitor.db.
    pub fn open() -> Result<Self> {
        let data_dir =
            dirs::data_local_dir().context("could not determine LOCALAPPDATA directory")?;
        let app_dir = data_dir.join("LocalMonitor");
        std::fs::create_dir_all(&app_dir)
            .with_context(|| format!("failed to create directory {}", app_dir.display()))?;
        let db_path = app_dir.join("localmonitor.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("failed to open database at {}", db_path.display()))?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// Open an in-memory database (for tests and display-only fallback).
    pub fn open_in_memory() -> Result<Self> {
        let conn =
            Connection::open_in_memory().context("failed to open in-memory SQLite database")?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// Create the five resolution tables and their ts indexes.
    fn init_tables(&self) -> Result<()> {
        for &(table, _) in TABLES {
            self.conn
                .execute_batch(&format!(
                    "CREATE TABLE IF NOT EXISTS {table} (
                        ts              INTEGER NOT NULL,
                        cpu_percent     REAL,
                        cpu_temp        REAL,
                        ram_percent     REAL,
                        swap_percent    REAL,
                        gpu_percent     REAL,
                        gpu_temp        REAL,
                        vram_percent    REAL,
                        disk_read_bytes REAL,
                        disk_write_bytes REAL,
                        net_rx_bytes    REAL,
                        net_tx_bytes    REAL
                    );
                    CREATE INDEX IF NOT EXISTS idx_{table}_ts ON {table}(ts);"
                ))
                .with_context(|| format!("failed to create table {table}"))?;
        }
        Ok(())
    }

    /// Insert a `Sample` into the 1-second resolution table.
    pub fn insert_sample(&self, s: &Sample) -> Result<()> {
        self.conn
            .execute(
                &format!("INSERT INTO samples_1s ({COLUMNS}) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)"),
                rusqlite::params![
                    s.ts,
                    s.cpu_percent,
                    s.cpu_temp,
                    s.ram_percent,
                    s.swap_percent,
                    s.gpu_percent,
                    s.gpu_temp,
                    s.vram_percent,
                    s.disk_read_bytes as f64,
                    s.disk_write_bytes as f64,
                    s.net_rx_bytes as f64,
                    s.net_tx_bytes as f64,
                ],
            )
            .context("failed to insert sample into samples_1s")?;
        Ok(())
    }

    /// Query data points from the table that matches the given `Granularity`,
    /// returning rows within its time window.
    pub fn query(&self, granularity: Granularity) -> Result<Vec<DataPoint>> {
        let table = granularity.table_name();
        let window = granularity.window_secs();
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - window;

        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM {table} WHERE ts >= ?1 ORDER BY ts ASC"
            ))
            .with_context(|| format!("failed to prepare query on {table}"))?;

        let rows = stmt
            .query_map(rusqlite::params![cutoff], |row| {
                Ok(DataPoint {
                    ts: row.get(0)?,
                    cpu_percent: row.get::<_, Option<f64>>(1)?.unwrap_or_default(),
                    cpu_temp: row.get(2)?,
                    ram_percent: row.get::<_, Option<f64>>(3)?.unwrap_or_default(),
                    swap_percent: row.get::<_, Option<f64>>(4)?.unwrap_or_default(),
                    gpu_percent: row.get(5)?,
                    gpu_temp: row.get(6)?,
                    vram_percent: row.get(7)?,
                    disk_read_rate: row.get::<_, Option<f64>>(8)?.unwrap_or_default(),
                    disk_write_rate: row.get::<_, Option<f64>>(9)?.unwrap_or_default(),
                    net_rx_rate: row.get::<_, Option<f64>>(10)?.unwrap_or_default(),
                    net_tx_rate: row.get::<_, Option<f64>>(11)?.unwrap_or_default(),
                })
            })
            .with_context(|| format!("failed to query {table}"))?;

        let mut points = Vec::new();
        for row in rows {
            points.push(row.context("failed to read row")?);
        }
        Ok(points)
    }

    /// Aggregate data from finer-grained tables into coarser ones, then prune
    /// old rows beyond each table's retention window.
    pub fn aggregate_and_prune(&self) -> Result<()> {
        let aggregations: &[(&str, &str, i64)] = &[
            ("samples_1s", "samples_5s", 5),
            ("samples_5s", "samples_30s", 30),
            ("samples_30s", "samples_5m", 300),
            ("samples_5m", "samples_15m", 900),
        ];

        for &(src, dst, bucket_secs) in aggregations {
            self.conn
                .execute(
                    &format!(
                        "INSERT INTO {dst} ({COLUMNS})
                         SELECT (ts / ?1) * ?1,
                                AVG(cpu_percent), AVG(cpu_temp),
                                AVG(ram_percent), AVG(swap_percent),
                                AVG(gpu_percent), AVG(gpu_temp), AVG(vram_percent),
                                AVG(disk_read_bytes), AVG(disk_write_bytes),
                                AVG(net_rx_bytes), AVG(net_tx_bytes)
                         FROM {src}
                         WHERE ts <= (SELECT COALESCE(MAX(ts), 0) FROM {src})
                           AND (ts / ?1) * ?1 NOT IN (SELECT ts FROM {dst})
                         GROUP BY ts / ?1"
                    ),
                    rusqlite::params![bucket_secs],
                )
                .with_context(|| format!("failed to aggregate {src} -> {dst}"))?;
        }

        // Prune old data beyond each table's retention.
        let now = chrono::Utc::now().timestamp();
        for &(table, retention_secs) in TABLES {
            let cutoff = now - retention_secs;
            self.conn
                .execute(
                    &format!("DELETE FROM {table} WHERE ts < ?1"),
                    rusqlite::params![cutoff],
                )
                .with_context(|| format!("failed to prune {table}"))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample(ts: i64) -> Sample {
        Sample {
            ts,
            cpu_percent: 42.5,
            cpu_temp: Some(65.0),
            ram_percent: 55.0,
            swap_percent: 10.0,
            gpu_percent: Some(30.0),
            gpu_temp: Some(70.0),
            vram_percent: Some(25.0),
            disk_read_bytes: 1024,
            disk_write_bytes: 2048,
            net_rx_bytes: 500,
            net_tx_bytes: 300,
            ..Sample::default()
        }
    }

    #[test]
    fn test_insert_and_query() {
        let db = Db::open_in_memory().unwrap();
        let now = chrono::Utc::now().timestamp();
        let sample = make_sample(now);
        db.insert_sample(&sample).unwrap();

        let points = db.query(Granularity::M1).unwrap();
        assert_eq!(points.len(), 1);

        let p = &points[0];
        assert!((p.cpu_percent - 42.5).abs() < 0.01);
        assert!((p.cpu_temp.unwrap() - 65.0).abs() < 0.01);
        assert!((p.ram_percent - 55.0).abs() < 0.01);
        assert!((p.swap_percent - 10.0).abs() < 0.01);
        assert!((p.gpu_percent.unwrap() - 30.0).abs() < 0.01);
        assert!((p.gpu_temp.unwrap() - 70.0).abs() < 0.01);
        assert!((p.vram_percent.unwrap() - 25.0).abs() < 0.01);
        assert!((p.disk_read_rate - 1024.0).abs() < 0.01);
        assert!((p.disk_write_rate - 2048.0).abs() < 0.01);
        assert!((p.net_rx_rate - 500.0).abs() < 0.01);
        assert!((p.net_tx_rate - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregate_and_prune() {
        let db = Db::open_in_memory().unwrap();
        let now = chrono::Utc::now().timestamp();

        // Insert 10 samples at 1-second intervals, all within the same 5s bucket.
        let bucket_start = (now / 5) * 5;
        for i in 0..5 {
            let mut s = make_sample(bucket_start + i);
            s.cpu_percent = 40.0 + (i as f64) * 2.0; // 40, 42, 44, 46, 48 → avg 44
            db.insert_sample(&s).unwrap();
        }
        // Second bucket.
        for i in 0..5 {
            let mut s = make_sample(bucket_start + 5 + i);
            s.cpu_percent = 50.0 + (i as f64) * 2.0; // 50, 52, 54, 56, 58 → avg 54
            db.insert_sample(&s).unwrap();
        }

        db.aggregate_and_prune().unwrap();

        // Verify samples_5s has aggregated rows.
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM samples_5s").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert!(
            count >= 2,
            "expected at least 2 aggregated rows, got {count}"
        );

        // Verify averaged CPU values.
        let mut stmt = db
            .conn
            .prepare("SELECT cpu_percent FROM samples_5s ORDER BY ts ASC")
            .unwrap();
        let avgs: Vec<f64> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(
            (avgs[0] - 44.0).abs() < 0.01,
            "first bucket avg: {}",
            avgs[0]
        );
        assert!(
            (avgs[1] - 54.0).abs() < 0.01,
            "second bucket avg: {}",
            avgs[1]
        );
    }
}
