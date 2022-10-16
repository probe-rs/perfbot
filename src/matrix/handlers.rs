use std::sync::Arc;

use askama::Template;
use diesel::{dsl::sql, sql_types::Text, QueryDsl, RunQueryDsl, TextExpressionMethods};
use matrix_sdk::{
    room::Joined,
    ruma::{
        api::client::r0::message::send_message_event::Response,
        events::{room::message::MessageEventContent, AnyMessageEventContent},
    },
    Error,
};
use perfbot_common::{schema::logs, Log};

use crate::get_pr_commits;

use super::templates::{HelpTemplate, PerfTemplate};

pub async fn help(room: Joined) -> Result<Response, Error> {
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
        "",
        HelpTemplate {}.render().unwrap(),
    ));

    room
        // send our message to the room we found the "!party" command in
        // the last parameter is an optional Uuid which we don't care about.
        .send(content, None)
        .await
}

pub async fn perf(
    private_key: Arc<Vec<u8>>,
    database_path: Arc<String>,
    room: Joined,
) -> Result<Response, Error> {
    // TODO:
    //
    // 1. Get open PRs.
    let prs = get_pr_commits(&private_key).await.unwrap();
    // 2. Get perf benchmarks for last commit of open PRs.
    let prs = prs
        .iter()
        .map(|(pr, commits)| Pr {
            number: *pr,
            benchmarks: logs::table
                .filter(
                    sql::<Text>(&format!("'{}'", commits.last().unwrap()))
                        .like(logs::commit_hash.concat("%")),
                )
                .load::<Log>(&crate::db::establish_connection(&database_path))
                .unwrap()
                .len(),
        })
        .collect::<Vec<Pr>>();
    // 3. Count number of benchmarks per PR.
    // 4. Print to commandline.

    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
        "",
        PerfTemplate { prs }.render().unwrap(),
    ));

    // send our message to the room we found the "!party" command in
    // the last parameter is an optional Uuid which we don't care about.
    room.send(content, None).await
}

pub struct Pr {
    pub number: u64,
    pub benchmarks: usize,
}
