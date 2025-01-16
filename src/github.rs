pub mod webhook;

use std::sync::Arc;

use anyhow::{Ok, Result};
use itertools::Itertools;
use octocrab::Octocrab;

use crate::{components::benchmark_list_entry::Benchmark, helpers::read_env_var};

pub async fn renew_token() -> Result<Arc<Octocrab>> {
    let app_id = read_env_var("GITHUB_APP_ID").parse::<u64>().unwrap().into();

    let app_private_key = read_env_var("GITHUB_APP_PRIVATE_KEY");
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(app_private_key.as_bytes()).unwrap();

    let octocrab = Octocrab::builder().app(app_id, key).build()?;
    let octocrab = octocrab::initialise(octocrab);

    Ok(octocrab)
}

pub async fn octocrab() -> Result<Arc<Octocrab>> {
    let app_installation_id = read_env_var("GITHUB_APP_INSTALLATION_ID")
        .parse::<u64>()
        .unwrap()
        .into();
    let (octocrab, _) = octocrab::instance()
        .installation_and_token(app_installation_id)
        .await?;
    Ok(Arc::new(octocrab))
}

async fn create_gh_comment(
    octocrab: Arc<Octocrab>,
    pr: i64,
    run_id: i64,
    benchmarks: Vec<Benchmark>,
) -> Result<octocrab::models::issues::Comment> {
    let results = benchmarks
        .iter()
        .map(|benchmark| {
            let status_text = benchmark.status_text();
            let percent_change = benchmark.percent_change_text();
            let name = &benchmark.name;
            let abs = benchmark.abs;
            let unit = &benchmark.unit;
            let diff = benchmark.diff;
            format!("| {name}  | {abs}{unit} ({diff}{unit})  | {percent_change} | {status_text} |")
        })
        .join("\n");

    let public_url = read_env_var("PUBLIC_URL");

    let results = if results.is_empty() {
        "No benchmarks were found. Check your CI workflows.".into()
    } else {
        format!(
            r#"
| Name          | Change (abs)  | Change (%)    | Outcome       |
| ------------- | ------------- | ------------- | ------------- |
{results}
        "#
        )
    };

    let body = format!(
        r#"# Benchmarks

{public_url}/run/{run_id}

{results}
    "#
    );

    let org = read_env_var("GITHUB_ORG");
    let repo = read_env_var("GITHUB_REPO");
    Ok(octocrab
        .issues(org, repo)
        .create_comment(pr as u64, body)
        .await?)
}
