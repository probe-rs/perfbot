use crate::components::run_list_entry::RunListEntry;
use crate::models::run::RunModel;
use crate::DB;
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let runs = use_server_future(get_runs)?;
    let runs = runs.value();
    let runs = runs.read();

    let runs = match &*runs {
        Some(Ok(runs)) => runs,
        Some(Err(err)) => return rsx!("Unable to load runs: {err}"),
        None => unreachable!(),
    };

    rsx! {
        div { class: "p-5",
            table { class: "w-full",
                {runs.iter().enumerate().map(|(i, run)| rsx! {
                    tr { class: "w-full odd:border-probe-rs-green hover:bg-probe-rs-green",
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
