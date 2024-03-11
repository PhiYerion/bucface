use bucface_utils::{Event, EventDB, EventDBError};
use surrealdb::Surreal;

use crate::db::insert_event;

/// Handles a [rmp](rmp_serde) encoded [event](Event) by transforming it into a
/// [EventDB](EventDB) and inserting it into the [database](Surreal).
///
/// # Arguments
/// * `buf` - A [rmp](rmp_serde) encoded slice of bytes representing a [Event]
/// * `db` - A [Surreal](surrealdb::Surreal) database connection
/// * `id_count` - A type of primary key for [EventDB] structs
///
/// # Returns
/// * `Result<Event, EventDBError>` - A [Result] containing either the [Event]
/// the [database](Surreal) was updated with or an [EventDBError] if the
/// operation failed.
pub async fn handle_new_event<T: surrealdb::Connection>(
    encoded_event: &[u8],
    db: Surreal<T>,
    id_count: i64,
) -> Result<Event, EventDBError> {
    let event: Event =
        rmp_serde::decode::from_slice(encoded_event).map_err(|_| EventDBError::Rmp)?;
    let server_event = EventDB::from(event.clone(), id_count);

    match insert_event(&server_event, db).await {
        Ok(res) => {
            assert_eq!(res.len(), 1);
            assert_eq!(res[0], server_event);
            Ok(event)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod app_tests {
    use rand::Rng;
    use surrealdb::engine::local::Mem;

    use crate::db::start_db;

    use super::*;

    #[tokio::test]
    async fn test_handle_new_event() {
        let mut rng = rand::thread_rng();

        for i in 0..100 {
            let mut db = Surreal::new::<Mem>(()).await.unwrap();
            start_db(&mut db).await.unwrap();
            let event: Event = rng.gen();
            let buf = rmp_serde::encode::to_vec(&event).unwrap();

            let result = handle_new_event(&buf, db, i).await.unwrap();
            assert_eq!(result, event);
        }
    }
}
