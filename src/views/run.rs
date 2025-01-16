use crate::components::benchmark_list_entry::{Benchmark, BenchmarkListEntry};
use crate::models::run::RunMappedModel;
use crate::DB;
use dioxus::prelude::*;

#[component]
pub fn Run(run: i64) -> Element {
    let runs = use_server_future(move || get_benchmark_measurements(run))?;
    let runs = runs.value();
    let runs = runs.read();

    let benchmarks = match &*runs {
        Some(Ok(runs)) => runs,
        Some(Err(err)) => return rsx!("Unable to load runs: {err}"),
        None => unreachable!(),
    };

    rsx! {
        div { class: "p-5",
            table { class: "w-full border-collapse",
                {
                    benchmarks
                        .iter()
                        .enumerate()
                        .map(|(i, benchmark)| rsx! {
                            tr { class: "w-full group hover:bg-gray-400 border-[1px]",
                                BenchmarkListEntry { benchmark: benchmark.clone(), odd: i % 2 == 0 }
                            }
                        })
                }
            }
        }
    }
}

#[server]
pub async fn get_benchmark_measurements(run: i64) -> Result<Vec<Benchmark>, ServerFnError> {
    let mut results = DB
        .query(
            r#"SELECT * FROM run
            WHERE id = type::thing("run", $run)
            FETCH
                measurements,
                measurements.run,
                measurements.benchmark,
                previous,
                previous.measurements
            "#,
        )
        .bind(("run", run))
        .await?;
    let current_run: Option<RunMappedModel> = results.take(0)?;

    let Some(current_run) = current_run else {
        return Err(format!("Run {run} not found")).map_err(ServerFnError::new);
    };

    let benchmarks = current_run.measurements.iter().map(|new| {
        if let Some(old) = current_run.previous.as_ref().and_then(|p| {
            p.measurements.iter().find(|old| {
                old.chip == new.chip
                    && old.probe == new.probe
                    && old.speed_khz == new.speed_khz
                    && old.benchmark == new.benchmark.id
            })
        }) {
            Benchmark {
                name: new.benchmark.name.clone(),
                description: new.benchmark.description.clone(),
                abs: new.value,
                diff: old.value - new.value,
                percentage: old.value / new.value,

                unit: new.benchmark.unit.clone(),
                improves: new.benchmark.improves,
            }
        } else {
            Benchmark {
                name: new.benchmark.name.clone(),
                description: new.benchmark.description.clone(),
                abs: new.value,
                diff: 0.0,
                percentage: 0.0,

                unit: new.benchmark.unit.clone(),
                improves: new.benchmark.improves,
            }
        }
    });

    Ok(benchmarks.collect())
}
