use axum::extract::State;
use axum::Json;
use mongodb::{bson::Bson, results::InsertOneResult, Database};
use serde::{Deserialize, Serialize};

use crate::utils;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigurationItems {
    key: String,
    value: String,  //json string
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
    permissions: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    _id: String,
    create_at: String,
    #[serde(flatten)]
    user_base: UserBase,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreationDB {
    create_at: String,
    #[serde(flatten)]
    user_creation: UserCreation
}
impl From<UserCreation>  for UserCreationDB {
    fn from(value: UserCreation) -> Self {
        let mut u = UserCreationDB{
            create_at: utils::now(),
            user_creation:value
        };

        u.user_creation.password=utils::encrypt(&*u.user_creation.password).unwrap();
        u
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreation {
    password: String,
    #[serde(flatten)]
    user_base: UserBase,
}

const COLLECTION : &str = "user";

pub async fn create_user(db: State<Database>, Json(payload): Json<UserCreation>) -> Result<String, String>{
    let c = db.collection(COLLECTION);
    let ud: UserCreationDB = payload.into();
    c.insert_one(ud, None).await.map(|r|match r.inserted_id {
        Bson::ObjectId(id)=> id.to_string(),
        _ => "".to_owned()
    }).map_err(|e|e.to_string())
}


#[cfg(test)]
mod test{
    use chrono::Local;

    use super::*;

    #[test]
    fn struct_test(){
        let user_base = UserBase{
            name:"zhangsang".to_string(),
            phone: "12344".to_string(),
            roles: vec![],
            permissions:vec![]
        };
        let user_profile = UserProfile {
            _id: "asdfbasfalsjdf".to_string(),
            user_base: user_base.clone(),
            create_at: Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
        };

        let str = serde_json::to_string(&user_profile).unwrap();
        println!("ss, {}",str);

        let ss =r#"{"_id":"asdfbasfalsjdf","name":"zhangsang","create_at":"2024-02-19T14:47:44.984Z","phone":"12344","roles":[],"permissions":[]}"#;

        let user: UserProfile = serde_json::from_str(ss).unwrap();
        assert_eq!(user.user_base.name, "zhangsang");
    
    }
   
}
