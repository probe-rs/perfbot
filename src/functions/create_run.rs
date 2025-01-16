use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

use crate::{
    models::{run::RunModel, Record},
    DB,
};

// #[axum::debug_handler]
// pub async fn create_run_handler(
//     Json(run): Json<CreateRunModel>,
// ) -> Result<Json<Option<Record>>, crate::AppError> {
//     let created: Option<Record> = DB.create(("run", run.run_id)).content(run).await?;
//     Ok(Json(created))
// }

pub async fn ensure_run(
    run_id: i64,
    run: CreateRunModel,
) -> Result<Option<Record>, crate::AppError> {
    let created: Option<Record> = DB.upsert(("run", run_id)).content(run).await?;
    Ok(created)
}

pub async fn get_run(commit: String) -> Result<Option<RunModel>> {
    let mut run = DB
        .query("SELECT * FROM run WHERE commit=$commit")
        .bind(("commit", commit))
        .await?;
    let run: Option<RunModel> = run.take(0)?;
    Ok(run)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunModel {
    pub datetime: Datetime,

    pub author: String,

    pub pr: i64,

    pub commit: String,
    pub previous: Option<RecordId>,

    #[serde(default)]
    pub measurements: Vec<RecordId>,
}
