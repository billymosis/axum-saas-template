mod config;
mod http;
use clap::Parser;
use config::Config;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let config = Config::parse();

    if !Sqlite::database_exists(&config.database_url)
        .await
        .unwrap_or(false)
    {
        println!("Creating database {}", &config.database_url);
        match Sqlite::create_database(&config.database_url).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }
    let db = SqlitePool::connect(&config.database_url).await?;

    sqlx::migrate!().run(&db).await?;

    http::serve(config, db).await?;

    Ok(())
}
