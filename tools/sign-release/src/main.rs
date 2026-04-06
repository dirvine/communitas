// SPDX-License-Identifier: MIT OR Apache-2.0

//! Release artifact signer using ML-DSA-65
//! 
//! Usage: sign-release <secret_key_hex_or_base64> <file_to_sign>
//! Output: Base64-encoded signature to stdout
//!
//! The secret key can be provided as:
//! - Hex-encoded (8064 characters for 4032 bytes)
//! - Base64-encoded (~5376 characters for 4032 bytes)

use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: sign-release <secret_key> <file_to_sign>");
        process::exit(1);
    }

    let sk_input = &args[1];
    let file_path = &args[2];

    // Try to decode the secret key (hex or base64)
    let sk_bytes = decode_secret_key(sk_input);

    // ML-DSA-65 secret key is 4032 bytes
    if sk_bytes.len() != 4032 {
        eprintln!(
            "Invalid secret key length: expected 4032 bytes, got {} bytes (input was {} chars)",
            sk_bytes.len(),
            sk_input.len()
        );
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

fn decode_secret_key(input: &str) -> Vec<u8> {
    // Clean up the input - remove whitespace
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    // Try hex first (should be 8064 chars for 4032 bytes)
    if cleaned.len() == 8064 {
        match hex::decode(&cleaned) {
            Ok(bytes) => return bytes,
            Err(e) => eprintln!("Failed to decode as hex: {}", e),
        }
    }

    // Try base64 (should be ~5376 chars for 4032 bytes)
    // Allow some flexibility in base64 length
    if cleaned.len() >= 5300 && cleaned.len() <= 5500 {
        match base64::decode(&cleaned) {
            Ok(bytes) => return bytes,
            Err(e) => eprintln!("Failed to decode as base64: {}", e),
        }
    }

    // Try hex with any length
    let hex_cleaned: String = cleaned
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    
    if hex_cleaned.len() >= 8000 && hex_cleaned.len() <= 8100 {
        match hex::decode(&hex_cleaned) {
            Ok(bytes) => {
                eprintln!("Note: decoded {} hex chars to {} bytes", hex_cleaned.len(), bytes.len());
                return bytes;
            }
            Err(e) => eprintln!("Failed to decode filtered hex: {}", e),
        }
    }

    eprintln!(
        "Could not decode secret key. Input length: {} chars. \
         Expected: 8064 hex chars or ~5376 base64 chars",
        cleaned.len()
    );
    process::exit(1);
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
