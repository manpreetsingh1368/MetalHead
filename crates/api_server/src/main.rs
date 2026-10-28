use axum::{
    extract::{Path, State, WebSocketUpgrade},
    extract::ws::{WebSocket, Message},
    routing::{get, post},
    Json, Router, response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use llm_core::{GenerateResponse, SamplingParams, FinishReason};
use sim_agent::{SimAgent, SimRequest};
use domain_physics::{ConstantsDB, ChemicalFormula};
use web_search::{SearchRouter, SearchConfig, SearchResponse};

pub struct AppState {
    pub sim_agent:     SimAgent,
    pub constants_db:  ConstantsDB,
    pub search_router: SearchRouter,
}

type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "physllm=debug,tower_http=info,web_search=debug".into())
        )
        .json()
        .init();

    let search_config = SearchConfig {
        brave_api_key:     std::env::var("BRAVE_API_KEY").ok(),
        serp_api_key:      std::env::var("SERP_API_KEY").ok(),
        nasa_ads_key:      std::env::var("NASA_ADS_KEY").ok(),
        max_results:       5,
        cache_ttl_s:       3600,
        request_timeout_s: 15,
        full_page_fetch:   std::env::var("FULL_PAGE_FETCH").is_ok(),
        max_page_chars:    8_000,
    };

    info!("Brave search: {}", search_config.brave_api_key.is_some());
    info!("NASA ADS:     {}", search_config.nasa_ads_key.is_some());

    let state = Arc::new(RwLock::new(AppState {
        sim_agent:     SimAgent::new(),
        constants_db:  ConstantsDB::built_in(),
        search_router: SearchRouter::new(search_config),
    }));

    let app = Router::new()
        .route("/v1/health",            get(health))
        .route("/v1/models",            get(list_models))
        .route("/v1/tools",             get(list_tools))
        .route("/v1/generate",          post(generate))
        .route("/v1/generate/search",   post(generate_with_search))
        .route("/v1/stream",            get(stream_ws))
        .route("/v1/simulate",          post(simulate))
        .route("/v1/search",            post(search))
        .route("/v1/search/arxiv",      post(search_arxiv))
        .route("/v1/fetch",             post(fetch_url_handler))
        .route("/v1/constants/:symbol", get(get_constant))
        .route("/v1/chemistry/mw",      post(molecular_weight))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::env::var("METALHEAD_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()); // use reverse proxy in production
    info!("MetalHead API server on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}