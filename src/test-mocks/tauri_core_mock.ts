export async function invoke(command: string, args?: any): Promise<any> {
  switch (command) {
    case 'generate_ml_dsa_keypair':
      return {
        public_key: Array(32).fill(2),
        secret_key: Array(64).fill(3),
        algorithm: 'ML-DSA-65',
        created_at: Date.now(),
      }
    case 'ml_dsa_sign': {
      const context = args && args.context ? args.context : 'default'
      const data: number[] = (args && args.data) || []
      const checksum = data.reduce((a: number, b: number) => (a + b) % 256, 0)
      return {
        signature: Array(64).fill(checksum),
        algorithm: 'ML-DSA-65',
        context,
      }
    }
    case 'ml_dsa_verify': {
      const data: number[] = (args && args.data) || []
      const signature: number[] = (args && args.signatureBytes) || []
      const checksum = data.reduce((a: number, b: number) => (a + b) % 256, 0)
      const is_valid = signature.length > 0 && signature[0] === checksum
      return {
        is_valid,
        algorithm: 'ML-DSA-65',
        details: {},
      }
    }
    case 'get_pqc_info':
      return {
        ml_kem_768_public_key_size: '1184',
        ml_kem_768_secret_key_size: '2400',
        ml_kem_768_ciphertext_size: '1088',
        ml_dsa_65_public_key_size: '1312',
        ml_dsa_65_secret_key_size: '2528',
        ml_dsa_65_signature_size: '2420',
        chacha20poly1305_key_size: '32',
        chacha20poly1305_nonce_size: '12',
        algorithms: 'ML-DSA-65, ML-KEM-768',
        version: 'test',
      }
    case 'generate_ml_kem_keypair':
      return {
        public_key: Array(32).fill(11),
        secret_key: Array(64).fill(13),
        algorithm: 'ML-KEM-768',
        created_at: Date.now(),
      }
    case 'ml_kem_encapsulate':
      return {
        ciphertext: Array(32).fill(17),
        shared_secret: Array(32).fill(19),
        algorithm: 'ML-KEM-768',
      }
    case 'ml_kem_decapsulate':
      return Array(32).fill(19)
    case 'pqc_encrypt':
      return {
        ciphertext: Array(16).fill(23),
        nonce: Array(12).fill(29),
        ml_kem_ciphertext: Array(32).fill(31),
        algorithm: 'hybrid',
      }
    case 'pqc_decrypt':
      return Array(16).fill(23)
    default:
      return null
  }
}
