use bcrypt::{hash, verify, DEFAULT_COST};

pub fn encrypt(pwd: &str) -> anyhow::Result<String> {
    let encrypted = hash(pwd, DEFAULT_COST)?;
    Ok(encrypted)
}

pub fn valid(pwd: &str, encrypted: &str) -> anyhow::Result<bool> {
    let valid = verify(pwd, encrypted)?;
    Ok(valid)
}

// pub fn now() -> String {
//     // Local::now().to_rfc3339()
//     Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
//     // Local::now().format(Rfc3339).to_string()
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_crypt() {
        let hashed = encrypt("hunter2").unwrap();
        println!("hashed :{}", hashed);
        let valid = valid("hunter2", &hashed).unwrap();
        assert!(valid);
    }
}
