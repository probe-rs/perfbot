use std::sync::Arc;

use askama::Template;
use diesel::{dsl::sql, sql_types::Text, QueryDsl, RunQueryDsl, TextExpressionMethods};
use matrix_sdk::{
    api::r0::message::send_message_event::Response,
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    locks::RwLock,
    Client, Error, Room,
};
use perfbot_common::{schema::logs, Log};

use crate::get_pr_commits;

use super::templates::{HelpTemplate, PerfTemplate};

pub async fn help(client: Client, room: Arc<RwLock<Room>>) -> Result<Response, Error> {
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
        "",
        HelpTemplate {}.render().unwrap(),
    ));
    // we clone here to hold the lock for as little time as possible.
    let room_id = room.read().await.room_id.clone();

    client
        // send our message to the room we found the "!party" command in
        // the last parameter is an optional Uuid which we don't care about.
        .room_send(&room_id, content, None)
        .await
}

pub async fn perf(client: Client, room: Arc<RwLock<Room>>) -> Result<Response, Error> {
    // TODO:
    //
    // 1. Get open PRs.
    let prs = get_pr_commits().await.unwrap();
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
                .load::<Log>(&crate::db::establish_connection())
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
    // we clone here to hold the lock for as little time as possible.
    let room_id = room.read().await.room_id.clone();

    client
        // send our message to the room we found the "!party" command in
        // the last parameter is an optional Uuid which we don't care about.
        .room_send(&room_id, content, None)
        .await
}

pub struct Pr {
    pub number: u64,
    pub benchmarks: usize,
}
