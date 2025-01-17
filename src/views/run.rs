use crate::components::benchmark_list_entry::{Benchmark, BenchmarkListEntry};
use crate::helpers::read_env_var;
use crate::models::run::RunMappedModel;
use crate::DB;
use dioxus::prelude::*;

#[component]
pub fn Run(run: i64) -> Element {
    let benchmarks = use_server_future(move || get_benchmark_measurements(run))?;
    let benchmarks = benchmarks.value();
    let benchmarks = benchmarks.read();

    let (run, benchmarks) = match &*benchmarks {
        Some(Ok(benchmarks)) => benchmarks,
        Some(Err(err)) => return rsx!( "Unable to load benchmarks: {err}" ),
        None => unreachable!(),
    };

    let org = read_env_var("GITHUB_ORG");
    let repo = read_env_var("GITHUB_REPO");

    rsx! {
        div {
            h1 { class: "m-y-2 text-4xl text-white",
                "Run for "
                a {
                    class: "hover:underline",
                    href: "https://github.com/{org}/{repo}/pull/{run.pr}/commits/{run.commit}",
                    "{run.commit}"
                }
            }
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
pub async fn get_benchmark_measurements(
    run: i64,
) -> Result<(RunMappedModel, Vec<Benchmark>), ServerFnError> {
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

    let benchmarks = benchmarks.collect();
    Ok((current_run, benchmarks))
}
