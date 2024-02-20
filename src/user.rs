use crate::utils;
use axum::Json;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid, Bson},
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigurationItems {
    key: String,
    value: String, //json string
}
// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct ConfigValue {
//    host:String,
//    port:u32
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBase {
    name: String,
    phone: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    _id: String,
    create_at: String,
    #[serde(flatten)]
    user_base: UserBase,
}

fn pick_id(oid: Bson) -> Option<String> {
    match oid {
        Bson::ObjectId(oid) => Some(oid.to_string()),
        _ => None,
    }
}

impl From<UserInDB> for UserProfile {
    fn from(value: UserInDB) -> Self {
        let sid = pick_id(value._id).map_or("".to_string(), |v| v);

        UserProfile {
            _id: sid,
            create_at: value.create_at,
            user_base: value.user_base,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreationDB {
    create_at: String,
    #[serde(flatten)]
    user_creation: UserCreation,
}
impl From<UserCreation> for UserCreationDB {
    fn from(value: UserCreation) -> Self {
        let mut u = UserCreationDB {
            create_at: utils::now(),
            user_creation: value,
        };

        u.user_creation.password = utils::encrypt(&*u.user_creation.password).unwrap();
        u
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct UserInDB {
//     _id: String,
//     create_at: String,
//     #[serde(flatten)]
//     user_creation: UserCreation,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreation {
    password: String,
    #[serde(flatten)]
    user_base: UserBase,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInDB {
    _id: Bson,
    password: String,
    create_at: String,
    #[serde(flatten)]
    user_base: UserBase,
}

const COLLECTION: &str = "user";

pub async fn create_user(
    db: State<Database>,
    Json(payload): Json<UserCreation>,
) -> Result<String, (StatusCode, String)> {
    let c = db.collection(COLLECTION);
    let ud: UserCreationDB = payload.into();
    c.insert_one(ud, None)
        .await
        .map(|r| match r.inserted_id {
            Bson::ObjectId(id) => id.to_string(),
            _ => "".to_owned(),
        })
        .map_err(|e| {
            error!("creat user faield: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

pub async fn find_user_by_id(
    db: State<Database>,
    Path(user_id): Path<String>,
) -> Result<String, (StatusCode, String)> {
    let oid = oid::ObjectId::parse_str(&*user_id).map_err(|e| {
        error!("parse id failed , {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let f = c
        .find_one(doc! {"_id":Bson::ObjectId(oid)}, None)
        .await
        .map_err(|e| {
            error!("find user failed, {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    if let Some(user_in_db) = f {
        let user_profile = UserProfile::from(user_in_db);
        Ok(serde_json::to_string(&user_profile).unwrap())
    } else {
        Err((StatusCode::NOT_FOUND, "Not Found".to_string()))
    }
}

pub async fn find_user_by_name(
    db: State<Database>,
    Path(user_name): Path<String>,
) -> Result<String, (StatusCode, String)> {
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let mut cursor = c.find(doc! {"name":user_name}, None).await.map_err(|e| {
        error!("get cursor failed, {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    let mut users = Vec::<UserProfile>::new();
    while let Some(user_in_db) = cursor.try_next().await.map_err(|e| {
        error!("cursor browse error {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })? {
        let user_profile = UserProfile::from(user_in_db);
        users.push(user_profile);
    }
    info!("users : {:?}", users);

    Ok(serde_json::to_string(&users).unwrap())
}

#[cfg(test)]
mod test {
    use chrono::Local;

    use super::*;

    #[test]
    fn struct_test() {
        let user_base = UserBase {
            name: "zhangsang".to_string(),
            phone: "12344".to_string(),
            roles: vec![],
            permissions: vec![],
        };
        let user_profile = UserProfile {
            _id: "asdfbasfalsjdf".to_string(),
            user_base: user_base.clone(),
            create_at: Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        };

        let str = serde_json::to_string(&user_profile).unwrap();
        println!("ss, {}", str);

        let ss = r#"{"_id":"asdfbasfalsjdf","name":"zhangsang","create_at":"2024-02-19T14:47:44.984Z","phone":"12344","roles":[],"permissions":[]}"#;

        let user: UserProfile = serde_json::from_str(ss).unwrap();
        assert_eq!(user.user_base.name, "zhangsang");
    }
}
