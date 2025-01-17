use axum::debug_handler;
use axum_github_hooks::GithubWebhook;
use chrono::Utc;
use octocrab::models::webhook_events::{
    payload::WorkflowRunWebhookEventAction, WebhookEventPayload,
};

use crate::{
    functions::create_run::{ensure_run, get_run, CreateRunModel},
    github::read_env_var,
    views::run::get_benchmark_measurements,
    AppError,
};

use super::create_gh_comment;

#[debug_handler]
pub async fn webhook(GithubWebhook(hook): GithubWebhook) -> Result<(), AppError> {
    // println!("{hook:#?}");
    let octocrab = super::octocrab().await?;

    let workflow_path = read_env_var("GITHUB_BENCHMARK_WORKFLOW_PATH");
    if let WebhookEventPayload::WorkflowRun(body) = hook.specific {
        if body.workflow_run["path"] == workflow_path {
            match body.action {
                WorkflowRunWebhookEventAction::InProgress
                    if body.workflow_run["event"] == "pull_request" =>
                {
                    let pr = &body.workflow_run["pull_requests"][0];
                    let commit = pr["head"]["sha"].as_str().unwrap().into();
                    let parent = pr["base"]["sha"].as_str().unwrap();
                    let run_id = body.workflow_run["id"].as_i64().unwrap();
                    let author = body.workflow_run["actor"]["login"].as_str().unwrap().into();
                    let pr = pr["number"].as_i64().unwrap();
                    let previous = get_run(parent.into()).await?.map(|c| c.id);

                    ensure_run(
                        run_id,
                        CreateRunModel {
                            datetime: Utc::now().into(),
                            author,
                            commit,
                            pr,
                            previous,
                            measurements: vec![],
                        },
                    )
                    .await?;
                }
                WorkflowRunWebhookEventAction::Completed
                    if body.workflow_run["event"] == "pull_request" =>
                {
                    let pr = &body.workflow_run["pull_requests"][0];
                    let run_id = body.workflow_run["id"].as_i64().unwrap();
                    let pr = pr["number"].as_i64().unwrap();

                    let (_run, benchmarks) = get_benchmark_measurements(run_id).await.unwrap();
                    create_gh_comment(octocrab, pr, run_id, benchmarks)
                        .await
                        .unwrap();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
