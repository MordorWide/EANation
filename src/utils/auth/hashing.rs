use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

// String hashing (mostly for passwords)
pub fn plain_string_to_hash(plain_password: &String) -> String {
    let config = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = config
        .hash_password(plain_password.as_bytes(), &salt)
        .unwrap();

    // Add argon2 prefix to be compatible with Django's password format
    format!("argon2{}", hash.to_string()).to_string()
}

// String validation (mostly for passwords)
pub fn verify_plain_string_for_hash(password: &String, hashdata: &String) -> bool {
    // Remove argon2 prefix first (due to Django's password format)
    let sliced_hashdata = &hashdata[6..];

    let config = Argon2::default();
    let parsed_hash = PasswordHash::new(&sliced_hashdata).unwrap();
    config
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
