use bucface_utils::Event;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Mem;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

enum EventDBError {
    Db(surrealdb::Error),
    NotFound,
    Rmp,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

const EVENTS_TABLE: &str = "events";

async fn insert_event<T: surrealdb::Connection>(
    event: Event,
    db: Surreal<T>,
) -> Result<(), EventDBError> {
    let _ = db
        .create::<Vec<Event>>(EVENTS_TABLE)
        .content(event)
        .await
        .map_err(EventDBError::Db);

    Ok(())
}

async fn get_events<T: surrealdb::Connection>(
    timestamp: i64,
    db: Surreal<T>,
) -> Result<Vec<Event>, EventDBError> {
    let mut response = db
        .query("SELECT * FROM type::table($table) WHERE time > $timestamp")
        .bind(("table", EVENTS_TABLE))
        .bind(("timestamp", timestamp))
        .await
        .map_err(EventDBError::Db)?;

    let events: Vec<Event> = response.take(0).map_err(EventDBError::Db)?;
    if events.is_empty() {
        return Err(EventDBError::NotFound);
    }

    Ok(events)
}
