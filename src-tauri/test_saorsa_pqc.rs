// Test what's available in saorsa-pqc 0.3.0
use saorsa_pqc::*;

fn main() {
    println!("Testing saorsa-pqc 0.3.0 exports");
    
    // Try to use ML-KEM-768
    let (pk, sk) = ml_kem_768();
    println!("ML-KEM-768 key pair generated");
    
    // Try to use ML-DSA-65
    let (signing_pk, signing_sk) = ml_dsa_65();
    println!("ML-DSA-65 key pair generated");
}