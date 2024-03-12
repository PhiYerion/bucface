pub mod ws;
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Event {
    pub author: String,
    pub machine: String,
    pub event: String,
    pub time: chrono::NaiveDateTime,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            author: "Default Author".into(),
            machine: "Default Machine".into(),
            event: "Default Event".into(),
            time: chrono::Utc::now().naive_utc(),
        }
    }
}

impl Distribution<Event> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Event {
        Event {
            author: random_string(rng.gen_range(1..3)),
            machine: random_string(rng.gen_range(1..3)),
            event: random_string(rng.gen_range(1..3)),
            time: chrono::Utc::now().naive_utc(),
        }
    }
}

impl From<EventDB> for Event {
    fn from(event: EventDB) -> Self {
        Self {
            author: event.author,
            machine: event.machine,
            event: event.event,
            time: event.time,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub enum ClientMessage {
    /// A message to add to the database.
    NewEvent(Event),
    /// A message that requests the event with the given id.
    GetEvent(u64),
    /// A message that requests all events since the given id.
    GetSince(u64),
    Ping(String),
}

fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDB {
    pub _id: u64,
    pub author: String,
    pub machine: String,
    pub event: String,
    pub time: chrono::NaiveDateTime,
}

impl EventDB {
    pub fn from(event: Event, id: u64) -> Self {
        Self {
            _id: id,
            author: event.author,
            machine: event.machine,
            event: event.event,
            time: event.time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerResponse {
    Event(EventDB),
    Error(EventDBErrorSerde),
    Pong(Vec<u8>),
    Close(Vec<u8>),
}

#[derive(Debug)]
pub enum EventDBError {
    Db(surrealdb::Error),
    NotFound,
    RmpEncode(rmp_serde::encode::Error),
    RmpDecode(rmp_serde::decode::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventDBErrorSerde {
    Db(String),
    NotFound,
    Rmp,
}

impl From<EventDBError> for EventDBErrorSerde {
    fn from(e: EventDBError) -> Self {
        match e {
            EventDBError::Db(e) => Self::Db(e.to_string()),
            EventDBError::NotFound => Self::NotFound,
            EventDBError::RmpEncode(_) | EventDBError::RmpDecode(_) => Self::Rmp,
        }
    }
}
