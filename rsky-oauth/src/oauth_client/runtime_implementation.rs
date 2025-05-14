use crate::jwk::Key;

pub type RuntimeKeyFactory = fn(Vec<String>) -> dyn Key;

pub type RuntimeRandomValues = fn(i64) -> Vec<u8>;

pub enum DigestAlgorithm {
    SHA256,
    SHA384,
    SHA512,
}
pub type RuntimeDigest = fn(Vec<u8>, DigestAlgorithm) -> Vec<u8>;

pub type RuntimeLock<T> = fn(&str, fn() -> T) -> T;
