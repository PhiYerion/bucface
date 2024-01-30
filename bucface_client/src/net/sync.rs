use bucface_utils::{Event, Events};
use rmp_serde::Serializer;
use serde::Serialize;

use crate::app::UpdateLogsError;

pub enum SendEventError {
    EncodeError(rmp_serde::encode::Error),
    RequestError(reqwest::Error),
}

pub async fn send_event(
    event: Event,
    server: &str,
    client: reqwest::Client,
    max_tries: usize,
) -> Result<(), SendEventError> {
    let logs = Events { inner: vec![event] };
    let mut buf = Vec::new();

    logs.serialize(&mut Serializer::new(&mut buf))
        .map_err(SendEventError::EncodeError);

    let mut counter = 0;
    while let Err(e) = client.post(server).body(buf.clone()).send().await {
        counter += 1;
        if counter > max_tries {
            return Err(SendEventError::RequestError(e));
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}

pub async fn get_events(
    server: String,
    client: reqwest::Client,
    start_index: usize,
) -> Result<Events, UpdateLogsError> {
    let res = client
        .get(server + &start_index.to_string())
        .send()
        .await
        .map_err(UpdateLogsError::Reqwest)?;
    if !res.status().is_success() {
        return Err(UpdateLogsError::InvalidStatusCode(res.status()));
    }
    let bytes = res.bytes().await.map_err(UpdateLogsError::Reqwest)?;

    Ok(rmp_serde::from_slice::<Events>(&bytes).map_err(UpdateLogsError::Rmp)?)
}
