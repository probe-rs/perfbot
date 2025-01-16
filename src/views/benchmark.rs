// use crate::components::benchmark_list_entry::{Bench, BenchmarkListEntry};
// use crate::models::run::{RunMappedModel, RunMappedParent};
// use crate::DB;
// use dioxus::prelude::*;

// #[component]
// pub fn Benchmark(run: String) -> Element {
//     let runs = use_server_future(move || get_benchmark(run.clone()))?;
//     let runs = runs.value();
//     let runs = runs.read();

//     let (run_a, run_b) = match &*runs {
//         Some(Ok(runs)) => runs,
//         Some(Err(err)) => return rsx!("Unable to load runs: {err}"),
//         None => unreachable!(),
//     };

//     let mut benches = vec![];
//     for a in &run_a.benchmarks {
//         let b = run_b.benchmarks.iter().find(|b| {
//             b.chip == a.chip && b.probe == a.probe && b.speed_khz == a.speed_khz && b.name == a.name
//         });

//         if let Some(b) = b {
//             benches.push(Bench {
//                 name: a.name.clone(),
//                 description: a.description.clone(),

//                 diff: a.value - b.value,
//                 percentage: a.value / b.value,

//                 unit: a.unit.clone(),
//             })
//         }
//     }

//     rsx! {
//         div { class: "p-5",
//             table { class: "w-full",
//                 {benches.iter().enumerate().map(|(i, bench)| rsx! {
//                     tr { class: "w-full odd:border-probe-rs-green hover:bg-probe-rs-green",
//                         BenchmarkListEntry { benchmark: bench.clone(), odd: i % 2 == 0 }
//                     }
//                 })}
//             }
//         }
//     }
// }

// #[server]
// async fn get_benchmark(run: String) -> Result<(RunMappedModel, RunMappedParent), ServerFnError> {
//     let mut results = DB
//         .query("SELECT * FROM run WHERE id = type::thing(\"run\", $run) FETCH benchmarks, previous, previous.benchmarks")
//         .bind(("run", run.clone()))
//         .await?;

//     let run_current: Option<RunMappedModel> = results.take(0)?;
//     let Some(run_current) = run_current else {
//         return Err(format!("Run {run} not found")).map_err(ServerFnError::new);
//     };

//     let previous_id = run_current
//         .previous
//         .as_ref()
//         .map(|p| p.id.to_owned())
//         .unwrap();
//     let mut results = DB
//         .query("SELECT * FROM run WHERE id = $run FETCH benchmarks")
//         .bind(("run", previous_id.clone()))
//         .await?;

//     let run_previous: Option<RunMappedParent> = results.take(0)?;
//     let Some(run_previous) = run_previous else {
//         return Err(format!("Run {previous_id:?} not found")).map_err(ServerFnError::new);
//     };

//     Ok((run_current, run_previous))
// }
