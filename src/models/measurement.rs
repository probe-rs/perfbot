use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

use super::{benchmark::BenchmarkModel, run::RunModel};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementModel {
    pub id: RecordId,
    pub run: RecordId,
    pub benchmark: RecordId,

    pub datetime: Datetime,

    pub probe: String,
    pub chip: String,
    pub speed_khz: usize,

    pub value: f64,
    pub std: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementMappedModel {
    pub id: RecordId,
    pub run: RunModel,
    pub benchmark: BenchmarkModel,

    pub datetime: Datetime,

    pub probe: String,
    pub chip: String,
    pub speed_khz: usize,

    pub value: f64,
    pub std: f64,
}
