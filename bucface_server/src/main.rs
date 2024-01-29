use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use bucface_utils::Events;
use futures::StreamExt;

#[derive(Debug, Clone, Default)]
struct AppState {
    events: Arc<Mutex<Events>>,
}

// If you are wondering how we can "return" Events, it is serialized using Events's Responder implementation
#[get("/events")]
async fn events(data: web::Data<AppState>) -> Option<Events> {
    match data.events.lock() {
        Ok(events) => Some(events.clone()),
        Err(_) => None,
    }
}

const MAX_SIZE: usize = 1_048_576; // max payload size is 1M
#[post("/events")]
async fn create_event(
    mut payload: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_SIZE {
            log::warn!("Overflow");
            return Err(actix_web::error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let decoded = match rmp_serde::from_slice::<Events>(&body) {
        Ok(decoded) => decoded,
        Err(e) => {
            log::warn!("Bad request: {}", e);
            return Err(actix_web::error::ErrorBadRequest(e));
        }
    };

    let mut data_events = match data.events.lock() {
        Ok(app_data_events) => app_data_events,
        Err(e) => {
            log::error!("Mutex error: {}", e);
            return Err(actix_web::error::ErrorInternalServerError(e.to_string()));
        }
    };

    data_events.inner.extend_from_slice(&decoded.inner);

    log::info!("Received {} events", decoded.inner.len());
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = AppState::default();
    HttpServer::new(move || {
        App::new()
            .service(events)
            .service(create_event)
            .app_data(data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::MessageBody;
    use actix_web::{http, test, web, App};
    use rand::Rng;
    use rmp_serde::Serializer;
    use serde::Serialize;

    #[actix_web::test]
    async fn send_events() {
        env_logger::init();
        let data = AppState::default();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(data.clone()))
                .service(events)
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
    async fn get_events() {
        let test_events: Events = rand::thread_rng().gen();
        let data = AppState {
            events: Arc::new(Mutex::new(test_events)),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(data.clone()))
                .service(events)
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
