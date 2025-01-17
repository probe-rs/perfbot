use crate::components::run_list_entry::RunListEntry;
use crate::models::run::RunModel;
use crate::DB;
use dioxus::prelude::*;

#[component]
pub fn Runs() -> Element {
    let runs = use_server_future(get_runs)?;
    let runs = runs.value();
    let runs = runs.read();

    let runs = match &*runs {
        Some(Ok(runs)) => runs,
        Some(Err(err)) => return rsx!( "Unable to load runs: {err}" ),
        None => unreachable!(),
    };

    rsx! {
        div {
            h1 { class: "m-y-2 text-4xl text-white", "Runs" }
            table { class: "w-full border-collapse",
                {runs.iter().rev().enumerate().map(|(i, run)| rsx! {
                    tr { class: "w-full border-probe-rs-green border-solid border-[1px] hover:bg-slate-600 rounded-sm text-probe-rs-green",
                        RunListEntry { run: run.clone(), odd: i % 2 != 0 }
                    }
                })}
            }
        }
    }
}

#[server]
async fn get_runs() -> Result<Vec<RunModel>, ServerFnError> {
    let people: Vec<RunModel> = DB.select("run").await?;
    Ok(people)
}
