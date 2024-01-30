mod events;
use std::sync::{Arc, Mutex};

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use bucface_utils::Events;
use events::get_events;

use self::events::create_event;

#[derive(Debug, Clone, Default)]
struct AppState {
    events: Arc<Mutex<Events>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let data = Data::new(AppState::default());
    HttpServer::new(move || {
        App::new()
            .service(get_events)
            .service(create_event)
            .app_data(data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use actix_web::{http, test, web, App};
    use bucface_utils::Events;
    use rand::Rng;
    use rmp_serde::Serializer;
    use serde::Serialize;

    use crate::events::get_events;
    use crate::{create_event, AppState};

    #[actix_web::test]
    async fn integration() {
        let _ = env_logger::try_init();
        let data = AppState::default();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(data.clone()))
                .service(get_events)
                .service(create_event),
        )
        .await;

        let test_events: Events = rand::thread_rng().gen();

        let mut body = Vec::new();
        test_events
            .serialize(&mut Serializer::new(&mut body))
            .unwrap();
        let req = test::TestRequest::post()
            .uri("/events")
            .set_payload(body.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let req = test::TestRequest::get().uri("/events").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let decoded = rmp_serde::decode::from_slice::<Events>(&body).unwrap();
        dbg!(decoded.clone());
        assert_eq!(decoded, test_events);
    }

    #[actix_web::test]
    async fn send_events() {
        let _ = env_logger::try_init();
        let data = AppState::default();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(data.clone()))
                .service(get_events)
                .service(create_event),
        )
        .await;

        let test_events: Events = rand::thread_rng().gen();

        let mut body = Vec::new();
        test_events
            .serialize(&mut Serializer::new(&mut body))
            .unwrap();
        let req = test::TestRequest::post()
            .uri("/events")
            .set_payload(body.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        assert_eq!(*data.events.lock().unwrap(), test_events);
    }

    #[actix_web::test]
    async fn get_events_test() {
        let _ = env_logger::try_init();
        let test_events: Events = rand::thread_rng().gen();
        let data = AppState {
            events: Arc::new(Mutex::new(test_events)),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(data.clone()))
                .service(get_events)
                .service(create_event),
        )
        .await;

        let req = test::TestRequest::get().uri("/events").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
        let body = test::read_body(resp).await;
        let decoded = rmp_serde::decode::from_slice::<Events>(&body).unwrap();
        assert_eq!(decoded, *data.events.lock().unwrap());
    }
}
