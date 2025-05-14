use crate::oauth_provider::constants::DPOP_NONCE_MAX_AGE;
use crate::oauth_provider::lib::current_epoch;
use hex::ToHex;
use hmac::{Hmac, KeyInit, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::SystemTime;

#[derive(Clone)]
pub struct DpopNonce {
    secret: Vec<u8>,
    counter: u64,
    prev: String,
    now: String,
    next: String,
    step: u64,
}

#[derive(Clone)]
pub enum DpopNonceInput {
    String(String),
    Uint8Array(Vec<u8>),
    DpopNonce(DpopNonce),
}

impl DpopNonce {
    pub fn new(secret: Vec<u8>, step: u64) -> Result<Self, DpopNonceError> {
        if secret.len() != 32 {
            return Err(DpopNonceError::InvalidRequestParams(
                "Expected 32 bytes".to_string(),
            ));
        }
        if step > DPOP_NONCE_MAX_AGE as u64 / 3 {
            return Err(DpopNonceError::InvalidRequestParams(
                "Invalid step".to_string(),
            ));
        }

        let current_time = current_epoch();
        let counter = current_time / step;

        let mut dpop_nonce = Self {
            secret,
            counter,
            prev: "".to_string(),
            now: "".to_string(),
            next: "".to_string(),
            step,
        };
        dpop_nonce.prev = dpop_nonce.compute(counter - 1);
        dpop_nonce.now = dpop_nonce.compute(counter);
        dpop_nonce.next = dpop_nonce.compute(counter + 1);
        Ok(dpop_nonce)
    }

    pub fn next_nonce(&mut self) -> String {
        self.rotate();
        self.next.clone()
    }

    fn rotate(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("timestamp in micros since UNIX epoch")
            .as_millis() as u64;
        let counter = now / self.step;
        match counter - self.counter {
            0 => return,
            1 => {
                self.prev = self.now.clone();
                self.now = self.next.clone();
                self.next = self.compute(counter + 1);
            }
            2 => {
                self.prev = self.next.clone();
                self.now = self.compute(counter);
                self.next = self.compute(counter + 1);
            }
            _ => {
                self.prev = self.compute(counter - 1);
                self.now = self.compute(counter);
                self.next = self.compute(counter + 1)
            }
        }
        self.counter = counter;
    }

    pub fn check(&self, nonce: &str) -> bool {
        self.next == nonce || self.now == nonce || self.prev == nonce
    }

    pub fn from(
        input: Option<DpopNonceInput>,
        _step: Option<u64>,
    ) -> Result<DpopNonce, DpopNonceError> {
        let step = _step.unwrap_or(DPOP_NONCE_MAX_AGE as u64 / 3);
        match input {
            None => {
                let mut rng = rand::rng();
                let mut random_bytes = [0u8; 32];
                rng.fill(&mut random_bytes);
                let secret = random_bytes.to_vec();
                DpopNonce::new(secret, step)
            }
            Some(dpop_nonce_input) => match dpop_nonce_input {
                DpopNonceInput::String(res) => {
                    let secret = hex::decode(res).expect("Decoding failed");
                    DpopNonce::new(secret, step)
                }
                DpopNonceInput::Uint8Array(secret) => DpopNonce::new(secret, step),
                DpopNonceInput::DpopNonce(res) => Ok(res),
            },
        }
    }

    fn compute(&self, counter: u64) -> String {
        let counter_bytes = num_to_64_bits(counter);
        let mut mac =
            Hmac::<Sha256>::new_from_slice(&self.secret).expect("HMAC can take key of any size");
        mac.update(&counter_bytes);
        let result = mac.finalize().into_bytes();
        result.encode_hex()
    }
}

fn num_to_64_bits(num: u64) -> [u8; 8] {
    let b1: u8 = ((num >> 56) & 0xff) as u8;
    let b2: u8 = ((num >> 48) & 0xff) as u8;
    let b3: u8 = ((num >> 40) & 0xff) as u8;
    let b4: u8 = ((num >> 32) & 0xff) as u8;
    let b5: u8 = ((num >> 24) & 0xff) as u8;
    let b6: u8 = ((num >> 16) & 0xff) as u8;
    let b7: u8 = ((num >> 8) & 0xff) as u8;
    let b8: u8 = (num & 0xff) as u8;
    [b1, b2, b3, b4, b5, b6, b7, b8]
}

/// Errors that can occur when creating a DpopNonce
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DpopNonceError {
    InvalidRequestParams(String),
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::oauth_types::OAuthAccessToken;
//
//     #[test]
//     fn test_new_dpop_nonce() {
//         let dpop_nonce = DpopNonce::new("valid_token").unwrap();
//         assert_eq!(token.as_ref(), "valid_token");
//     }
//
//     #[test]
//     fn test_new_empty_token() {
//         assert!(matches!(
//             OAuthAccessToken::new(""),
//             Err(AccessTokenError::Empty)
//         ));
//     }
//
//     #[test]
//     fn test_display() {
//         let token = OAuthAccessToken::new("test_token").unwrap();
//         assert_eq!(token.to_string(), "test_token");
//     }
//
//     #[test]
//     fn test_into_inner() {
//         let token = OAuthAccessToken::new("test_token").unwrap();
//         assert_eq!(token.into_inner(), "test_token");
//     }
//
//     #[test]
//     fn test_as_ref() {
//         let token = OAuthAccessToken::new("test_token").unwrap();
//         assert_eq!(token.as_ref(), "test_token");
//     }
// }
