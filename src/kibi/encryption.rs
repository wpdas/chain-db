use base64::{Engine as _, engine::general_purpose};

use aes::Aes256;
use block_modes::{BlockMode, Ecb};
use block_modes::block_padding::Pkcs7;
use hex_literal::hex;
use std::str;

pub struct Base64 {}

impl Base64 {
  pub fn encode(bytes: Vec<u8>) -> String {
    general_purpose::STANDARD_NO_PAD.encode(&bytes)
  }

  pub fn decode(encoded: String) -> Vec<u8> {
    general_purpose::STANDARD_NO_PAD.decode(&encoded).unwrap()
  }
}

// AES ECB
const BITS: usize = 64; // change to 128

type Aes256Ecb = Ecb<Aes256, Pkcs7>;

pub struct AesEcb {}

impl AesEcb {
    pub fn encode(message: &String, key: &String) -> String {
      let message_bytes = message.as_bytes();

      let msg_blocks = message_bytes
        .chunks(BITS - 1)
        .map(str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap();

      let encrypted_msb_blocks: Vec<String> = msg_blocks.iter()
        .map(|msg_block| {
          // AesEcb::encrypt(msg_block.to_owned(), "")
          AesEcb::encrypt(&msg_block.to_string(), key)
        })
        .collect();

      encrypted_msb_blocks.join(";")
    }

    pub fn decode (encoded_message: &String, key: &String) -> Option<String> {
      let message_chunks = encoded_message.split(";")
        .collect::<Vec<&str>>();

      let mut has_error = false;

      let decrypted_msg: Vec<String> = message_chunks.iter()
      .map(|chunk| {
        let decrypted_chunk = AesEcb::decrypt(&chunk.to_string(), key);

        if decrypted_chunk.is_none() {
          has_error = true;
          return "".to_string();
        }

        decrypted_chunk.unwrap()
      }).collect();

      if has_error {
        return None;
      }

      Some(decrypted_msg.join(""))
    }

    pub fn encrypt(message: &String, key: &String) -> String {
      let message_bytes = message.as_bytes();
      let key_bytes = hex::decode(key.as_bytes()).expect("Decoding failed");
      let iv = hex!("");
      let cipher = Aes256Ecb::new_from_slices(&key_bytes, &iv).unwrap();
      let pos = message_bytes.len();
      let mut buffer = [0u8; BITS];
      buffer[..pos].copy_from_slice(message_bytes);
      let ciphertext = cipher.encrypt(&mut buffer, pos).unwrap();
      hex::encode(ciphertext)
    }

    pub fn decrypt(message: &String, key: &String) -> Option<String> {
      let mut message_bytes = hex::decode(message).unwrap();
      let key_bytes = hex::decode(key.as_bytes()).expect("Decoding failed");
      let iv = hex!("");
      let cipher = Aes256Ecb::new_from_slices(&key_bytes,&iv).unwrap();
      let decrypted_ciphertext = cipher.decrypt(&mut message_bytes);

      if decrypted_ciphertext.is_err() {
        return None;
      };

      Some(str::from_utf8(decrypted_ciphertext.unwrap()).unwrap().to_string())
    }
}