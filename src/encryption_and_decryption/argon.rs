// Code inspired by https://github.com/skerkour/kerkour.com/blob/main/2022/rust_file_encryption_with_password/src/main.rs

use anyhow::anyhow;
use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use rand::{rngs::OsRng, RngCore};
use std::{
    fs::File,
    io::{Read, Write},
};
use std::str;
use zeroize::Zeroize;


// Orientation: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
fn argon2_config<'a>() -> argon2::Config<'a> {
    return argon2::Config {
        variant: argon2::Variant::Argon2id,
        hash_length: 32,
        lanes: 8,
        mem_cost: 16 * 1024, //Todo change to 4048 or ask user
        time_cost: 8,
        ..Default::default()
    };
}

pub fn encrypt_text(
    text: &str,
    dist_file_path: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    let argon2_config = argon2_config();

    let mut salt = [0u8; 32];
    let mut nonce = [0u8; 19];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let mut key = argon2::hash_raw(password.as_bytes(), &salt, &argon2_config)?;

    let aead = XChaCha20Poly1305::new(key[..32].as_ref().into());
    let mut stream_encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    let mut dist_file = File::create(dist_file_path)?;

    dist_file.write(&salt)?;
    dist_file.write(&nonce)?;


    let ciphertext = stream_encryptor
        .encrypt_next(text.as_bytes())
        .map_err(|err| anyhow!("Encrypting file: {}", err))?;
    dist_file.write(&ciphertext)?;

    salt.zeroize();
    nonce.zeroize();
    key.zeroize();

    Ok(())
}


pub fn decrypt_text(
    encrypted_file_path: &str,
    password: &str,
) -> Result<String, anyhow::Error> {
    let mut salt = [0u8; 32];
    let mut nonce = [0u8; 19];

    let mut encrypted_file = File::open(encrypted_file_path)?;
    let mut file_content = String::new();

    let mut read_count = encrypted_file.read(&mut salt)?;
    if read_count != salt.len() {
        return Err(anyhow!("Error reading salt."));
    }

    read_count = encrypted_file.read(&mut nonce)?;
    if read_count != nonce.len() {
        return Err(anyhow!("Error reading nonce."));
    }

    let argon2_config = argon2_config();

    let mut key = argon2::hash_raw(password.as_bytes(), &salt, &argon2_config)?;

    let aead = XChaCha20Poly1305::new(key[..32].as_ref().into());
    let mut stream_decryptor = stream::DecryptorBE32::from_aead(aead, nonce.as_ref().into());

    let buf:Vec<u8> = Vec::new();

    let plaintext = stream_decryptor
                .decrypt_next(buf.as_slice())
                .map_err(|err| anyhow!("Decrypting file: {}", err))?;
            file_content.push_str(str::from_utf8(&plaintext)?);


    salt.zeroize();
    nonce.zeroize();
    key.zeroize();
    Ok(file_content)
}