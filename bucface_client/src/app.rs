use bucface_utils::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Entry,
    Normal,
    Logging,
    Quitting,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Entry => write!(f, "Entry"),
            AppMode::Normal => write!(f, "Normal"),
            AppMode::Logging => write!(f, "Logging"),
            AppMode::Quitting => write!(f, "Quitting"),
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Entry
    }
}

#[derive(Debug, Default, Clone)]
pub struct App<'a> {
    pub human_events: Vec<Event>,
    pub other_events: Vec<Event>,
    pub machines: Vec<&'a str>,
    pub name: String,
    pub buf: Vec<char>,
    pub mode: AppMode,
}

impl App<'_> {
    pub(crate) fn send_buf(&mut self) -> std::io::Result<()> {
        let mode = self.mode;
        self.mode = AppMode::Normal;

        match mode {
            AppMode::Entry => self.buf_to_name(),
            AppMode::Logging => self.send_buf_as_log(),
            AppMode::Quitting | AppMode::Normal => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Attempting to send buffer while quitting",
            )),
        }
    }

    fn send_buf_as_log(&mut self) -> std::io::Result<()> {
        self.buf.clear();
        todo!()
    }

    fn buf_to_name(&mut self) -> std::io::Result<()> {
        self.name = self.buf.iter().collect::<String>();
        self.buf.clear();
        Ok(())
    }
}
