#![recursion_limit = "512"]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod matrix;
mod schema;

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use std::{env, sync::Mutex};

use askama::Template;
use chrono::NaiveDateTime;
use diesel::{
    debug_query, dsl::sql, sqlite::Sqlite, Connection, ExpressionMethods, Insertable, QueryDsl,
    QueryResult, Queryable, RunQueryDsl, SqliteConnection, TextExpressionMethods,
};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use matrix_bot_api::{
    handlers::{HandleResult, StatelessHandler},
    MatrixBot, MessageType,
};
use octocrab::Octocrab;
use openssl::hash::MessageDigest;
use rocket::{self, get, post, routes};
use rocket_contrib::{
    database, json::Json, serve::StaticFiles, templates::Template as RocketTemplate,
};
use schema::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::{Handle, Runtime};

const GH_ORG: &str = "probe-rs";
const GH_REPO: &str = "probe-rs";
const APP_ID: u64 = 93972;
const INSTALLATION_ID: u64 = 13730372;

lazy_static::lazy_static! {
    static ref RUNTIME: Mutex<Option<Handle>> = Mutex::new(None);
}

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

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
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

pub fn matrix_bot() {
    let mut handler = StatelessHandler::new();

    handler.register_handle("perf", |bot, message, tail| {
        // TODO:
        //
        // 1. Get open PRs.
        let mut handle = { RUNTIME.lock().unwrap().clone() };
        let prs = futures::executor::block_on(
            handle
                .as_mut()
                .unwrap()
                .spawn(async { get_pr_commits().await }),
        )
        .unwrap()
        .unwrap();
        // 2. Get perf benchmarks for last commit of open PRs.
        let prs = prs
            .iter()
            .map(|(pr, commits)| Pr {
                number: *pr,
                benchmarks: logs::table
                    .filter(
                        sql(&format!("'{}'", commits.last().unwrap()))
                            .like(logs::commit_hash.concat("%")),
                    )
                    .load::<Log>(&establish_connection())
                    .unwrap()
                    .len(),
            })
            .collect::<Vec<Pr>>();
        // 3. Count number of benchmarks per PR.
        // 4. Print to commandline.

        bot.send_html_message(
            &"",
            &PerfTemplate { prs }.render().unwrap(),
            &message.room,
            MessageType::TextMessage,
        );
        HandleResult::StopHandling
    });

    handler.register_handle("help", |bot, message, tail| {
        bot.send_html_message(
            "",
            &HelpTemplate {}.render().unwrap(),
            &message.room,
            MessageType::TextMessage,
        );
        HandleResult::StopHandling
    });

    let bot = MatrixBot::new(handler);
    std::thread::spawn(|| bot.run("perfbot", "k@Jr1ZrlhLKi", "https://matrix.org"));
}

fn main() {
    migrate();

    let mut rt = Runtime::new().unwrap();
    {
        let mut handle = RUNTIME.lock().unwrap();
        *handle = Some(rt.handle().clone());
    }
    rt.spawn(rocket());
    let bot = matrix_bot();
    rt.block_on(std::future::pending::<()>());
}

fn migrate() {
    let connection = establish_connection();

    // This will run the necessary migrations.
    embedded_migrations::run(&connection).unwrap();

    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
}

async fn rocket() -> Result<(), rocket::error::Error> {
    rocket::ignite()
        .mount("/", routes![index, list, add])
        .attach(Database::fairing())
        .attach(RocketTemplate::fairing())
        .mount("/static", StaticFiles::from("./static"))
        .launch()
        .await
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

#[post("/add?<pr>", data = "<data>")]
async fn add(
    db: Database,
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
            let _comment = create_gh_comment(pr, &log).await.map_err(|e| {
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

pub async fn renew_token() -> octocrab::Result<Arc<Octocrab>> {
    let key = include_bytes!("../probe-rs-perfbot.2020-12-25.private-key.pem");

    use jwt::{algorithm::openssl::PKeyWithDigest, SignWithKey};
    use std::time::{SystemTime, UNIX_EPOCH};
    let key = PKeyWithDigest {
        key: openssl::pkey::PKey::private_key_from_pem(key).unwrap(),
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

    renew_token().await?;

    octocrab::instance()
        .issues(GH_ORG, GH_REPO)
        .create_comment(pr, body)
        .await
}

async fn get_pr_commits() -> octocrab::Result<HashMap<u64, Vec<String>>> {
    renew_token().await?;

    let prs = octocrab::instance()
        .pulls(GH_ORG, GH_REPO)
        .list()
        .state(octocrab::params::State::Open)
        .send()
        .await?;

    let mut result = HashMap::<u64, Vec<String>>::new();

    for pr in prs {
        let commits: Value = octocrab::instance()
            .get::<_, _, Value>(pr.commits_url.clone(), None)
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

#[derive(Serialize, Deserialize, Queryable)]
pub struct Log {
    pub id: i32,
    pub probe: String,
    pub chip: String,
    pub os: String,
    pub protocol: String,
    pub protocol_speed: i32,
    pub commit_hash: String,
    #[serde(with = "timestamp")]
    pub timestamp: NaiveDateTime,
    pub kind: String,
    pub read_speed: i32,
    pub write_speed: i32,
}

mod timestamp {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(date.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = i64::deserialize(deserializer)?;
        Ok(NaiveDateTime::from_timestamp(s, 0))
    }
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "logs"]
pub struct NewLog {
    pub probe: String,
    pub chip: String,
    pub os: String,
    pub protocol: String,
    pub protocol_speed: i32,
    pub commit_hash: String,
    #[serde(with = "timestamp")]
    pub timestamp: NaiveDateTime,
    pub kind: String,
    pub read_speed: i32,
    pub write_speed: i32,
}

#[database("database")]
struct Database(diesel::SqliteConnection);
