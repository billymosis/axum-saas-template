use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_verification_token(length: usize) -> String {
    let token = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();

    token
}
