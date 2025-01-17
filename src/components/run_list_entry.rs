use crate::{helpers::read_env_var, models::run::RunModel};
use chrono::SecondsFormat;
use dioxus::prelude::*;

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn RunListEntry(run: RunModel, odd: bool) -> Element {
    let date = run
        .datetime
        .into_inner_ref()
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    let org = read_env_var("GITHUB_ORG");
    let repo = read_env_var("GITHUB_REPO");
    rsx! {
        td { class: "p-2 px-3",
            {
                rsx! {
                    Link {
                        class: "hover:underline",
                        to: crate::Route::Run {
                            run: run.id.key().to_string().parse().unwrap(),
                        },
                        "{run.id}"
                    }
                }
            }
        }
        td { class: "p-2 px-3", "{date}" }
        td { class: "p-2 px-3",
            a {
                class: "hover:underline",
                href: "https://github.com/{org}/{repo}/pull/{run.pr}/commits/{run.commit}",
                "{run.commit}"
            }
        }
        td { class: "p-2 px-3",
            a {
                class: "hover:underline",
                href: "https://github.com/{run.author}",
                "{run.author}"
            }
        }
        td { class: "p-2 px-3", {run.pr_url_element(&org, &repo)} }
        td { class: "p-2 px-3", "{run.measurements.len()}" }
    }
}
