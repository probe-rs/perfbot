#[macro_use]
extern crate diesel;

mod schema;

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, Queryable, RunQueryDsl};
use openssl::hash::MessageDigest;
use rocket::*;
use rocket_contrib::{database, json::Json, serve::StaticFiles, templates::Template};
use schema::*;
use serde::{Deserialize, Serialize};

const GH_ORG: &str = "probe-rs";
const GH_REPO: &str = "probe-rs";
const APP_ID: u64 = 93972;
const INSTALLATION_ID: u64 = 13730372;

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

#[get("/list?<_probe>&<_chip>&<_os>&<_kind>&<_protocol>&<_protocol_speed>")]
async fn list(
    db: Database,
    _probe: Option<String>,
    _chip: Option<String>,
    _os: Option<String>,
    _kind: Option<String>,
    _protocol: Option<String>,
    _protocol_speed: Option<String>,
) -> Json<ListResponse> {
    Json(
        db.run(move |c| {
            logs::table
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

#[launch]
fn rocket() -> rocket::Rocket {
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
    let log = db
        .run(move |c| {
            diesel::insert_into(logs::table)
                .values(&data.0)
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
