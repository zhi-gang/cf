use crate::user::{UserInDB, UserProfile};
use crate::{token, utils};
use axum::Json;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use tracing::error;

const COLLECTION: &str = "user";

#[derive(Debug, Serialize, Deserialize)]
pub struct Authentication {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    profile: UserProfile,
    token: String,
}

pub async fn authenticate(
    db: State<Database>,
    Json(payload): Json<Authentication>,
) -> Result<String, (StatusCode, String)> {
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let f = c.find_one(doc! {"name":&payload.name}, None).await;
    if let Ok(Some(user_in_db)) = f {
        let password_encrypted = &user_in_db.password;
        match utils::valid(&payload.password, password_encrypted) {
            Ok(v) => {
                if v {
                    let user_profile = UserProfile::from(user_in_db);
                    let jwt = token::generate_token(&user_profile, 14400).map_err(|e| {
                        error!("Failed to generate token {:?}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    })?;
                    let auth_res = AuthenticationResponse {
                        profile: user_profile,
                        token: jwt,
                    };
                    Ok(serde_json::to_string(&auth_res).unwrap())
                } else {
                    Err((StatusCode::FORBIDDEN, "Invalid Password".to_string()))
                }
            }
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        }
    } else {
        Err((StatusCode::CONFLICT, "User Not Found".to_string()))
    }
}
