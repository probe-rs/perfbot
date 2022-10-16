mod config;
pub mod db;
mod matrix;

use std::sync::Mutex;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use askama::Template;
use config::Config;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use diesel_migrations::embed_migrations;
use figment::map;
use octocrab::Octocrab;
use openssl::hash::MessageDigest;
use perfbot_common::{schema::logs, Log, NewLog};
use rocket::{self, get, post, routes, Shutdown, State};
use rocket_contrib::{
    database, json::Json, serve::StaticFiles, templates::Template as RocketTemplate,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::Runtime;

const GH_ORG: &str = "probe-rs";
const GH_REPO: &str = "probe-rs";
const APP_ID: u64 = 93972;
const INSTALLATION_ID: u64 = 13730372;

embed_migrations!();

#[get("/")]
fn index() -> RocketTemplate {
    let context = BTreeMap::<String, String>::new();
    RocketTemplate::render("index", &context)
}

#[derive(Serialize, Deserialize)]
struct ListResponse {
    error: Option<String>,
    logs: Vec<Log>,
}

#[get("/list?<probe>&<chip>&<os>&<kind>&<protocol>&<protocol_speed>")]
async fn list(
    db: Database,
    probe: Option<String>,
    chip: Option<String>,
    os: Option<String>,
    kind: Option<String>,
    protocol: Option<String>,
    protocol_speed: Option<i32>,
) -> Json<ListResponse> {
    Json(
        db.run(move |c| {
            let query = logs::table.into_boxed();
            let query = if let Some(probe) = probe {
                query.filter(logs::probe.eq(probe.to_ascii_lowercase()))
            } else {
                query
            };
            let query = if let Some(chip) = chip {
                query.filter(logs::chip.eq(chip.to_ascii_lowercase()))
            } else {
                query
            };
            let query = if let Some(os) = os {
                query.filter(logs::os.eq(os.to_ascii_lowercase()))
            } else {
                query
            };
            let query = if let Some(kind) = kind {
                query.filter(logs::kind.eq(kind.to_ascii_lowercase()))
            } else {
                query
            };
            let query = if let Some(protocol) = protocol {
                query.filter(logs::protocol.eq(protocol.to_ascii_lowercase()))
            } else {
                query
            };
            let query = if let Some(protocol_speed) = protocol_speed {
                query.filter(logs::protocol_speed.eq(protocol_speed))
            } else {
                query
            };
            query
                .load::<Log>(c)
                .map(|l| ListResponse {
                    error: None,
                    logs: l,
                })
                .unwrap_or_else(|e| ListResponse {
                    error: Some(format!("{:?}", e)),
                    logs: vec![],
                })
        })
        .await,
    )
}

#[derive(Serialize, Deserialize)]
struct ChipsResponse {
    error: Option<String>,
    chips: Vec<String>,
}

#[get("/chips")]
async fn chips(db: Database) -> Json<ChipsResponse> {
    Json(
        db.run(move |c| {
            let chips = logs::table
                .group_by(logs::chip)
                .select(logs::chip)
                .load(c)
                .map(|l| ChipsResponse {
                    error: None,
                    chips: l,
                })
                .unwrap_or_else(|e| ChipsResponse {
                    error: Some(format!("{:?}", e)),
                    chips: vec![],
                });
            chips
        })
        .await,
    )
}

#[derive(Serialize, Deserialize)]
struct ProbesResponse {
    error: Option<String>,
    probes: Vec<String>,
}

#[get("/probes")]
async fn probes(db: Database) -> Json<ProbesResponse> {
    Json(
        db.run(move |c| {
            let probes = logs::table
                .group_by(logs::probe)
                .select(logs::probe)
                .load(c)
                .map(|l| ProbesResponse {
                    error: None,
                    probes: l,
                })
                .unwrap_or_else(|e| ProbesResponse {
                    error: Some(format!("{:?}", e)),
                    probes: vec![],
                });
            probes
        })
        .await,
    )
}

#[derive(Serialize, Deserialize)]
struct SetupsResponse {
    error: Option<String>,
    setups: Vec<(String, String)>,
}

#[get("/setups")]
async fn setups(db: Database) -> Json<SetupsResponse> {
    Json(
        db.run(move |c| {
            let setups = logs::table
                .group_by((logs::probe, logs::chip))
                .select((logs::probe, logs::chip))
                .load(c)
                .map(|l| SetupsResponse {
                    error: None,
                    setups: l,
                })
                .unwrap_or_else(|e| SetupsResponse {
                    error: Some(format!("{:?}", e)),
                    setups: vec![],
                });
            setups
        })
        .await,
    )
}

struct Pr {
    number: u64,
    benchmarks: usize,
}

#[derive(Template)]
#[template(path = "commands/perf.html")]
struct PerfTemplate {
    prs: Vec<Pr>,
}

#[derive(Template)]
#[template(path = "commands/help.html")]
struct HelpTemplate {}

async fn matrix(
    username: &str,
    password: &str,
    gh_key: Vec<u8>,
    database_path: String,
    json_store: &str,
    shutdown: Arc<Mutex<bool>>,
) {
    matrix::login_and_sync(
        "https://matrix.org",
        username,
        password,
        gh_key,
        database_path,
        json_store,
        shutdown,
    )
    .await
    .unwrap()
}

fn main() {
    let config = config::get_config();

    let gh_key = std::fs::read(config.github_key_path.clone()).unwrap();

    migrate(&config.database_path);

    let rocket_shutdown = Arc::new(Mutex::new(None));
    let rocket_shutdown_clone = rocket_shutdown.clone();
    let matrix_shutdown = Arc::new(Mutex::new(false));
    let matrix_shutdown_clone = matrix_shutdown.clone();
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        if let Some::<Shutdown>(shdn) = rocket_shutdown_clone.lock().unwrap().clone() {
            shdn.shutdown();
        }
        *matrix_shutdown_clone.lock().unwrap() = true;
    })
    .expect("Error setting Ctrl-C handler");

    let mut rt = Runtime::new().unwrap();
    let gh_key_clone = gh_key.clone();
    let cloned_config = config.clone();
    rt.spawn(async move { rocket(&cloned_config, gh_key_clone, rocket_shutdown).await });
    rt.block_on(matrix(
        &config.matrix_user,
        &config.matrix_password,
        gh_key,
        config.database_path.clone(),
        &config.matrix_json_store,
        matrix_shutdown,
    ));
}

