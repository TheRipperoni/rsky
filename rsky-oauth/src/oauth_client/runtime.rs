use crate::jwk::Key;
use std::sync::Mutex;

pub struct Runtime {
    has_implementation_lock: bool,
}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_key(&self) -> Key {
        unimplemented!()
    }

    pub async fn sha256(&self, text: &str) -> String {
        unimplemented!()
    }

    pub async fn generate_nonce(&self, length: Option<usize>) -> String {
        unimplemented!()
    }

    pub async fn generate_pckce(&self, byte_length: Option<usize>) -> String {
        unimplemented!()
    }

    pub async fn calculate_jwk_thumbprint() {
        unimplemented!()
    }

    /**
     * @see {@link https://datatracker.ietf.org/doc/html/rfc7636#section-4.1}
     * @note It is RECOMMENDED that the output of a suitable random number generator
     * be used to create a 32-octet sequence. The octet sequence is then
     * base64url-encoded to produce a 43-octet URL safe string to use as the code
     * verifier.
     */
    async fn generate_verifier(&self, byte_length: Option<usize>) -> String {
        unimplemented!()
    }
}
