use bucface_utils::Event;
use rmp_serde::Serializer;
use serde::Serialize;

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
    pub buf: Vec<u8>,
    pub mode: AppMode,
    client: reqwest::Client,
}

impl App<'_> {
    pub(crate) async fn send_buf(&mut self) -> std::io::Result<()> {
        let mode = self.mode;
        self.mode = AppMode::Normal;

        match mode {
            AppMode::Entry => self.buf_to_name(),
            AppMode::Logging => self.send_buf_as_log().await,
            AppMode::Quitting | AppMode::Normal => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Attempting to send buffer while quitting",
            )),
        }
    }

    async fn send_buf_as_log(&mut self) -> std::io::Result<()> {
        let log = Event {
            author: self.name.clone().into(),
            machine: "test".into(),
            event: self.buf.iter().map(|x| char::from(*x)).collect::<String>().into(),
            time: chrono::Utc::now().naive_utc(),
        };
        self.buf.clear();

        log.serialize(&mut Serializer::new(&mut self.buf));

        while self.client.post("127.0.0.1:8080/events").body(self.buf.clone()).send().await.is_err() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }



        self.buf.clear();
        Ok(())
    }

    fn buf_to_name(&mut self) -> std::io::Result<()> {
        self.name = self.buf.iter().map(|x| char::from(*x)).collect::<String>();
        self.buf.clear();
        Ok(())
    }
}
