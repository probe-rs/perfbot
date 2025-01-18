use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::benchmark::BenchmarkImproves;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn BenchmarkListEntry(benchmark: Benchmark, odd: bool) -> Element {
    let improved = benchmark.improved();

    let color = if improved {
        "text-green-500"
    } else {
        "text-red-500"
    };

    let status_text = benchmark.status_text();
    let percent_change = benchmark.percent_change_text();
    let value = benchmark.value_text();

    let background = if !odd { "bg-gray-300" } else { "" };

    rsx! {
        td { class: "p-2 px-3 {background}",
            h1 { class: "text-2xl", "{benchmark.name}" }
            p { "{benchmark.description}" }
        }
        td { class: "p-2 px-3 {background}",
            "{value} {benchmark.unit} ("
            span { class: color, "{benchmark.diff:.02} {benchmark.unit}" }
            ")"
        }
        td { class: "p-2 px-3 {background}",
            span { class: color, "{percent_change}" }
        }
        td { class: "p-2 px-3 {background}",
            span { class: "{color} ml-2", "{status_text}" }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Benchmark {
    pub name: String,
    pub description: String,
    pub value: f64,
    pub std: f64,
    pub diff: f64,
    pub percentage: f64,
    pub unit: String,
    pub improves: BenchmarkImproves,
}

impl Benchmark {
    pub fn improved(&self) -> bool {
        (match (self.diff >= 0.0, &self.improves) {
            (true, BenchmarkImproves::Up) => true,
            (true, BenchmarkImproves::Down) => false,
            (false, BenchmarkImproves::Up) => false,
            (false, BenchmarkImproves::Down) => true,
        }) || self.diff == 0.0
    }

    pub fn status_text(&self) -> &str {
        if self.improved() {
            "IMPROVED"
        } else {
            "DECLINED"
        }
    }

    pub fn percent_change(&self) -> f64 {
        -(self.percentage - 1.0)
    }

    pub fn percent_change_text(&self) -> String {
        format!("{:.02} %", self.percent_change() * 100.0)
    }

    pub fn value_text(&self) -> String {
        format!("{:.02} ± {:.02}", self.value, self.std)
    }
}
