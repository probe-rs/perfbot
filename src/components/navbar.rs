use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div { class: "bg-slate-700 p-5",
            div { id: "navbar",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::Runs {}, "Runs" }
            }

            Outlet::<Route> {}
        }
    }
}
