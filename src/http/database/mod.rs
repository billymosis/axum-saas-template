pub mod user;
pub mod session;

#[derive(Clone)]
pub struct DB {
    pub db: sqlx::PgPool,
}

impl DB {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { db: pool }
    }
}
