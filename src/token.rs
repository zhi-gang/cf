
use jsonwebtoken::{decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Serialize, Deserialize};

use jsonwebtoken::errors::ErrorKind;

// Example user profile struct
#[derive(Debug, Serialize, Deserialize,PartialEq)]
struct UserProfile {
    username: String,
    roles: Vec<String>,
    permission: Vec<String>,
    exp:usize
}

// Secret key for encoding and decoding JWTs
const SECRET_KEY: &[u8] = b"secret";

// Generate JWT token based on user profile
fn generate_token(user_profile: &UserProfile) -> Result<String> {
    encode(
        &Header::default(),
        user_profile,
        &EncodingKey::from_secret(SECRET_KEY),
    )
}

// Verify JWT token
fn verify_token(token: &str) -> Result<UserProfile> {
    decode::<UserProfile>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|e| e.into())
} 

#[cfg(test)]
mod test{
    use jsonwebtoken::decode_header;
    use chrono;
    use super::*;

    #[test]
    fn token_test(){
        let user_profile = {
            UserProfile {
                username: "User1".to_string(),
                roles : vec!["admin".to_string(), "super".to_string()],
                permission: vec!["read".to_string(), "write".to_string()],
                exp: usize::try_from(chrono::offset::Utc::now().timestamp() +3600).unwrap()
            }
        };

        let user_profile2 = {
            UserProfile {
                username: "User1".to_string(),
                roles : vec!["admin".to_string(), "super".to_string()],
                permission: vec!["read".to_string(), "write".to_string()],
                exp: usize::try_from(chrono::offset::Utc::now().timestamp() - 3600).unwrap()
            }
        };

        let token = generate_token(&user_profile).unwrap();
        println!("token: {}",token);
 
        let header = decode_header(&token).unwrap();
        println!("header: {:?}",header);
        
        let user = verify_token(&*token).unwrap();
        assert_eq!(user,user_profile);

        let token2 = generate_token(&user_profile2).unwrap();
        println!("token: {}",token);

        if let Err(e) = verify_token(&*token2){
            println!("error:{:?}", e.to_string());
            match e.kind() {
                ErrorKind::ExpiredSignature=> println!("expire"),
                _ => println!("others"),
            }
        }else{
            assert!(false);
        }

        if let Err(e) = verify_token("123"){
            println!("error:{:?}", e.to_string());
            assert_eq!(e.kind(), &ErrorKind::InvalidToken);
            match e.kind() {
                ErrorKind::InvalidToken=> println!("invalid"),
                _ => println!("others"),
            }
        }else{
            assert!(false);
        }
        
 
 
    }
}
