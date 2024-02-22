use crate::permission_check;
use axum::http::header::HeaderMap;
use axum::{
    extract::State,
    http::StatusCode,
};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Bson},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use tracing::error;

const COLLECTION: &str = "data";

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigData {
    pub _id: Bson,
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserConfigDataResponse {
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

pub async fn get_user_cfg_data(
    headers: HeaderMap, //the order is important!
    db: State<Database>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "get_user_cfg_data", "")?;
    let c: Collection<UserConfigData> = db.collection(COLLECTION);

    let mut cursor = c.find(doc! {}, None).await.map_err(|e| {
        error!("get cursor failed, {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    let mut roles: Vec<String> = Vec::new();
    let mut permissions: Vec<String> = Vec::new();

    while let Some(data) = cursor.try_next().await.map_err(|e| {
        error!("cursor browse error {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })? {
        if data.key == "roles" {
            roles = data.values;
        } else if data.key == "permissions" {
            permissions = data.values;
        }
    }
    let res = UserConfigDataResponse { roles, permissions };
    Ok(serde_json::to_string(&res).unwrap())
}
