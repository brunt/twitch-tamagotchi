use crate::commands::PetCommand;
use crate::parser::get_command;
use crate::tamagotchi::{Health, Tamagotchi};
use crate::webserver::start_web_server;
use dotenv::dotenv;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

mod commands;
mod parser;
mod tamagotchi;
mod webserver;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let tamagotchi = Arc::new(Mutex::new(Tamagotchi::new("Bernard".to_string())));
    let (tx, rx) = mpsc::channel::<PetCommand>(100);

    _ = tokio::spawn(read_commands_from_chat(tx));
    _ = tokio::spawn(start_web_server(Arc::clone(&tamagotchi)));

    start_game_loop(Arc::clone(&tamagotchi), rx).await;
    Ok(())
}

async fn read_commands_from_chat(tx: mpsc::Sender<PetCommand>) {
    let channel = format!(
        "#{}",
        std::env::var("CHANNEL_NAME").expect("missing CHANNEL_NAME env var")
    );

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
                            let _ = tx.send(command).await;
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
    mut rx: mpsc::Receiver<PetCommand>,
) {
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if let Ok(mut tamagotchi) = tamagotchi.lock() {
                    match msg {
                        PetCommand::Kill => tamagotchi.kill(),
                        PetCommand::New(name) => {
                            if matches!(tamagotchi.health, Health::Dead) {
                                *tamagotchi = Tamagotchi::new(name);
                            }
                        }
                        command => {
                            tamagotchi.add_command(command);
                        }
                    }
                    tamagotchi.tick();
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                if let Ok(mut t) = tamagotchi.lock() {
                    t.tick();
                }
            }
        }
    }
}
