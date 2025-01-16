use crate::models::run::RunModel;
use dioxus::prelude::*;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn RunListEntry(run: RunModel, odd: bool) -> Element {
    rsx! {
        td { class: "p-2 px-3 rounded-s-md",
            {
                rsx! {
                    Link {
                        to: crate::Route::Run {
                            run: run.id.key().to_string().parse().unwrap(),
                        },
                        "{run.id}"
                    }
                }
            }
        }
        td { class: "p-2 px-3", "{run.datetime}" }
        td { class: "p-2 px-3", "{run.author}" }
        td { class: "p-2 px-3", {run.pr_url_element()} }
        td { class: "p-2 px-3 rounded-e-md", "{run.measurements.len()}" }
    }
}
