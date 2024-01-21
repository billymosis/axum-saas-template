use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailResponse {
    pub data: Vec<DataItem>,
    pub message: String,
    pub request_id: String,
    pub object: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataItem {
    pub code: String,
    pub additional_info: Vec<String>, // Assuming additional_info is an array of strings, you can adjust this based on the actual data type
    pub message: String,
}
