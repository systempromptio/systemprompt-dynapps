//! Decoding of the SP's HTTP-Redirect `AuthnRequest`, so a flow test can
//! answer the request the SP actually issued without a browser.

use std::io::Read as _;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRedirect {
    pub request_id: String,
    pub relay_state: Option<String>,
    pub destination: Option<String>,
}

// Decode the `SAMLRequest` of an HTTP-Redirect URL (base64, raw DEFLATE).
pub fn parse_authn_request_redirect(url: &str) -> ParsedRedirect {
    let url = url::Url::parse(url).expect("redirect URL");
    let param = |k: &str| {
        url.query_pairs()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.into_owned())
    };
    let compressed = STANDARD
        .decode(param("SAMLRequest").expect("SAMLRequest param"))
        .expect("SAMLRequest base64");
    let mut xml = String::new();
    flate2::read::DeflateDecoder::new(compressed.as_slice())
        .read_to_string(&mut xml)
        .expect("SAMLRequest DEFLATE");
    let root = &xml[..xml.find('>').expect("AuthnRequest tag")];
    ParsedRedirect {
        request_id: attr(root, "ID").expect("AuthnRequest ID"),
        relay_state: param("RelayState"),
        destination: attr(root, "Destination"),
    }
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let len = tag[start..].find('"')?;
    Some(tag[start..start + len].to_owned())
}
