
use std::sync::Arc;

use axum::{
    extract::{
        State,
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};

use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

pub struct AppState {
    // Add your application components here.
}

type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "MetalHead=debug,tower_http=info".into()),
        )
        .json()
        .init();

    let state = Arc::new(RwLock::new(AppState {}));

    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/stream", get(stream_ws))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::env::var("METALHEAD_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into());

    info!("MetalHead API server on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "MetalHead API server is running")
}

async fn stream_ws(
    ws: WebSocketUpgrade,
    State(_state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(message)) = socket.recv().await {
        match message {
            Message::Text(text) => {
                info!("Received WebSocket message: {text}");

                if socket
                    .send(Message::Text(format!("Echo: {text}").into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }

            Message::Close(_) => break,

            _ => {}
        }
    }
}
