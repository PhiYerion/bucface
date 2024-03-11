pub mod ws;
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
            author: random_string(rng.gen_range(3..100)),
            machine: random_string(rng.gen_range(3..100)),
            event: random_string(rng.gen_range(3..10000)),
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

fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDB {
    pub _id: i64,
    pub author: String,
    pub machine: String,
    pub event: String,
    pub time: chrono::NaiveDateTime,
}

impl EventDB {
    pub fn from(event: Event, id: i64) -> Self {
        Self {
            _id: id,
            author: event.author,
            machine: event.machine,
            event: event.event,
            time: event.time,
        }
    }
}

#[derive(Debug)]
pub enum EventDBError {
    Db(surrealdb::Error),
    NotFound,
    Rmp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventDBResponse {
    pub id: i64,
    pub inner: Result<Event, EventDBErrorSerde>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventDBErrorSerde {
    Db(String),
    NotFound,
    Rmp,
}

impl EventDBResponse {
    pub fn from_err(id: i64, e: EventDBError) -> Self {
        Self {
            id,
            inner: Err(match e {
                EventDBError::Db(e) => EventDBErrorSerde::Db(e.to_string()),
                EventDBError::NotFound => EventDBErrorSerde::NotFound,
                EventDBError::Rmp => EventDBErrorSerde::Rmp,
            }),
        }
    }
}
