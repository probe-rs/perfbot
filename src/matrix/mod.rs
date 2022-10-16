mod handlers;
mod templates;

use std::sync::{Arc, Mutex};

use matrix_sdk::{
    self,
    room::Room,
    ruma::events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        SyncMessageEvent,
    },
    Client, ClientConfig, LoopCtrl, SyncSettings,
};
use url::Url;

async fn on_room_message(
    event: SyncMessageEvent<MessageEventContent>,
    room: Room,
    gh_key: Arc<Vec<u8>>,
    database_path: Arc<String>,
) {
    if let Room::Joined(room) = room {
        let msg_body = if let SyncMessageEvent {
            content:
                MessageEventContent {
                    msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                    ..
                },
            ..
        } = event
        {
            msg_body.clone()
        } else {
            String::new()
        };

        if msg_body.starts_with("!help") {
            handlers::help(room).await.unwrap();
        } else if msg_body.starts_with("!perf") {
            handlers::perf(gh_key, database_path, room).await.unwrap();
        }
    }
}

pub async fn login_and_sync(
    homeserver_url: &str,
    username: &str,
    password: &str,
    gh_key: Vec<u8>,
    database_path: String,
    json_store: &str,
    shutdown: Arc<Mutex<bool>>,
) -> Result<(), matrix_sdk::Error> {
    let client_config = ClientConfig::new().store_path(json_store);

    let homeserver_url = Url::parse(&homeserver_url).expect("Couldn't parse the homeserver URL");
    let client = Client::new_with_config(homeserver_url, client_config).unwrap();

    client
        .login(&username, &password, None, Some("perfbot"))
        .await?;

    println!("logged in as {}", username);

    // An initial sync to set up state and so our bot doesn't respond to old messages.
    // If the `StateStore` finds saved state in the location given the initial sync will
    // be skipped in favor of loading state from the store
    client.sync_once(SyncSettings::default()).await.unwrap();

    let gh_key = Arc::new(gh_key);
    let database_path = Arc::new(database_path);

    // add our CommandBot to be notified of incoming messages, we do this after the initial
    // sync to avoid responding to messages before the bot was running.
    client
        .register_event_handler(move |ev, room| {
            on_room_message(ev, room, gh_key.clone(), database_path.clone())
        })
        .await;

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default()
        .token(client.sync_token().await.unwrap())
        .timeout(std::time::Duration::from_millis(500));
    // this keeps state from the server streaming in to CommandBot via the EventEmitter trait
    // client.sync(settings).await;
    client
        .sync_with_callback(settings, |_| async {
            if *shutdown.lock().unwrap() {
                LoopCtrl::Break
            } else {
                LoopCtrl::Continue
            }
        })
        .await;

    Ok(())
}
