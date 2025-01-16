use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use super::measurement::MeasurementModel;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkModel {
    pub id: RecordId,

    pub measurements: Vec<RecordId>,

    pub name: String,
    pub description: String,
    pub unit: String,
    pub improves: BenchmarkImproves,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkMappedModel {
    pub id: RecordId,

    pub measurements: Vec<MeasurementModel>,

    pub name: String,
    pub description: String,
    pub unit: String,
    pub improves: BenchmarkImproves,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Copy)]
pub enum BenchmarkImproves {
    Up,
    Down,
}
