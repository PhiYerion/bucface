#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Logging,
    Quitting,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Normal => write!(f, "Normal"),
            AppMode::Logging => write!(f, "Logging"),
            AppMode::Quitting => write!(f, "Quitting"),
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Default, Clone)]
pub struct App<'a> {
    pub human_events: Vec<Event<'a>>,
    pub other_events: Vec<Event<'a>>,
    pub name: &'a str,
    pub buf: Vec<char>,
    pub mode: AppMode,
}

impl App<'_> {
    pub(crate) fn send_buf(&mut self) -> std::io::Result<()> {
        self.buf.clear();
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Event<'a> {
    pub author: &'a str,
    pub machine: &'a str,
    pub time: std::time::Instant,
    pub event: &'a str,
}
