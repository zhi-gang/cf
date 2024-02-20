use chrono;
use jsonwebtoken::{
    decode, encode, errors::Result, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use super::user::UserProfile;


/// Struct for build JWT token
#[derive(Serialize, Deserialize)]
struct UserProfileEx {
    profile: UserProfile,
    exp: i64,
}
impl UserProfileEx {
    pub fn from_profile(user_profile: UserProfile, expire_in: i64) -> Self {
        UserProfileEx {
            profile: user_profile,
            exp: chrono::offset::Utc::now().timestamp() + expire_in,
        }
    }
    pub fn to_profile(&self) -> UserProfile {
        self.profile.clone()
    }
}

// Secret key for encoding and decoding JWTs
const SECRET_KEY: &[u8] = b"secret";

// Generate JWT token based on user profile
pub fn generate_token(user_profile: &UserProfile, expire_in: i64) -> Result<String> {
    let user_profile_ex = UserProfileEx::from_profile(user_profile.clone(), expire_in);
    encode(
        &Header::default(),
        &user_profile_ex,
        &EncodingKey::from_secret(SECRET_KEY),
    )
}

// Verify JWT token
pub fn verify_token(token: &str) -> Result<UserProfile> {
    let user_profile_ex = decode::<UserProfileEx>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)?;

    Ok(user_profile_ex.to_profile())
}

#[cfg(test)]
mod test {
    use crate::user::UserBase;

    use super::*;
    use chrono::Utc;
    use jsonwebtoken::{decode_header, errors::ErrorKind};
    // Example user profile struct

    #[test]
    fn token_test() {
        let user = UserBase {
            name: "User1".to_string(),
            phone: "12123".to_string(),
            roles: vec!["admin".to_string(), "super".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
        };
        let user_profile = UserProfile {
            _id: "122333".to_string(),
            create_at: Utc::now(),
            user_base: user,
        };

        let token = generate_token(&user_profile, 3600).unwrap();
        println!("token: {}", token);

        let header = decode_header(&token).unwrap();
        println!("header: {:?}", header);

        let user = verify_token(&*token).unwrap();
        assert_eq!(user._id, user_profile._id);

        let token2 = generate_token(&user_profile, -100).unwrap();
        println!("token: {}", token);

        if let Err(e) = verify_token(&*token2) {
            println!("error:{:?}", e.to_string());
            match e.kind() {
                ErrorKind::ExpiredSignature => println!("expire"),
                _ => println!("others"),
            }
        } else {
            assert!(false);
        }

        if let Err(e) = verify_token("123") {
            println!("error:{:?}", e.to_string());
            assert_eq!(e.kind(), &ErrorKind::InvalidToken);
            match e.kind() {
                ErrorKind::InvalidToken => println!("invalid"),
                _ => println!("others"),
            }
        } else {
            assert!(false);
        }
    }
}
