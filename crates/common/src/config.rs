//! Fail-fast runtime config (W1). All secrets validated at boot —
//! no silent empty-secret fallbacks.
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub jwt_secret: Vec<u8>,
    pub jwt_issuer: String,
    pub jwt_ttl_secs: u64,
    pub database_url_main: String,
    pub database_url_hospital: String,
    pub redis_url: String,
    pub rabbitmq_url: String,
    pub mongodb_url: String,
    pub ollama_url: String,
    pub paystack_secret: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing env {0}")]
    Missing(&'static str),
    #[error("JWT_SECRET must be >= 32 bytes, got {0}")]
    JwtTooShort(usize),
    #[error("AES256_CLINIC_KEY_B64 must decode to exactly 32 bytes")]
    BadClinicKey,
}

fn req(key: &'static str) -> Result<String, ConfigError> {
    std::env::var(key)
        .map_err(|_| ConfigError::Missing(key))
        .and_then(|v| {
            if v.trim().is_empty() {
                Err(ConfigError::Missing(key))
            } else {
                Ok(v)
            }
        })
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv(); // optional .env
        let jwt_secret = req("JWT_SECRET")?.into_bytes();
        if jwt_secret.len() < 32 {
            return Err(ConfigError::JwtTooShort(jwt_secret.len()));
        }
        // Validate clinic key early even though hospital-service owns it.
        let clinic_b64 = req("AES256_CLINIC_KEY_B64")?;
        let raw = B64
            .decode(clinic_b64.trim())
            .map_err(|_| ConfigError::BadClinicKey)?;
        if raw.len() != 32 {
            return Err(ConfigError::BadClinicKey);
        }
        Ok(Self {
            jwt_secret,
            jwt_issuer: std::env::var("JWT_ISSUER").unwrap_or_else(|_| "newgate-erp".into()),
            jwt_ttl_secs: std::env::var("JWT_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            database_url_main: req("DATABASE_URL_MAIN")?,
            database_url_hospital: req("DATABASE_URL_HOSPITAL")?,
            redis_url: req("REDIS_URL")?,
            rabbitmq_url: req("RABBITMQ_URL")?,
            mongodb_url: std::env::var("MONGODB_URL")
                .unwrap_or_else(|_| "mongodb://mongo:27017/exams".into()),
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://ollama:11434".into()),
            paystack_secret: req("PAYSTACK_SECRET_KEY")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_short_jwt_and_bad_clinic_key() {
        let key = B64.encode([1u8; 32]);
        std::env::set_var("AES256_CLINIC_KEY_B64", &key);
        std::env::set_var("JWT_SECRET", "short");
        std::env::set_var("DATABASE_URL_MAIN", "postgres://x");
        std::env::set_var("DATABASE_URL_HOSPITAL", "postgres://y");
        std::env::set_var("REDIS_URL", "redis://x");
        std::env::set_var("RABBITMQ_URL", "amqp://x");
        std::env::set_var("PAYSTACK_SECRET_KEY", "k");
        assert!(matches!(
            Config::from_env(),
            Err(ConfigError::JwtTooShort(_))
        ));
        std::env::set_var("JWT_SECRET", "this-is-a-very-long-secret-over-32-bytes!");
        std::env::set_var("AES256_CLINIC_KEY_B64", "!!!not-base64!!!");
        assert!(matches!(Config::from_env(), Err(ConfigError::BadClinicKey)));
    }
}
