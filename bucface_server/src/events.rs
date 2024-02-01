use actix_web::{get, post, web, HttpResponse};
use bucface_utils::Events;
use futures::StreamExt;

use crate::AppState;

// If you are wondering how we can "return" Events, it is serialized using Events's Responder implementation
#[get("/events/{index}")]
pub async fn get_events(path: web::Path<usize>, data: web::Data<AppState>) -> Option<Events> {
    log::debug!("Got get request");
    let index = path.into_inner();
    let events = data.events.lock().ok()?;

    let res = Some(Events {
        inner: events.inner.get(index..)?.to_vec(),
    });
    log::debug!("Returning {} events", res.as_ref().unwrap().inner.len());

    res
}

const MAX_SIZE: usize = 1_048_576; // max payload size is 1M
#[post("/events/create")]
pub async fn create_event(
    mut payload: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    log::debug!("Got post request");
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
