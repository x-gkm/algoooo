use std::env::{self, VarError};

use sqlx::{PgPool, postgres::PgPoolOptions};

mod fetcher;
mod ranking;
mod web;

pub use {fetcher::fetch, ranking::rate, web::serve};

#[derive(Debug, thiserror::Error)]
enum InitError {
    #[error(transparent)]
    Env(#[from] VarError),

    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

async fn init() -> Result<PgPool, InitError> {
    let _ = dotenvy::dotenv();

    let db_url = env::var("DATABASE_URL")?;

    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?)
}

pub async fn migrate() -> anyhow::Result<()> {
    let db = init().await?;
    sqlx::migrate!().run(&db).await?;
    Ok(())
}