fn migrate(database_path: &str) {
    let connection = db::establish_connection(database_path);

    // This will run the necessary migrations.
    embedded_migrations::run(&connection).unwrap();

    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
}

async fn rocket(
    config: &Config,
    gh_key: Vec<u8>,
    handle: Arc<Mutex<Option<Shutdown>>>,
) -> Result<(), rocket::error::Error> {
    let rocket = rocket::custom(
        rocket::Config::figment()
            .merge(("address", &config.address))
            .merge(("port", config.port))
            .merge((
                "databases",
                map!["database" => map!["url" => &config.database_path]],
            )),
    )
    .mount("/", routes![index, list, add, chips, probes, setups])
    .attach(Database::fairing())
    .attach(RocketTemplate::fairing())
    .manage(GhKey { key: gh_key })
    .mount("/static", StaticFiles::from("./static"));
    *handle.lock().unwrap() = Some(rocket.shutdown());

    rocket.launch().await
}

#[derive(Serialize, Deserialize)]
struct AddResponse {
    error: Option<String>,
    log: Option<Log>,
}

impl From<QueryResult<Log>> for AddResponse {
    fn from(r: QueryResult<Log>) -> Self {
        match r {
            Ok(log) => AddResponse {
                error: None,
                log: Some(log),
            },
            Err(e) => AddResponse {
                error: Some(format!("{:?}", e)),
                log: None,
            },
        }
    }
}

struct GhKey {
    key: Vec<u8>,
}

