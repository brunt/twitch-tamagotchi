use crate::commands::PetCommand;
use crate::tamagotchi::{Health, Tamagotchi};
use axum::extract::State;
use axum::http::{Method, StatusCode, Uri, header};
use axum::response::sse::Event;
use axum::response::{Html, IntoResponse, Response, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::IntervalStream;
use tower_http::cors::{Any, CorsLayer};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

static INDEX_HTML: &str = "index.html";

#[derive(Clone)]
pub struct AppState {
    pub tamagotchi: Arc<Mutex<Tamagotchi>>,
    pub notify: Arc<Notify>,
}

impl AppState {
    pub fn new(tamagotchi: Arc<Mutex<Tamagotchi>>, notify: Arc<Notify>) -> AppState {
        Self { tamagotchi, notify }
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

pub async fn start_web_server(
    tamagotchi: Arc<Mutex<Tamagotchi>>,
    notify: Arc<Notify>,
) -> anyhow::Result<()> {
    let default_port = std::env::var("PORT")?;
    let state = AppState::new(tamagotchi, notify);

    let app = Router::new()
        .route("/action", post(action))
        .route("/sse", get(sse_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST]),
        )
        .fallback(static_handler)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{default_port}")).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn action(State(state): State<AppState>, Json(req): Json<ActionRequest>) -> Response {
    if let Ok(mut tamagotchi) = state.tamagotchi.lock() {
        match req.action {
            PetCommand::Kill => {
                tamagotchi.kill();
            }
            PetCommand::New(name) => {
                if matches!(tamagotchi.health, Health::Dead) {
                    *tamagotchi = Tamagotchi::new(name);
                }
            }
            command => {
                tamagotchi.add_command(command);
                if tamagotchi.is_idle() {
                    state.notify.notify_one();
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

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return not_found().await;
            }

            index_html().await
        }
    }
}

async fn index_html() -> Response {
    match Assets::get(INDEX_HTML) {
        Some(content) => Html(content.data).into_response(),
        None => not_found().await,
    }
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "404").into_response()
}
