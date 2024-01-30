pub mod user;
pub mod session;

#[derive(Clone)]
pub struct DB {
    pub db: sqlx::SqlitePool,
}

impl DB {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { db: pool }
    }
}
