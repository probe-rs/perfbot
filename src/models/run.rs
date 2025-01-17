use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use surrealdb::{Datetime, RecordId};

use super::measurement::{MeasurementMappedModel, MeasurementModel};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RunModel {
    pub id: RecordId,

    pub datetime: Datetime,

    pub author: String,

    pub pr: i64,

    pub commit: String,
    pub previous: Option<RecordId>,

    pub measurements: Vec<RecordId>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RunMappedModel {
    pub id: RecordId,

    pub datetime: Datetime,

    pub author: String,

    pub pr: i64,

    pub commit: String,
    pub previous: Option<RunMappedParentModel>,

    pub measurements: Vec<MeasurementMappedModel>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RunMappedParentModel {
    pub id: RecordId,

    pub datetime: Datetime,

    pub author: String,

    pub pr: i64,

    pub commit: String,
    pub previous: Option<RecordId>,

    pub measurements: Vec<MeasurementModel>,
}

impl RunModel {
    pub fn pr_url_element(&self, org: &str, repo: &str) -> Element {
        rsx! {
            a { href: "https://github.com/{org}/{repo}/pull/{self.pr}", "#{self.pr}" }
        }
    }
}
