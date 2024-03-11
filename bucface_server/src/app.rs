use bucface_utils::{Event, EventDB, EventDBError};
use surrealdb::Surreal;

use crate::db::insert_event;

pub async fn handle_new_event<T: surrealdb::Connection>(
    buf: &[u8],
    db: Surreal<T>,
    id_count: i64,
) -> Result<Event, EventDBError> {
    let event: Event = rmp_serde::decode::from_slice(buf).map_err(|_| EventDBError::Rmp)?;
    let server_event = EventDB::from(event.clone(), id_count);

    match insert_event(server_event.clone(), db).await {
        Ok(res) => {
            assert_eq!(res.len(), 1);
            assert_eq!(res[0], server_event);
            Ok(event)
        }
        Err(e) => Err(e),
    }
}
