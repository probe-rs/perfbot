mod handlers;
mod templates;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use matrix_sdk::{
    self,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        SyncMessageEvent,
    },
    Client, ClientConfig, EventEmitter, JsonStore, LoopCtrl, SyncRoom, SyncSettings,
};
use url::Url;

struct CommandBot {
    /// This clone of the `Client` will send requests to the server,
    /// while the other keeps us in sync with the server using `sync`.
    client: Client,
}

impl CommandBot {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl EventEmitter for CommandBot {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            let msg_body = if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                ..
            } = event
            {
                msg_body.clone()
            } else {
                String::new()
            };

            if msg_body.starts_with("!help") {
                handlers::help(self.client.clone(), room).await.unwrap();
            } else if msg_body.starts_with("!perf") {
                handlers::perf(self.client.clone(), room).await.unwrap();
            }
        }
    }
}

pub async fn login_and_sync(
    homeserver_url: &str,
    username: &str,
    password: &str,
    shutdown: Arc<Mutex<bool>>,
) -> Result<(), matrix_sdk::Error> {
    // the location for `JsonStore` to save files to
    let mut home = dirs::home_dir().expect("no home directory found");
    home.push("perfbot");

    let store = JsonStore::open(&home)?;
    let client_config = ClientConfig::new().state_store(Box::new(store));

    let homeserver_url = Url::parse(&homeserver_url).expect("Couldn't parse the homeserver URL");
    let mut client = Client::new_with_config(homeserver_url, client_config).unwrap();

    client
        .login(&username, &password, None, Some("perfbot"))
        .await?;

    println!("logged in as {}", username);

    // An initial sync to set up state and so our bot doesn't respond to old messages.
    // If the `StateStore` finds saved state in the location given the initial sync will
    // be skipped in favor of loading state from the store
    client.sync_once(SyncSettings::default()).await.unwrap();
    // add our CommandBot to be notified of incoming messages, we do this after the initial
    // sync to avoid responding to messages before the bot was running.
    client
        .add_event_emitter(Box::new(CommandBot::new(client.clone())))
        .await;

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default()
        .token(client.sync_token().await.unwrap())
        .timeout(std::time::Duration::from_millis(1000));
    // this keeps state from the server streaming in to CommandBot via the EventEmitter trait
    // client.sync(settings).await;
    client
        .sync_with_callback(settings, |_| async {
            println!("*KEKKE");
            if *shutdown.lock().unwrap() {
                println!("BREAKBREAK");
                LoopCtrl::Break
            } else {
                LoopCtrl::Continue
            }
        })
        .await;
    println!("DUNDID");

    Ok(())
}
