use crate::parser::get_command;
use crate::tamagotchi::Tamagotchi;
use crate::webserver::{ActionRequest, start_web_server};
use dotenv::dotenv;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

mod commands;
mod parser;
mod tamagotchi;
mod webserver;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let tamagotchi = Arc::new(Mutex::new(Tamagotchi::new("Bernard".to_string())));
    let notify = Arc::new(Notify::new());

    _ = tokio::spawn(read_commands_from_chat());
    _ = tokio::spawn(start_web_server(
        Arc::clone(&tamagotchi),
        Arc::clone(&notify),
    ));
    start_game_loop(Arc::clone(&tamagotchi), Arc::clone(&notify)).await?;
    Ok(())
}

async fn read_commands_from_chat() {
    let channel = format!(
        "#{}",
        std::env::var("CHANNEL_NAME").expect("missing CHANNEL_NAME env var")
    );

    let action_url = format!(
        "http://localhost:{}/action",
        std::env::var("PORT").unwrap_or("9091".to_string())
    );

    let http_client = reqwest::Client::new();
    let mut irc_client = tmi::Client::anonymous()
        .await
        .expect("failed to create client");
    irc_client
        .join(&channel)
        .await
        .expect("failed to join channel");

    loop {
        if let Ok(msg) = irc_client.recv().await {
            if let Ok(m) = msg.as_typed() {
                match m {
                    tmi::Message::Privmsg(msg) => {
                        if let Some(command) = get_command(&mut msg.text()) {
                            _ = http_client
                                .post(&action_url)
                                .json(&ActionRequest::from(command))
                                .send()
                                .await;
                        }
                    }
                    tmi::Message::Reconnect => {
                        _ = irc_client.reconnect().await;
                        _ = irc_client.join(&channel).await;
                    }
                    tmi::Message::Ping(ping) => {
                        _ = irc_client.pong(&ping).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn start_game_loop(
    tamagotchi: Arc<Mutex<Tamagotchi>>,
    notify: Arc<Notify>,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {}
            _ = notify.notified() => {}
        }
        if let Ok(mut t) = tamagotchi.lock() {
            t.tick();
        }
    }
}
