use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyData {
    settings: String,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct SessionModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub data: sqlx::types::Json<MyData>,
    pub expiry_date: OffsetDateTime,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
}
