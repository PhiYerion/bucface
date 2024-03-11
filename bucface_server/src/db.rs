use bucface_utils::{EventDB, EventDBError};
use surrealdb::Surreal;

pub const EVENTS_TABLE: &str = "events";

pub async fn insert_event<T: surrealdb::Connection>(
    event: EventDB,
    db: Surreal<T>,
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

pub async fn start_db<T: surrealdb::Connection>(
    db: &mut Surreal<T>,
) -> Result<(), surrealdb::Error> {
    db.use_ns("Bucface").await?;
    db.use_db("Events").await
}

pub async fn get_events<T: surrealdb::Connection>(
    timestamp: i64,
    db: Surreal<T>,
) -> Result<Vec<EventDB>, EventDBError> {
    log::debug!("Getting events after timestamp: {timestamp}");

    let mut response = db
        .query("SELECT * FROM type::table($table) WHERE time > type::number($timestamp)")
        .bind(("table", EVENTS_TABLE))
        .bind(("timestamp", timestamp))
        .await
        .map_err(EventDBError::Db)?;

    let events: Vec<EventDB> = response.take(0).map_err(EventDBError::Db)?;
    if events.is_empty() {
        return Err(EventDBError::NotFound);
    }

    Ok(events)
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;

    #[tokio::test]
    async fn test_start_db() {
        env_logger::try_init();

        let mut db = Surreal::new::<surrealdb::engine::local::Mem>(())
            .await
            .expect("Failed to start db");
        start_db(&mut db).await.expect("Failed to initialize db");
    }

    #[tokio::test]
    async fn test_insert_event() {
        env_logger::try_init();

        let mut db = Surreal::new::<surrealdb::engine::local::Mem>(())
            .await
            .expect("Failed to start db");
        start_db(&mut db).await.expect("Failed to initialize db");

        let mut rng = rand::thread_rng();

        for i in 0..100 {
            insert_event(EventDB::from(rng.gen(), i), db.clone())
                .await
                .expect("Failed to insert event");
        }
    }
}
