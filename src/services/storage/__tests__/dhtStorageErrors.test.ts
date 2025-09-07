import { describe, it, expect, beforeEach } from 'vitest'
import { DHTStorage } from '../dhtStorage'

describe('DHTStorage error handling', () => {
  let dht: DHTStorage

  beforeEach(async () => {
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

  it('getRawBlock throws for unknown block', async () => {
    await expect(dht.getRawBlock('missing')).rejects.toThrow('Block missing not found')
  })

  it('get throws for unknown block', async () => {
    await expect(dht.get('deadbeef')).rejects.toThrow('Block deadbeef not found')
  })
})

