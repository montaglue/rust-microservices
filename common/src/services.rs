use once_cell::sync::Lazy;

pub static AUTH: Lazy<String> =
    Lazy::new(|| std::env::var("AUTH_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()));
