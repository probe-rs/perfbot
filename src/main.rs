#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod schema;

use std::collections::BTreeMap;
use std::env;

use chrono::NaiveDateTime;
use diesel::{
    Connection, ExpressionMethods, Insertable, QueryDsl, QueryResult, Queryable, RunQueryDsl,
    SqliteConnection,
};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use openssl::hash::MessageDigest;
use rocket::*;
use rocket_contrib::{database, json::Json, serve::StaticFiles, templates::Template};
use schema::*;
use serde::{Deserialize, Serialize};

const GH_ORG: &str = "probe-rs";
const GH_REPO: &str = "probe-rs";
const APP_ID: u64 = 93972;
const INSTALLATION_ID: u64 = 13730372;

embed_migrations!();

#[get("/")]
fn index() -> Template {
    let context = BTreeMap::<String, String>::new();
    Template::render("index", &context)
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

#[launch]
fn rocket() -> rocket::Rocket {
    let connection = establish_connection();

    // This will run the necessary migrations.
    embedded_migrations::run(&connection).unwrap();

    // By default the output is thrown out. If you want to redirect it to stdout, you
    // should call embedded_migrations::run_with_output.
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();

    rocket::ignite()
        .mount("/", routes![index, list, add])
        .attach(Database::fairing())
        .attach(Template::fairing())
        .mount("/static", StaticFiles::from("./static"))
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

async fn create_gh_comment(
    pr: u64,
    log: &Log,
) -> octocrab::Result<octocrab::models::issues::Comment> {
    let body = format!(
        r#"
**Running performance benchmakrs:**
Commit: {}
Probe: {}
Chip: {}
    "#,
        log.commit_hash, log.probe, log.chip
    );

    let key = include_bytes!("../probe-rs-perfbot.2020-12-25.private-key.pem");

    use jwt::{algorithm::openssl::PKeyWithDigest, SignWithKey};
    use serde_json::Value;
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

    println!("{:?}", claims);

    let token_str = claims.sign_with_key(&key).unwrap();
    println!("{}", token_str);

    octocrab::initialise(octocrab::Octocrab::builder().personal_token(token_str.into())).unwrap();

    let response: Value = octocrab::instance()
        .post(
            format!("/app/installations/{}/access_tokens", INSTALLATION_ID),
            None::<&()>,
        )
        .await
        .unwrap();
    let token = response["token"].as_str().unwrap();

    octocrab::initialise(octocrab::Octocrab::builder().personal_token(token.into())).unwrap();

    octocrab::instance()
        .issues(GH_ORG, GH_REPO)
        .create_comment(pr, body)
        .await
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
