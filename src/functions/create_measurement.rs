use anyhow::anyhow;
use axum::Json;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

use crate::{
    models::{benchmark::BenchmarkImproves, Record},
    DB,
};

#[axum::debug_handler]
pub async fn create_measurement(
    axum::Json(run): Json<CreateMeasurement>,
) -> Result<axum::Json<Option<Record>>, crate::AppError> {
    let created: Option<Record> = DB
        .upsert(("benchmark", &run.name))
        .content(CreateBenchmarkModel {
            measurements: vec![],
            name: run.name,
            description: run.description,
            unit: run.unit,
            improves: run.improves,
        })
        .await?;
    let created: Option<Record> = DB
        .create("measurement")
        .content(CreateMeasurementModel {
            run: RecordId::from_table_key("run", run.run),
            benchmark: created.unwrap().id,
            datetime: run.datetime,
            probe: run.probe,
            chip: run.chip,
            speed_khz: run.speed_khz,
            value: run.value,
        })
        .await?;

    let Some(created) = created else {
        return Err(anyhow!("No benchmark was created")).map_err(From::from);
    };

    DB.query("BEGIN")
        .query(r#"UPDATE type::thing("run",$run) SET measurements += ($created);"#)
        .bind(("run", run.run))
        .bind(("created", created.id.clone()))
        .query("COMMIT")
        .await?
        .check()?;

    Ok(axum::Json(Some(created)))
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMeasurement {
    pub run: i64,
    pub datetime: Datetime,

    pub name: String,
    pub description: String,
    pub unit: String,
    pub improves: BenchmarkImproves,

    pub probe: String,
    pub chip: String,
    pub speed_khz: usize,

    pub value: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMeasurementModel {
    pub run: RecordId,
    pub benchmark: RecordId,

    pub datetime: Datetime,

    pub probe: String,
    pub chip: String,
    pub speed_khz: usize,

    pub value: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateBenchmarkModel {
    pub measurements: Vec<RecordId>,

    pub name: String,
    pub description: String,
    pub unit: String,
    pub improves: BenchmarkImproves,
}
