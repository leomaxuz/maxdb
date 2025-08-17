use std::collections::HashMap;
use std::fs::File;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use openssl::pkcs5::pbkdf2_hmac;
use openssl::hash::MessageDigest;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub salt_hex: String,
    pub hash_hex: String,
    pub iters: u32,
}

#[derive(Clone)]  // Clone trait qo‘shildi
pub struct AuthDB {
    users: HashMap<String, User>,
}

impl AuthDB {
    pub fn new(path: &str) -> Self {
        let f = File::open(path).unwrap();
        let cfg: Value = serde_json::from_reader(f).unwrap();
        let mut users = HashMap::new();
        for u in cfg["users"].as_array().unwrap() {
            let user = User {
                username: u["username"].as_str().unwrap().to_string(),
                salt_hex: u["salt_hex"].as_str().unwrap().to_string(),
                hash_hex: u["hash_hex"].as_str().unwrap().to_string(),
                iters: u.get("iters").and_then(|v| v.as_u64()).unwrap_or(200_000) as u32,
            };
            users.insert(user.username.clone(), user);
        }
        AuthDB { users }
    }

    pub fn verify(&self, username: &str, password: &str) -> bool {
        if let Some(u) = self.users.get(username) {
            let salt = hex::decode(&u.salt_hex).unwrap();
            let mut hash = [0u8; 32];
            pbkdf2_hmac(
                password.as_bytes(),
                &salt,
                u.iters.try_into().unwrap(), // u32 → usize
                MessageDigest::sha256(),
                &mut hash,
            ).unwrap();
            hex::encode(hash) == u.hash_hex
        } else { 
            false 
        }
    }
}
