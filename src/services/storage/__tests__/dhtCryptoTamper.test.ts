import { describe, it, expect, beforeEach } from 'vitest'
import { DHTStorage } from '../dhtStorage'
import { cryptoManager } from '../../security/cryptoManager'

describe('DHTStorage crypto integrity', () => {
  let dht: DHTStorage

  beforeEach(async () => {
    await cryptoManager.generateKeyPair('pk_test_123')
    dht = new DHTStorage({
      identity: {
        fourWords: 'alpha-beta-gamma-delta',
        publicKey: 'pk_test_123',
        dhtAddress: 'dht://alpha-beta-gamma-delta'
      },
      bootstrapNodes: ['local:5001'],
      replicationFactor: 3,
    })
    await dht.connect()
  })

  it('decrypt fails if ciphertext is tampered (AES-GCM integrity)', async () => {
    const key = new Uint8Array(32)
    crypto.getRandomValues(key)
    const data = new Uint8Array(128).map((_, i) => (i % 256))

    const block = await dht.encrypt(data, key)
    // Tamper with ciphertext
    block.encryptedData[0] = block.encryptedData[0] ^ 0xff

    await expect(dht.decrypt(block, key)).rejects.toBeTruthy()
  })
})
