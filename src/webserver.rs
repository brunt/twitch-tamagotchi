use crate::commands::PetCommand;
use crate::tamagotchi::Tamagotchi;
use axum::extract::State;
use axum::http::{HeaderValue, Method, StatusCode};
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
use tower_http::cors::CorsLayer;
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
    let default_port = std::env::var("PORT")?; //_or("9091".to_string());

    let state = AppState::new(tamagotchi);

    let app = Router::new()
        .route("/action", post(action))
        .route("/sse", get(sse_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(format!("http://localhost:{default_port}").parse::<HeaderValue>()?)
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
                dbg!("{:?}", tamagotchi.to_string());
            }
            PetCommand::Clean => {
                tamagotchi.clean();
                dbg!("{:?}", tamagotchi.to_string());
            }
            PetCommand::Play => {
                tamagotchi.play();
                dbg!("{:?}", tamagotchi.to_string());
            }
            PetCommand::Sleep => {
                tamagotchi.sleep();
                dbg!("{:?}", tamagotchi.to_string());
            }
        }
        StatusCode::OK.into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

// prob want a channel, polling is messy
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // let tamagotchi = state.tamagotchi.clone();
    //
    // let interval = interval(Duration::from_secs(5));
    // let stream = IntervalStream::new(interval)
    //     .map(move |_| {
    //     let state_json = {
    //
    //         // have to convert to json manually because axum Json() sets content type which we don't want in sse
    //         serde_json::to_string(&tamagotchi).unwrap()
    //     };
    //     Ok(Event::default().data(state_json))
    // });
    //
    // Sse::new(stream).keep_alive(
    //     axum::response::sse::KeepAlive::new()
    //         .interval(Duration::from_secs(10))
    //         .text("keep-alive-text"),
    // )
    let interval = tokio::time::interval(Duration::from_secs(5));
    let tamagotchi = state.tamagotchi.clone();

    let stream = IntervalStream::new(interval).then(move |_| {
        let tamagotchi = tamagotchi.clone();
        async move {
            let tamagotchi = tamagotchi.lock().unwrap(); // or .await if using TokioMutex
            let state_json = serde_json::to_string(&*tamagotchi).unwrap();
            Ok(Event::default().data(state_json))
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive-text"),
    )
}
