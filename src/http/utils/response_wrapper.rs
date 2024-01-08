use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct JsonData<T>(pub T, pub Option<String>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonDataResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    api_version: Option<String>,
    data: T,
}

impl<T> IntoResponse for JsonData<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let x = Json(JsonDataResponse {
            data: self.0,
            api_version: self.1,
        });
        x.into_response()
    }
}
