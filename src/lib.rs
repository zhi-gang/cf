use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use token::verify_token;
use tracing::{error, info};

pub mod auth;
pub mod config;
pub mod mongo_api;
pub mod token;
pub mod user;
pub mod utils;
pub mod user_config;

/// valid the auth token
/// if invalid return status code 401
fn permission_check(
    headers: &HeaderMap,
    fn_name: &str,
    arg: &str,
) -> Result<(), (StatusCode, String)> {
    if let Some(auth) = headers.get("authorization") {
        let token = auth.to_str().unwrap();
        match verify_token(token) {
            Ok(p) => {
                //TODO: permission check
                info!(
                    "{} invoke {} on {} at {}",
                    p.user_base.name,
                    fn_name,
                    arg,
                    Utc::now()
                );
                Ok(())
            }
            Err(e) => {
                error!("verify_token failed, {:?}", e);
                Err((StatusCode::UNAUTHORIZED, e.to_string()))
            }
        }
    } else {
        error!("missing authecication information");
        Err((
            StatusCode::UNAUTHORIZED,
            "missing authecication information".to_string(),
        ))
    }
}