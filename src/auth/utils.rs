use bcrypt::{hash, verify, DEFAULT_COST};

pub fn encrypt_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Error Hashing Password")
}

pub fn decrypt_password(password: &str, encrypted_pass: &str) -> bool {
    verify(password, encrypted_pass).unwrap_or(false)
}
