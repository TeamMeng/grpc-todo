mod config;
mod model;
mod server;

use anyhow::Result;
use sqlx::{Executor, PgPool};
use sqlx_db_tester::TestPg;
use std::path::Path;

pub use config::*;
pub use model::*;
pub use server::run_server;

pub async fn db_init() -> Result<PgPool> {
    let config = Config::new()?;
    let pool = PgPool::connect(&config.database_url).await?;
    Ok(pool)
}

pub async fn db_new_for_test() -> Result<(TestPg, PgPool)> {
    let config = Config::new()?;

    let post = config
        .database_url
        .rfind('/')
        .expect("Database url should invalid");

    let database_url = &config.database_url[..post];
    let tdb = TestPg::new(database_url.to_string(), Path::new(".././migrations"));
    let pool = tdb.get_pool().await;

    let sql = include_str!("../../fixtures/test.sql").split(';');
    let mut ts = pool.begin().await.expect("begin transaction failed");
    for s in sql {
        if s.trim().is_empty() {
            continue;
        }
        ts.execute(s).await.expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");

    Ok((tdb, pool))
}
