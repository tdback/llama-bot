use anyhow;
use matrix_sdk::{
    config::SyncSettings,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, str::FromStr};

// Could probably have this be a parameter instead of a constant, but I
// honestly would only have this support the handful of models I have
// installed anyways.
const SUPPORTED_MODELS: [&str; 3] = ["deepseek-r1", "mistral", "llama3.2"];
const TRIGGER: &str = "!llama";
const USAGE: &str = r#"Usage: !llama <COMMAND>

Commands:
  ask <MODEL> <PROMPT>   Send <PROMPT> to <MODEL>. See `list` for a listing of supported models.
  list                   List supported models.
  help                   Display this help message.
"#;

#[derive(Debug)]
enum Command {
    Ask,
    Help,
    List,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Command, String> {
        match s {
            "ask" => Ok(Command::Ask),
            "help" => Ok(Command::Help),
            "list" => Ok(Command::List),
            _ => Err(format!(
                "{} is not a valid command. Run `!llama help` for a list of valid commands.",
                s
            )),
        }
    }
}

impl Command {
    // TODO: Clean this up!
    async fn run(&self, api: &str, data: Option<&str>) -> anyhow::Result<RoomMessageEventContent> {
        let msg = match *self {
            Command::Ask => match data {
                // Ensure we can split at least once. If we can, assign the
                // first as the intended model to run and the rest to
                // the prompt.
                Some(d) => match d.split_once(char::is_whitespace) {
                    Some((model, prompt)) => {
                        if SUPPORTED_MODELS.contains(&model) {
                            let payload = HashMap::from([("model", &model), ("prompt", &prompt)]);
                            &query_api(&api, &payload).await?
                        } else {
                            &format!("{} is an unsupported model. Run `!llama list` for a list of supported models.", &model)
                        }
                    }
                    None => &format!("Please provide a model and a prompt."),
                },
                None => &format!("Please provide a model and a prompt."),
            },
            Command::List => &format!("Supported models: {}", SUPPORTED_MODELS.join(", ")),
            Command::Help => USAGE,
        };

        Ok(RoomMessageEventContent::text_plain(&*msg))
    }
}

pub async fn login_and_sync(
    homeserver_url: &str,
    username: &str,
    password: &str,
    api: String,
) -> anyhow::Result<()> {
    let bot = matrix_sdk::Client::builder()
        .homeserver_url(homeserver_url)
        .build()
        .await?;

    bot.matrix_auth()
        .login_username(&username, &password)
        .initial_device_display_name("llama-bot")
        .await?;

    println!("bot logged in as {}", username);

    // Use a closure to pass the client into the event handler.
    let response = bot.sync_once(SyncSettings::default()).await?;
    bot.add_event_handler(move |ev, room| on_room_message(ev, room, api));

    // Pass a sync token to keep state from the server streaming in via the
    // `EventHandler` trait.
    let settings = SyncSettings::default().token(response.next_batch);
    bot.sync(settings).await?;

    Ok(())
}

async fn on_room_message(
    event: OriginalSyncRoomMessageEvent,
    room: matrix_sdk::Room,
    api: String,
) -> anyhow::Result<()> {
    if room.state() != matrix_sdk::RoomState::Joined {
        return Ok(());
    }

    let MessageType::Text(text_content) = event.content.msgtype else {
        return Ok(());
    };

    if !text_content.body.starts_with(TRIGGER) {
        return Ok(());
    }

    // TODO: Although it always will start with the trigger, we should still
    // handle the strip more carefully to prevent panics.
    let raw_content = text_content
        .body
        .as_str()
        .strip_prefix(TRIGGER)
        .unwrap()
        .trim();

    let (command, data) = match raw_content.split_once(char::is_whitespace) {
        Some((c, d)) => (c, Some(d)),
        None => (raw_content.trim_start(), None),
    };

    let message = match command.parse::<Command>() {
        Ok(cmd) => cmd.run(&api, data).await?,
        Err(e) => RoomMessageEventContent::text_plain(e),
    };

    room.send(message).await?;

    Ok(())
}

// For desiralizing responses from the LLM.
#[derive(Debug, Deserialize, Serialize)]
struct ModelMessage {
    model: String,
    created_at: String,
    response: String,
    done: bool,
}

async fn query_api(api: &str, payload: &HashMap<&str, &&str>) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let stream = client.post(api).json(&payload).send().await?.text().await?;

    Ok(stream
        .split('\n')
        .filter_map(|json| serde_json::from_str::<ModelMessage>(&json).ok())
        .filter(|msg| !msg.done)
        .map(|msg| msg.response)
        .collect::<Vec<_>>()
        .join("")
        .trim_start()
        .to_string())
}
