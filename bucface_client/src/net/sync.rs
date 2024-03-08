use std::sync::Arc;

use bucface_utils::{Event, Events};
use rmp_serde::Serializer;
use serde::Serialize;

pub enum SendEventError {
    EncodeError(rmp_serde::encode::Error),
    RequestError(reqwest::Error),
    InvalidStatusCode(reqwest::StatusCode),
}

pub async fn send_event(
    event: Event,
    server: &str,
    client: Arc<reqwest::Client>,
    max_tries: usize,
) -> Result<(), SendEventError> {
    log::info!("Sending event: {:?}", event);
    let logs = Events { inner: vec![event] };
    let mut buf = Vec::new();

    logs.serialize(&mut Serializer::new(&mut buf))
        .map_err(|e| {
            log::error!("Error encoding event: {:?}", e);
            SendEventError::EncodeError(e)
        })?;

    let mut counter = 0;
    loop {
        match client.post(server).body(buf.clone()).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    break;
                }
                log::error!("Invalid status code: {:?}", res.status());
                counter += 1;
                if counter > max_tries {
                    return Err(SendEventError::InvalidStatusCode(res.status()));
                }
            }
            Err(e) => {
                log::error!("Error sending event: {:?}", e);
                counter += 1;
                if counter > max_tries {
                    return Err(SendEventError::RequestError(e));
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }

    log::info!("Event sent");

    Ok(())
}

/* struct GetEventsFuture {
    events: Option<Result<Events, UpdateLogsError>>,
}

impl Future for GetEventsFuture {
    type Output = Result<Events, UpdateLogsError>;
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match &self.events {
            Some(result) => std::task::Poll::Ready(result),
            None => std::task::Poll::Pending,
        }
    }
}
*/

#[derive(Debug)]
pub enum UpdateLogsError {
    Reqwest(reqwest::Error),
    InvalidStatusCode(reqwest::StatusCode),
    Rmp(rmp_serde::decode::Error),
}

pub async fn get_events(
    server: String,
    client: reqwest::Client,
    start_index: usize,
) -> Result<Events, UpdateLogsError> {
    let url = server + "/" + &start_index.to_string();
    log::info!("Getting events with url: {}", url);
    let res = client.get(url).send().await.map_err(|e| {
        log::error!("Error getting events: {}", e);
        UpdateLogsError::Reqwest(e)
    })?;
    if !res.status().is_success() {
        let error = Err(UpdateLogsError::InvalidStatusCode(res.status()));
        log::error!("Error getting events: {:?}", error);
        return error;
    }

    let bytes = res.bytes().await.map_err(UpdateLogsError::Reqwest)?;
    log::info!("Got events: {:?}", bytes);

    rmp_serde::from_slice::<Events>(&bytes).map_err(UpdateLogsError::Rmp)
}
