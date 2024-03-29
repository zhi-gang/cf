use crate::{permission_check, utils};
use axum::http::header::HeaderMap;
use axum::Json;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use mongodb::options::FindOptions;
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserBase {
    pub name: String,
    pub phone: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserProfile {
    pub _id: String,
    pub create_at: DateTime<Utc>,
    #[serde(flatten)]
    pub user_base: UserBase,
}

impl UserProfile {
    pub fn default_super() -> Self {
        let base = UserBase {
            name: "super".to_string(),
            phone: "111111".to_string(),
            roles: vec!["super".to_string()],
            permissions: vec![],
        };
        UserProfile {
            _id: "0".to_string(),
            create_at: Utc::now(),
            user_base: base,
        }
    }
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
    create_at: DateTime<Utc>,
    #[serde(flatten)]
    user_creation: UserCreation,
}
impl From<UserCreation> for UserCreationDB {
    fn from(value: UserCreation) -> Self {
        let mut u = UserCreationDB {
            create_at: Utc::now(),
            user_creation: value,
        };

        u.user_creation.password = utils::encrypt(&*u.user_creation.password).unwrap();
        u
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreation {
    password: String,
    #[serde(flatten)]
    user_base: UserBase,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInDB {
    pub _id: Bson,
    pub password: String,
    pub create_at: DateTime<Utc>,
    #[serde(flatten)]
    pub user_base: UserBase,
}

const COLLECTION: &str = "user";

pub async fn create_user(
    headers: HeaderMap, //the order is important!
    db: State<Database>,
    Json(payload): Json<UserCreation>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "create_user", &*payload.user_base.name)?;
    let c = db.collection(COLLECTION);
    let f = c
        .find_one(doc! {"name":&payload.user_base.name}, None)
        .await;
    if let Ok(Some(_)) = f {
        return Err((StatusCode::CONFLICT, "User Name esists".to_string()));
    }
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

pub async fn update_user(
    headers: HeaderMap,
    db: State<Database>,
    Json(payload): Json<UserProfile>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "update_user", &*payload.user_base.name)?;

    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let oid = build_obj_id(&*payload._id)?;

    let filter = doc! { "_id": Bson::ObjectId(oid) };
    let mut update_doc = bson::to_document(&payload).map_err(|e| {
        error!("build update doc faield: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    update_doc.remove("_id");
    let update = doc! {"$set": update_doc};

    c.update_one(filter, update, None)
        .await
        .map(|r| serde_json::to_string(&r).unwrap())
        .map_err(|e| {
            error!("update faield: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

pub async fn delete_user(
    headers: HeaderMap,
    db: State<Database>,
    Json(payload): Json<UserProfile>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "delete_user", &*payload.user_base.name)?;

    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let oid = build_obj_id(&*payload._id)?;

    let filter = doc! { "_id": Bson::ObjectId(oid) };

    c.delete_one(filter, None)
        .await
        .map(|r| serde_json::to_string(&r).unwrap())
        .map_err(|e| {
            error!("delete faield: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

fn user_prfile_after_find(res: Option<UserInDB>) -> Result<String, (StatusCode, String)> {
    if let Some(user_in_db) = res {
        let user_profile = UserProfile::from(user_in_db);
        Ok(serde_json::to_string(&user_profile).unwrap())
    } else {
        Err((StatusCode::NOT_FOUND, "Not Found".to_string()))
    }
}

pub async fn find_user_by_id(
    headers: HeaderMap,
    Path(user_id): Path<String>,
    db: State<Database>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "find_user_by_id", &*user_id)?;
    let oid = build_obj_id(&*user_id)?;
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let f = c
        .find_one(doc! {"_id":Bson::ObjectId(oid)}, None)
        .await
        .map_err(|e| {
            error!("find user failed, {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    user_prfile_after_find(f)
}

pub async fn find_user_by_name(
    headers: HeaderMap,
    Path(user_name): Path<String>,
    db: State<Database>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "find_user_by_name", &*user_name)?;
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let f = c
        .find_one(doc! {"name":user_name}, None)
        .await
        .map_err(|e| {
            error!("find user failed, {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    user_prfile_after_find(f)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumberOfUsers {
    total: i32,
}

pub async fn get_number_of_all_users(
    headers: HeaderMap,
    db: State<Database>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "find_user_by_name", "")?;
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let mut cursor = c
        .aggregate(vec![doc! {"$count":"total"}], None)
        .await
        .map_err(|e| {
            error!("get_number_of_all_users failed, {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let num = cursor
        .try_next()
        .await
        .map(|res| match res {
            Some(doc) => doc.get_i32("total").unwrap_or(-1),
            None => -1,
        })
        .map_err(|e| {
            error!("cursor browse error {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(
        serde_json::to_string(&NumberOfUsers { total: num }).map_err(|e| {
            error!("serilizer error {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?,
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryUserListOptions {
    limit: i64,
    skip: u64,
    sort_by_name: i8, //-1: desc, -1: asc
}

pub async fn get_user_in_page(
    headers: HeaderMap,
    db: State<Database>,
    Json(payload): Json<QueryUserListOptions>,
) -> Result<String, (StatusCode, String)> {
    permission_check(&headers, "get_user_in_page", "")?;
    let c: Collection<UserInDB> = db.collection(COLLECTION);
    let skip = Some(payload.skip);
    let limit = Some(payload.limit);
    let options_builder = FindOptions::builder().skip(skip).limit(limit);
    let options_builder2 = match payload.sort_by_name {
        -1 => options_builder.sort(doc! {"name":-1}),
        1 => options_builder.sort(doc! {"name":1}),
        _ => options_builder.sort(doc! {}),
    };
    let options = options_builder2.build();
    let mut cursor = c.find(doc! {}, options).await.map_err(|e| {
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

fn build_obj_id(id: &str) -> Result<ObjectId, (StatusCode, String)> {
    let oid = oid::ObjectId::parse_str(id).map_err(|e| {
        error!("parse id failed , {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(oid)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;

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
            create_at: Utc::now(),
        };

        let str = serde_json::to_string(&user_profile).unwrap();
        println!("ss, {}", str);

        let ss = r#"{"_id":"asdfbasfalsjdf","name":"zhangsang","create_at":"2024-02-20T13:47:49.756108+08:00","phone":"12344","roles":[],"permissions":[]}"#;

        let user: UserProfile = serde_json::from_str(ss).unwrap();
        assert_eq!(user.user_base.name, "zhangsang");
    }
}
