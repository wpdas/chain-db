use base64::{engine::general_purpose, Engine as _};

use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Ecb};
use borsh::{BorshDeserialize, BorshSerialize};
use hex_literal::hex;
use std::str;

use crate::core_tables::user_account::UserAccountTable;

pub struct Base64;

impl Base64 {
    pub fn encode(bytes: Vec<u8>) -> String {
        general_purpose::STANDARD_NO_PAD.encode(&bytes)
    }

    pub fn decode(encoded_data: String) -> Vec<u8> {
        general_purpose::STANDARD_NO_PAD
            .decode(&encoded_data)
            .unwrap()
    }
}

pub struct Base64VecU8;

impl Base64VecU8 {
    pub fn encode<T>(data: T) -> String
    where
        T: BorshSerialize,
    {
        Base64::encode(data.try_to_vec().unwrap())
    }

    pub fn decode<T>(encoded_data: String) -> T
    where
        T: BorshDeserialize,
    {
        let decoded_data = Base64::decode(encoded_data);
        T::try_from_slice(&decoded_data).unwrap()
    }
}

#[test]
fn base64vecu8_test() {
    let user = UserAccountTable {
        id: String::from("123456az"),
        user_name: String::from("Pimpolho.louco"),
        units: 21,
    };

    let encoded_user = Base64VecU8::encode(&user);
    let decoded_user: UserAccountTable = Base64VecU8::decode(encoded_user);
    assert_eq!(user, decoded_user);
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

        let encrypted_msb_blocks: Vec<String> = msg_blocks
            .iter()
            .map(|msg_block| {
                // AesEcb::encrypt(msg_block.to_owned(), "")
                AesEcb::encrypt(&msg_block.to_string(), key)
            })
            .collect();

        encrypted_msb_blocks.join(";")
    }

    pub fn decode(encoded_message: &String, key: &String) -> Option<String> {
        let message_chunks = encoded_message.split(";").collect::<Vec<&str>>();

        let mut has_error = false;

        let decrypted_msg: Vec<String> = message_chunks
            .iter()
            .map(|chunk| {
                let decrypted_chunk = AesEcb::decrypt(&chunk.to_string(), key);

                if decrypted_chunk.is_none() {
                    has_error = true;
                    return "".to_string();
                }

                decrypted_chunk.unwrap()
            })
            .collect();

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
        let cipher = Aes256Ecb::new_from_slices(&key_bytes, &iv).unwrap();
        let decrypted_ciphertext = cipher.decrypt(&mut message_bytes);

        if decrypted_ciphertext.is_err() {
            return None;
        };

        Some(
            str::from_utf8(decrypted_ciphertext.unwrap())
                .unwrap()
                .to_string(),
        )
    }
}
