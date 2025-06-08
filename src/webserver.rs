use std::cmp::PartialEq;
use crate::commands::PetCommand;
use crate::tamagotchi::{Health, Tamagotchi};
use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::sse::Event;
use axum::response::{IntoResponse, Response, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::IntervalStream;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

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
    let default_port = std::env::var("PORT")?;
    let state = AppState::new(tamagotchi);

    let app = Router::new()
        .route("/action", post(action))
        .route("/sse", get(sse_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST]),
        )
        .fallback_service(ServeDir::new("assets"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{default_port}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn action(State(state): State<AppState>, Json(req): Json<ActionRequest>) -> Response {
    if let Ok(mut tamagotchi) = state.tamagotchi.lock() {
        match req.action {
            PetCommand::Feed => {
                tamagotchi.feed();
            },
            PetCommand::Clean => {
                tamagotchi.clean();
            },
            PetCommand::Play => {
                tamagotchi.play();
            },
            PetCommand::Sleep => {
                tamagotchi.sleep();
            },
            PetCommand::Kill => {
                tamagotchi.kill();
            },
            PetCommand::New(name) => {
                if matches!(tamagotchi.health, Health::Dead) {
                    *tamagotchi = Tamagotchi::new(name);
                }
            }
        }
        StatusCode::OK.into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(Duration::from_secs(2));

    let stream = IntervalStream::new(interval).then(move |_| {
        let tamagotchi = state.tamagotchi.clone();
        async move {
            if let Ok(tamagotchi) = tamagotchi.lock() {
                if let Ok(state_json) = serde_json::to_string(&*tamagotchi) {
                    return Ok(Event::default().data(state_json));
                }
            }
            Ok(Event::default().data("{}"))
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}
