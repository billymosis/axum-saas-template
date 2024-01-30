mod controllers;
mod database;
mod error;
mod middleware;
mod models;
mod services;
mod utils;

pub use error::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;

use crate::config::Config;
use anyhow::Context;
use axum::Extension;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use sqlx::SqlitePool;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};

use self::controllers::auth::auth_routes;
use self::database::DB;
use self::middleware::middleware::{auth_middleware, AuthContext};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DB,
    pub reqwest: reqwest::Client,
}

pub async fn serve(config: Config, db: SqlitePool) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .build()
        .expect("Failed to create reqwest client");

    let app_state = AppState {
        config: Arc::new(config),
        db: DB::new(db),
        reqwest: client,
    };

    let app = api_router(app_state);
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 1234));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("error running HTTP server")
}

fn api_router(app_state: AppState) -> Router {
    Router::new()
        .route("/protected", get(protected))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .nest("/api", Router::new().nest("/auth", auth_routes(app_state)))
        .route("/", get(|| async { Html("<div>Hello</div>") }))
        .layer((
            CompressionLayer::new(),
            TraceLayer::new_for_http().on_failure(()),
            TimeoutLayer::new(Duration::from_secs(30)),
            CatchPanicLayer::new(),
        ))
        .layer(CookieManagerLayer::new())
}

async fn protected(Extension(context): Extension<AuthContext>) -> Result<impl IntoResponse> {
    println!("{:?}", context);
    Ok(Html(format!("<h1>Authenticated {}</h1>", context.user_id)))
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
