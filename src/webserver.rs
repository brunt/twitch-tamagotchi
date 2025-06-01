use crate::commands::PetCommand;
use crate::tamagotchi::Tamagotchi;
use axum::extract::State;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub tamagotchi: Arc<Mutex<Tamagotchi>>,
}

impl AppState {
    pub fn new(tamagotchi: Arc<Mutex<Tamagotchi>>) -> AppState {
        Self { tamagotchi }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ActionRequest {
    pub action: PetCommand,
}

impl From<PetCommand> for ActionRequest {
    fn from(cmd: PetCommand) -> Self {
        Self { action: cmd }
    }
}

pub async fn start_web_server(tamagotchi: Arc<Mutex<Tamagotchi>>) -> anyhow::Result<()> {
    //TODO: read from .env properly
    let default_port = std::env::var("PORT").unwrap_or("9091".to_string());

    let state = AppState::new(tamagotchi);
    let app = Router::new()
        .route("/action", post(action))
        // .route("/status", get(status))
        // .route("/dist/{*file}", get(static_handler))
        // .route("/", get(index))
        .layer(
            CorsLayer::new()
                .allow_origin(
                    format!("http://localhost:{default_port}")
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_methods([Method::GET, Method::POST]),
        )
        // .fallback_service(get(not_found))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{default_port}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn action(State(state): State<AppState>, Json(req): Json<ActionRequest>) -> Response {
    if let Ok(mut tamagotchi) = state.tamagotchi.lock() {
        match req.action {
            PetCommand::Feed => {
                dbg!("feed happened");
                tamagotchi.feed(20);
            }
            PetCommand::Clean => {
                tamagotchi.clean(20);
            }
            PetCommand::Play => {
                tamagotchi.play();
            }
            _ => unimplemented!(),
        }
        StatusCode::OK.into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
