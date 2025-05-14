use crate::jwk::{JwtHeader, JwtPayload, Key};
use crate::simple_store::SimpleStore;

pub struct DpopFetchWrapperOptions {
    pub key: dyn Key,
    pub iss: String,
    pub nonces: dyn SimpleStore<String, String, Error = ()>,
    pub supported_algs: Option<Vec<String>>,
    pub sha256: Option<fn(input: &str) -> String>,

    /**
     * Is the intended server an authorization server (true) or a resource server
     * (false)? Setting this may allow to avoid parsing the response body to
     * determine the dpop-nonce.
     *
     * @default undefined
     */
    pub is_auth_server: bool,
}

pub async fn build_proof(
    key: Box<dyn Key>,
    alg: &str,
    iss: &str,
    htm: &str,
    htu: &str,
    nonce: Option<&str>,
    ath: Option<&str>,
) {
    if key.bare_jwk().is_none() {
        panic!("Only asymmetric keys can be used as DPoP proofs")
    }

    let now;

    let header = JwtHeader {
        alg: None,
        jku: None,
        jwk: None,
        kid: None,
        x5u: None,
        x5c: None,
        x5t: None,
        x5t_s256: None,
        typ: None,
        cty: None,
        crit: None,
    };
    let payload = JwtPayload {
        iss: None,
        aud: None,
        sub: None,
        exp: None,
        nbf: None,
        iat: None,
        jti: None,
        htm: None,
        htu: None,
        ath: None,
        acr: None,
        azp: None,
        amr: None,
        cnf: None,
        client_id: None,
        scope: None,
        nonce: None,
        at_hash: None,
        c_hash: None,
        s_hash: None,
        auth_time: None,
        name: None,
        family_name: None,
        given_name: None,
        middle_name: None,
        nickname: None,
        preferred_username: None,
        gender: None,
        picture: None,
        profile: None,
        website: None,
        birthdate: None,
        zoneinfo: None,
        locale: None,
        updated_at: None,
        email: None,
        email_verified: None,
        phone_number: None,
        phone_number_verified: None,
        address: None,
        authorization_details: None,
        additional_claims: Default::default(),
    };
    key.create_jwt(header, payload).await.unwrap();
    unimplemented!()
}

pub fn negotiate_alg() {
    unimplemented!()
}

pub async fn subtle_sha256(input: &str) -> String {
    unimplemented!()
}
