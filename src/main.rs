mod components;

#[cfg(feature = "server")]
mod error;
#[cfg(feature = "server")]
mod functions;
#[cfg(feature = "server")]
pub mod github;
pub mod helpers;
mod models;
mod views;

use components::navbar::Navbar;
use dioxus::prelude::*;
#[cfg(feature = "server")]
pub use error::AppError;
use helpers::read_env_var;
use std::sync::LazyLock;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use views::{home::Home, run::Run, runs::Runs};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/runs")]
    Runs {},
    #[route("/run/:run")]
    Run { run: i64 },
    // #[route("/benchmark/:run")]
    // Benchmark { run: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

pub static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

fn main() {
    // Set the logger ahead of time since we don't use `dioxus::launch` on the server
    dioxus::logger::initialize_default();

    #[cfg(feature = "web")]
    // Hydrate the application on the client
    dioxus_web::launch::launch_cfg(App, dioxus_web::Config::new().hydrate(true));

    #[cfg(feature = "server")]
    {
        let _ = dotenvy::dotenv();

        use axum::routing::*;
        // use axum_session::SessionConfig;
        // use axum_session::SessionStore;
        // use axum_session_auth::AuthConfig;
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let db_address = &read_env_var("DB_ADDRESS");
                if db_address.contains("localhost") || db_address.contains("127.0.0.1") {
                    DB.connect::<surrealdb::engine::remote::ws::Ws>(db_address)
                        .await
                        .unwrap();
                } else {
                    DB.connect::<surrealdb::engine::remote::ws::Wss>(db_address)
                        .await
                        .unwrap();
                }

                let namespace = &read_env_var("DB_NAMESPACE");
                let database = &read_env_var("DB_DATABASE");
                let username = &read_env_var("DB_USERNAME");
                let password = &read_env_var("DB_PASSWORD");

                DB.signin(surrealdb::opt::auth::Database {
                    namespace,
                    database,
                    username,
                    password,
                })
                .await
                .unwrap();

                DB.use_ns(namespace).use_db(database).await.unwrap();

                github::renew_token().await.unwrap();
                // let pool = connect_to_database().await;

                //This Defaults as normal Cookies.
                //To enable Private cookies for integrity, and authenticity please check the next Example.
                // let session_config = SessionConfig::default().with_table_name("test_table");
                // let auth_config = AuthConfig::<i64>::default().with_anonymous_user_id(Some(1));
                // let session_store = SessionStore::<SessionSqlitePool>::new(
                //     Some(pool.clone().into()),
                //     session_config,
                // )
                // .await
                // .unwrap();

                // User::create_user_tables(&pool).await;

                // build our application with some routes
                let app = Router::new()
                    // Server side render the application, serve static assets, and register server functions
                    .serve_dioxus_application(ServeConfig::new().unwrap(), App)
                    // .route(
                    //     "/api/runs/",
                    //     post(crate::functions::create_run::create_run_handler),
                    // )
                    .route(
                        "/api/measurements/",
                        post(crate::functions::create_measurement::create_measurement),
                    )
                    .route("/api/github_webhook", post(crate::github::webhook::webhook));
                // .layer(
                //     axum_session_auth::AuthSessionLayer::<
                //         crate::auth::User,
                //         i64,
                //         axum_session_auth::SessionSqlitePool,
                //         sqlx::SqlitePool,
                //     >::new(Some(pool))
                //     .with_config(auth_config),
                // )
                // .layer(axum_session::SessionLayer::new(session_store));

                // serve the app using the address passed by the CLI
                let addr = dioxus_cli_config::fullstack_address_or_localhost();
                let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

                axum::serve(listener, app.into_make_service())
                    .await
                    .unwrap();
            });
    }
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}
