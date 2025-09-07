import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { MarkdownWebPublisher } from '../markdownPublisher'
import { DHTStorage } from '../dhtStorage'
import { ReedSolomonEncoder } from '../reedSolomon'
import { cryptoManager } from '../../security/cryptoManager'

describe('MarkdownWebPublisher versioning', () => {
  let publisher: MarkdownWebPublisher
  let dht: DHTStorage
  let encoder: ReedSolomonEncoder

  beforeEach(async () => {
    await cryptoManager.generateKeyPair('pk_test_123')
    const identity = {
      fourWords: 'alpha-beta-gamma-delta',
      publicKey: 'pk_test_123',
      dhtAddress: 'dht://alpha-beta-gamma-delta'
    }
    dht = new DHTStorage({ identity, bootstrapNodes: ['local:5001'], replicationFactor: 3 })
    encoder = new ReedSolomonEncoder({ dataShards: 2, parityShards: 1 })
    publisher = new MarkdownWebPublisher({ identity, dht, encoder, baseDirectory: '/web/' })
    await publisher.initialize()
  })

  afterEach(async () => {
    await publisher.destroy()
  })

  it('bumps patch version after initial publish', async () => {
    await publisher.addFile('/web/home.md', '# V1')
    const first = await publisher.publish()
    expect(first.manifest.version).toBe('1.0.0')

    // Change content and publish again
    await publisher.updateFile('/web/home.md', '# V2')
    const second = await publisher.publish()
    expect(second.manifest.version).toBe('1.0.1')
    expect(second.manifest.previousVersion).toBe(first.manifest.version)
  })
})