#[post("/add?<pr>", data = "<data>")]
async fn add<'a>(
    db: Database,
    gh_key: State<'a, GhKey>,
    pr: Option<String>,
    data: Json<NewLog>,
) -> Result<Json<AddResponse>, Json<AddResponse>> {
    let mut data = data.0;
    data.probe = data.probe.to_ascii_lowercase();
    data.chip = data.chip.to_ascii_lowercase();
    data.os = data.os.to_ascii_lowercase();
    data.kind = data.kind.to_ascii_lowercase();
    data.protocol = data.protocol.to_ascii_lowercase();

    let log = db
        .run(move |c| {
            diesel::insert_into(logs::table)
                .values(&data)
                .execute(c)
                .and_then(|_| logs::table.order(logs::id.desc()).first::<Log>(c))
                .map_err(|e| {
                    Json(AddResponse {
                        error: Some(format!("{:?}", e)),
                        log: None,
                    })
                })
        })
        .await?;

    if let Some(pr) = pr {
        if let Ok(pr) = pr.parse::<u64>() {
            let _comment = create_gh_comment(&gh_key.key, pr, &log)
                .await
                .map_err(|e| {
                    Json(AddResponse {
                        error: Some(format!("{:?}", e)),
                        log: None,
                    })
                })?;
        }
    }

    Ok(Json(AddResponse {
        error: None,
        log: Some(log),
    }))
}

struct Claims {
    iat: u64,
    exp: u64,
    iss: u64,
}

pub async fn renew_token(private_key: &[u8]) -> octocrab::Result<Arc<Octocrab>> {
    use jwt::{algorithm::openssl::PKeyWithDigest, SignWithKey};
    use std::time::{SystemTime, UNIX_EPOCH};
    let key = PKeyWithDigest {
        key: openssl::pkey::PKey::private_key_from_pem(private_key).unwrap(),
        digest: MessageDigest::sha256(),
    };

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut claims = BTreeMap::new();
    claims.insert("iat", since_the_epoch);
    claims.insert("exp", since_the_epoch + 10 * 60);
    claims.insert("iss", APP_ID);

    let token_str = claims.sign_with_key(&key).unwrap();

    octocrab::initialise(octocrab::Octocrab::builder().personal_token(token_str.into()))?;

    let response: Value = octocrab::instance()
        .post(
            format!("/app/installations/{}/access_tokens", INSTALLATION_ID),
            None::<&()>,
        )
        .await?;
    let token = response["token"].as_str().unwrap();

    octocrab::initialise(octocrab::Octocrab::builder().personal_token(token.into()))
}

async fn create_gh_comment(
    private_key: &[u8],
    pr: u64,
    log: &Log,
) -> octocrab::Result<octocrab::models::issues::Comment> {
    let body = format!(
        r#"
**Ran performance benchmarks**
Commit: {}
Probe: {}
Chip: {}
Kind: {}
Read: {}
Write: {}
    "#,
        log.commit_hash, log.probe, log.chip, log.kind, log.read_speed, log.write_speed
    );

    renew_token(private_key).await?;

    octocrab::instance()
        .issues(GH_ORG, GH_REPO)
        .create_comment(pr, body)
        .await
}

async fn get_pr_commits(private_key: &[u8]) -> octocrab::Result<HashMap<u64, Vec<String>>> {
    renew_token(private_key).await?;

    let prs = octocrab::instance()
        .pulls(GH_ORG, GH_REPO)
        .list()
        .state(octocrab::params::State::Open)
        .send()
        .await?;

    let mut result = HashMap::<u64, Vec<String>>::new();

    for pr in prs {
        let commits: Value = octocrab::instance()
            .get::<_, _, Value>(pr.commits_url.unwrap().clone(), None)
            .await?;

        result.insert(
            pr.number,
            commits
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.get("sha").unwrap().as_str().unwrap().to_string())
                .collect(),
        );
    }

    Ok(result)
}

#[database("database")]
struct Database(diesel::SqliteConnection);
