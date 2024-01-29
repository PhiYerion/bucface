#[cfg(feature = "actix")]
use actix_web::body::BoxBody;
#[cfg(feature = "actix")]
use actix_web::{HttpRequest, HttpResponse, Responder};
use rand::distributions::{Alphanumeric, Distribution, Standard};
use rand::Rng;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub author: Box<str>,
    pub machine: Box<str>,
    pub event: Box<str>,
    pub time: chrono::NaiveDateTime,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Events {
    pub inner: Vec<Event>,
}

impl Distribution<Events> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Events {
        Events {
            inner: (0..rng.gen_range(1..100)).map(|_| rng.gen()).collect(),
        }
    }
}

#[cfg(feature = "actix")]
impl Responder for Events {
    fn respond_to(self, _: &HttpRequest) -> HttpResponse {
        let mut buf = Vec::new();
        if let Err(e) = self.serialize(&mut Serializer::new(&mut buf)) {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
        HttpResponse::Ok().body(buf)
    }
    type Body = BoxBody;
}
