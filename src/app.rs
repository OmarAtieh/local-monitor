use crate::metrics::{Granularity, Sample};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFocus {
    CpuRam,
    GpuVram,
    DiskIo,
    Network,
}

impl GraphFocus {
    pub fn label(&self) -> &str {
        match self {
            Self::CpuRam => "CPU / RAM",
            Self::GpuVram => "GPU / VRAM",
            Self::DiskIo => "Disk I/O",
            Self::Network => "Network",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::CpuRam => Self::GpuVram,
            Self::GpuVram => Self::DiskIo,
            Self::DiskIo => Self::Network,
            Self::Network => Self::CpuRam,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::CpuRam => Self::Network,
            Self::GpuVram => Self::CpuRam,
            Self::DiskIo => Self::GpuVram,
            Self::Network => Self::DiskIo,
        }
    }
}

pub struct App {
    pub focus: GraphFocus,
    pub granularity: Granularity,
    pub latest_sample: Sample,
    pub gpu_available: bool,
    pub running: bool,
    pub db_warning: Option<String>,
}

impl App {
    pub fn new(gpu_available: bool) -> Self {
        Self {
            focus: GraphFocus::CpuRam,
            granularity: Granularity::M1,
            latest_sample: Sample::default(),
            gpu_available,
            running: true,
            db_warning: None,
        }
    }

    pub fn next_view(&mut self) {
        self.focus = self.focus.next();
        if !self.gpu_available && self.focus == GraphFocus::GpuVram {
            self.focus = self.focus.next();
        }
    }

    pub fn prev_view(&mut self) {
        self.focus = self.focus.prev();
        if !self.gpu_available && self.focus == GraphFocus::GpuVram {
            self.focus = self.focus.prev();
        }
    }

    pub fn longer_granularity(&mut self) {
        self.granularity = self.granularity.next();
    }

    pub fn shorter_granularity(&mut self) {
        self.granularity = self.granularity.prev();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_focus_next_wraps() {
        assert_eq!(GraphFocus::Network.next(), GraphFocus::CpuRam);
    }

    #[test]
    fn test_graph_focus_prev_wraps() {
        assert_eq!(GraphFocus::CpuRam.prev(), GraphFocus::Network);
    }

    #[test]
    fn test_app_skips_gpu_when_unavailable() {
        let mut app = App::new(false);
        assert_eq!(app.focus, GraphFocus::CpuRam);
        app.next_view();
        assert_eq!(app.focus, GraphFocus::DiskIo);
        app.prev_view();
        assert_eq!(app.focus, GraphFocus::CpuRam);
    }

    #[test]
    fn test_app_includes_gpu_when_available() {
        let mut app = App::new(true);
        app.next_view();
        assert_eq!(app.focus, GraphFocus::GpuVram);
    }

    #[test]
    fn test_granularity_cycling() {
        let mut app = App::new(false);
        assert_eq!(app.granularity, Granularity::M1);
        app.longer_granularity();
        assert_eq!(app.granularity, Granularity::M5);
        app.shorter_granularity();
        assert_eq!(app.granularity, Granularity::M1);
    }
}
