// SPDX-License-Identifier: MIT OR Apache-2.0

//! Release artifact signer using ML-DSA-65
//! 
//! Usage: sign-release <secret_key_hex> <file_to_sign>
//! Output: Base64-encoded signature to stdout

use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: sign-release <secret_key_hex> <file_to_sign>");
        process::exit(1);
    }

    let sk_hex = &args[1];
    let file_path = &args[2];

    // Decode secret key from hex
    let sk_bytes = match hex::decode(sk_hex.trim()) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to decode secret key: {}", e);
            process::exit(1);
        }
    };

    // ML-DSA-65 secret key is 4032 bytes
    if sk_bytes.len() != 4032 {
        eprintln!("Invalid secret key length: expected 4032 bytes, got {}", sk_bytes.len());
        process::exit(1);
    }

    // Read file to sign
    let message = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            process::exit(1);
        }
    };

    // Sign using ML-DSA-65
    let signature = sign_ml_dsa_65(&sk_bytes, &message);

    // Output base64-encoded signature
    println!("{}", base64::encode(&signature));
}

fn sign_ml_dsa_65(sk_bytes: &[u8], message: &[u8]) -> Vec<u8> {
    use fips204::ml_dsa_65;
    use fips204::traits::{SerDes, Signer};
    use rand_core::OsRng;

    let sk_array: [u8; 4032] = sk_bytes.try_into().expect("Valid key length");
    let signing_key = ml_dsa_65::PrivateKey::try_from_bytes(sk_array)
        .expect("Valid ML-DSA-65 secret key");

    let sig = signing_key
        .try_sign_with_rng(&mut OsRng, message, b"")
        .expect("Signing should not fail");

    sig.to_vec()
}
