mod config;
mod http;
use anyhow::Context;
use clap::Parser;
use config::Config;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let config = Config::parse();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!().run(&db).await?;

    http::serve(config, db).await?;

    Ok(())
}
