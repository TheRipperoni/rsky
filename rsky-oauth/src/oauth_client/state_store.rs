use crate::jwk::Key;
use crate::simple_store::SimpleStore;

pub struct InternalStateData {
    pub iss: String,
    pub dpop_key: dyn Key,
    pub verifier: Option<String>,
    pub app_state: Option<String>,
}

pub type StateStore = dyn SimpleStore<String, InternalStateData, Error = ()>;
