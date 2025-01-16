use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

pub mod benchmark;
pub mod measurement;
pub mod run;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Record {
    pub id: RecordId,
}
