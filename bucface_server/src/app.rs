use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use bucface_utils::{ClientMessage, EventDB, EventDBError};
use surrealdb::Surreal;

use crate::db::{get_event, get_events_since, insert_event};

/// Handles a [rmp](rmp_serde) encoded [ClientMessage] by updating the database
/// and echoing the updated [EventDB]s or returning the requested [EventDB]s.
///
/// # Arguments
/// * `buf` - A [rmp](rmp_serde) encoded slice of bytes representing a [Event]
/// * `db` - A [Surreal](surrealdb::Surreal) database connection
/// * `id_count` - A type of primary key for [EventDB] structs
///
/// # Returns
/// The return is intended to be sent back to the client, but can be handled in
/// any way the caller sees fit.
///
/// * `Result<EventDB, EventDBError>`
/// - In the case of [ClientMessage::NewEvent], returns [Result] containing
/// the [EventDB] the [database](Surreal) was updated with or an [EventDBError]
/// if the operation failed.
/// - In the case of [ClientMessage::GetEvent], returns [Result] containing
/// the [EventDB] requested or an [EventDBError] if the operation failed.
/// - In the case of [ClientMessage::GetSince], returns [Result] containing
/// a [Vec] of [EventDB]s requested or an [EventDBError] if the operation
/// failed.
///
/// # Notes
/// This function is kind of dumb. It is tailored to be called in a singular
/// place and is too tailored to that purpose. Either abstract the wriiter into
/// this function in websockets.rs or put this function back into websocket.rs.
/// This function was just because the indentation was getting to me.
pub async fn handle_client_message<T: surrealdb::Connection>(
    message: &[u8],
    db: &Surreal<T>,
    id_count: Arc<AtomicI64>,
) -> Result<Vec<EventDB>, (EventDBError, i64)> {
    let id = id_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let message: ClientMessage =
        rmp_serde::decode::from_slice(message).map_err(|e| (EventDBError::RmpDecode(e), id))?;

    match message {
        ClientMessage::NewEvent(event) => {
            let server_event = EventDB::from(event, id);
            let db_response = insert_event(&server_event, db).await.map_err(|e| (e, id))?;
            assert_eq!(db_response.len(), 1);
            assert_eq!(db_response[0], server_event);
            Ok(db_response)
        }
        ClientMessage::GetEvent(id) => match get_event(id, db).await {
            Ok(event) => Ok(vec![event]),
            Err(e) => Err((e, id)),
        },
        ClientMessage::GetSince(timestamp) => {
            get_events_since(timestamp, db).await.map_err(|e| (e, id))
        }
    }
}

#[cfg(test)]
mod app_tests {
    use bucface_utils::Event;
    use rand::Rng;
    use surrealdb::engine::local::Mem;

    use crate::db::start_db;

    use super::*;

    #[tokio::test]
    async fn test_handle_new_event() {
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let mut db = Surreal::new::<Mem>(()).await.unwrap();
            let id_counter = Arc::new(AtomicI64::new(0));
            start_db(&mut db).await.unwrap();

            let event: Event = rng.gen();
            let client_message = ClientMessage::NewEvent(event.clone());
            let buf = rmp_serde::encode::to_vec(&client_message).unwrap();
            let result = handle_client_message(&buf, &db, id_counter).await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].event, event.event);
        }
    }

    #[tokio::test]
    async fn test_handle_get_since() {
        let mut rng = rand::thread_rng();
        let mut db = Surreal::new::<Mem>(()).await.unwrap();
        let id_counter = Arc::new(AtomicI64::new(0));
        start_db(&mut db).await.unwrap();

        let send_message = |message: ClientMessage| {
            let buf = rmp_serde::encode::to_vec(&message).unwrap();
            let db = db.clone();
            let id_counter = id_counter.clone();
            async move { handle_client_message(&buf, &db, id_counter).await }
        };

        let mut events = (0..10).map(|_| rng.gen()).collect::<Vec<Event>>();
        for event in &events {
            let response = send_message(ClientMessage::NewEvent(event.clone()))
                .await
                .unwrap();
            assert_eq!(response.len(), 1);
            assert_eq!(response[0].event, event.event);
        }

        let mut result = send_message(ClientMessage::GetSince(0)).await.unwrap();
        result.sort_by(|a, b| a._id.cmp(&b._id));

        let events_db = events
            .drain(..)
            .enumerate()
            .map(|(i, event)| EventDB::from(event, i.try_into().unwrap()))
            .collect::<Vec<EventDB>>();

        assert_eq!(result.len(), events_db.len());
        assert_eq!(result, events_db);
    }
}
