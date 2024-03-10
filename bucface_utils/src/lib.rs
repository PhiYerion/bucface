pub mod ws;
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub author: Box<str>,
    pub machine: Box<str>,
    pub event: Box<str>,
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
            author: random_string(rng.gen_range(3..100)).into(),
            machine: random_string(rng.gen_range(3..100)).into(),
            event: random_string(rng.gen_range(3..10000)).into(),
            time: chrono::Utc::now().naive_utc(),
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
