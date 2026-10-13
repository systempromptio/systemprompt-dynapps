//! The id_token claims Entra ID v2.0 issues, as far as sign-in reads them.

use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    #[serde(default)]
    pub sub: String,
    #[serde(deserialize_with = "one_or_many")]
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub nbf: Option<i64>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub tid: Option<String>,
    #[serde(default)]
    pub oid: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub groups: Option<Vec<String>>,
    // Why: Entra ID sets `_claim_names` when the user is in more groups than fit
    // in the token; those claims must then come from Graph.
    #[serde(default, rename = "_claim_names")]
    pub claim_names: Option<Map<String, Value>>,
    #[serde(default, rename = "_claim_sources")]
    pub claim_sources: Option<Value>,
    #[serde(default)]
    pub jti: Option<String>,
}

fn one_or_many<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(de)? {
        OneOrMany::One(aud) => vec![aud],
        OneOrMany::Many(auds) => auds,
    })
}
