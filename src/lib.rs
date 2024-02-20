pub mod config;
pub mod mongo_api;
pub mod utils;
pub mod token;
pub mod user;
pub mod auth;

use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigurationItems {
    key: String,
    value: String,  //json string
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct UserBase {
//     name: String,
//     phone: String,
//     #[serde(default)]
//     roles: Vec<String>,
//     #[serde(default)]
//     permissions: Vec<String>
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct UserProfile {
//     _id: String,
//     create_at: String,
//     #[serde(flatten)]
//     user_base: UserBase,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct UserCreation {
//     password: String,
//     #[serde(flatten)]
//     user_base: UserBase,
// }


// #[cfg(test)]
// mod test{
//     use chrono::Local;

//     use super::*;

//     #[test]
//     fn struct_test(){
//         let user_base = UserBase{
//             name:"zhangsang".to_string(),
//             phone: "12344".to_string(),
//             roles: vec![],
//             permissions:vec![]
//         };
//         let user_profile = UserProfile {
//             _id: "asdfbasfalsjdf".to_string(),
//             user_base: user_base.clone(),
//             create_at: Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
//         };

//         let str = serde_json::to_string(&user_profile).unwrap();
//         println!("ss, {}",str);

//         let ss =r#"{"_id":"asdfbasfalsjdf","name":"zhangsang","create_at":"2024-02-19T14:47:44.984Z","phone":"12344","roles":[],"permissions":[]}"#;

//         let user: UserProfile = serde_json::from_str(ss).unwrap();
//         assert_eq!(user.user_base.name, "zhangsang");
    
//     }
   
// }
