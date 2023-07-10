use base64::{Engine as _, engine::general_purpose};

pub struct Base64 {}

impl Base64 {
  pub fn encode(bytes: Vec<u8>) -> String {
    general_purpose::STANDARD_NO_PAD.encode(&bytes)
  }

  pub fn decode(encoded: String) -> Vec<u8> {
    general_purpose::STANDARD_NO_PAD.decode(&encoded).unwrap()
  }
}