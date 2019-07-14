extern crate ppslib;

use self::ppslib::crypt::*;
use base64::encode;
use rand::rngs::OsRng;
use rand_core::RngCore;
use std::io::stdin;

fn main() {
    println!("What string to encrypt?");
    let mut message = String::new();
    stdin().read_line(&mut message).unwrap();

    let mut key: [u8; 32] = [0; 32];
    let mut iv: [u8; 16] = [0; 16];

    // In a real program, the key and iv may be determined
    // using some other mechanism. If a password is to be used
    // as a key, an algorithm like PBKDF2, Bcrypt, or Scrypt (all
    // supported by Rust-Crypto!) would be a good choice to derive
    // a password. For the purposes of this example, the key and
    // iv are just random values.
    let mut rng = OsRng;
    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut iv);

    let encrypted_data = encrypt(message.as_bytes(), &key, &iv).ok().unwrap();
    let decrypted_data = decrypt(&encrypted_data[..], &key, &iv).ok().unwrap();

    assert!(message.as_bytes() == &decrypted_data[..]);

    println!("Key: {}", encode(&key));
    println!("IV: {}", encode(&iv));
    println!("Ciphertext: {}", encode(&encrypted_data));
}
