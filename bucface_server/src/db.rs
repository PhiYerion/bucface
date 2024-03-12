use bucface_utils::{EventDB, EventDBError};
use surrealdb::Surreal;

pub const EVENTS_TABLE: &str = "events";

/// Inserts an [EventDB](EventDB) into the [database](Surreal), returning the
/// [database](Surreal) response, which contains the [Vec] of [EventDB] that
/// was inserted.
pub async fn insert_event<T: surrealdb::Connection>(
    event: &EventDB,
    db: &Surreal<T>,
) -> Result<Vec<EventDB>, EventDBError> {
    log::debug!("Inserting event: {event:?}");

    log::debug!(
        "events: {:?}",
        db.select::<Vec<EventDB>>(EVENTS_TABLE).await.unwrap()
    );

    db.create::<Vec<EventDB>>(EVENTS_TABLE)
        .content(event)
        .await
        .map_err(EventDBError::Db)
}

/// Initializes the [database](Surreal) by setting the namespace to "Bucface"
/// and the database to "Events".
pub async fn start_db<T: surrealdb::Connection>(
    db: &mut Surreal<T>,
) -> Result<(), surrealdb::Error> {
    db.use_ns("Bucface").await?;
    db.use_db("Events").await
}

/// Gets all [EventDB]s since and including the given id.
pub async fn get_events_since<T: surrealdb::Connection>(
    id: u64,
    db: &Surreal<T>,
) -> Result<Vec<EventDB>, EventDBError> {
    log::debug!("Getting events after id: {id}");

    let mut response = db
        .query("SELECT * FROM type::table($table) WHERE _id >= type::number($id)")
        .bind(("table", EVENTS_TABLE))
        .bind(("id", id))
        .await
        .map_err(EventDBError::Db)?;

    let events: Vec<EventDB> = response.take(0).map_err(EventDBError::Db)?;
    if events.is_empty() {
        return Err(EventDBError::NotFound);
    }

    Ok(events)
}

pub async fn get_event<T: surrealdb::Connection>(
    id: u64,
    db: &Surreal<T>,
) -> Result<EventDB, EventDBError> {
    let mut query = db
        .query("SELECT * FROM type::table($table) WHERE _id == type::number($id)")
        .bind(("table", EVENTS_TABLE))
        .bind(("id", id))
        .await
        .map_err(EventDBError::Db)?;

    let event = query
        .take::<Option<EventDB>>(0)
        .map_err(EventDBError::Db)?
        .ok_or(EventDBError::NotFound)?;

    Ok(event)
}

#[cfg(test)]
mod db_tests {
    use rand::Rng;

    use super::*;

    #[tokio::test]
    async fn test_start_db() {
        let _ = env_logger::try_init();

        let mut db = Surreal::new::<surrealdb::engine::local::Mem>(())
            .await
            .expect("Failed to start db");
        start_db(&mut db).await.expect("Failed to initialize db");
    }

    #[tokio::test]
    async fn test_insert_event() {
        let _ = env_logger::try_init();

        let mut db = Surreal::new::<surrealdb::engine::local::Mem>(())
            .await
            .expect("Failed to start db");
        start_db(&mut db).await.expect("Failed to initialize db");

        let mut rng = rand::thread_rng();

        for i in 0..100 {
            insert_event(&EventDB::from(rng.gen(), i), &db)
                .await
                .expect("Failed to insert event");
        }
    }

    #[tokio::test]
    async fn test_get_events() {
        let _ = env_logger::try_init();

        let mut db = Surreal::new::<surrealdb::engine::local::Mem>(())
            .await
            .expect("Failed to start db");
        start_db(&mut db).await.expect("Failed to initialize db");

        let mut rng = rand::thread_rng();

        let events = (0..100)
            .map(|i| EventDB::from(rng.gen(), i))
            .collect::<Vec<EventDB>>();

        for event in &events {
            let response = insert_event(event, &db)
                .await
                .expect("Failed to insert event");
            assert_eq!(response.len(), 1);
            assert_eq!(response[0], *event);
        }

        let mut new_events = get_events_since(0, &db)
            .await
            .expect("Failed to get events");

        new_events.sort_by(|a, b| a._id.cmp(&b._id));
        assert_eq!(events.len(), new_events.len());

        for (event, new_event) in events.iter().zip(new_events.iter()) {
            assert_eq!(event, new_event);
        }
    }
}
