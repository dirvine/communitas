import crypto from 'crypto'
import { encode as rsEncode, reconstruct as rsReconstruct } from 'wasm-reed-solomon-erasure'

export interface ReedSolomonConfig {
  dataShards: number
  parityShards: number
  mode?: 'standard' | 'two-person'
}

export interface EncodedShard {
  id: string
  data: Uint8Array
  shardIndex: number
  index: number
  totalShards: number
  isParityShard: boolean
  checksum: string
  version: string
  originalLength: number
}

export class ReedSolomonEncoder {
  private readonly dataShards: number
  private readonly parityShards: number
  private readonly totalShards: number
  private readonly mode: 'standard' | 'two-person'

  constructor(config: ReedSolomonConfig) {
    this.dataShards = config.dataShards
    this.parityShards = config.parityShards
    this.totalShards = this.dataShards + this.parityShards
    this.mode = config.mode ?? 'standard'

    if (this.dataShards <= 0) {
      throw new Error('dataShards must be greater than 0')
    }
    if (this.parityShards <= 0 && this.mode !== 'two-person') {
      throw new Error('parityShards must be greater than 0 for standard mode')
    }
  }

  async encode(data: Uint8Array): Promise<EncodedShard[]> {
    if (this.mode === 'two-person') {
      return this.encodeTwoPerson(data)
    }

    if (data.length === 0) {
      return this.wrapShardData(Array.from({ length: this.totalShards }, () => new Uint8Array(0)), 0)
    }

    const { shards: dataShardPayloads } = this.splitIntoDataShards(data)
    const encodedShards = rsEncode(dataShardPayloads, this.parityShards)

    return this.wrapShardData(encodedShards, data.length)
  }

  async decode(shards: EncodedShard[]): Promise<Uint8Array> {
    if (this.mode === 'two-person') {
      return this.decodeTwoPerson(shards)
    }

    if (shards.length < this.dataShards) {
      throw new Error('InsufficientShardsError')
    }
    if (shards.some((s) => s.totalShards !== this.totalShards)) {
      throw new Error('ShardSetMismatchError')
    }

    const firstVersion = shards[0].version
    if (shards.some((s) => s.version !== firstVersion)) {
      throw new Error('VersionMismatchError')
    }

    const shardMap = new Array<EncodedShard | undefined>(this.totalShards)
    for (const shard of shards) {
      shardMap[shard.shardIndex] = shard
    }

    const missingIndices: number[] = []
    const shardPayloads: Uint8Array[] = new Array(this.totalShards)

    for (let index = 0; index < this.totalShards; index++) {
      const shard = shardMap[index]
      if (shard) {
        this.verifyShard(shard)
        shardPayloads[index] = shard.data
      } else {
        missingIndices.push(index)
        shardPayloads[index] = new Uint8Array(0)
      }
    }

    const completed =
      missingIndices.length > 0
        ? rsReconstruct(shardPayloads, this.parityShards, new Uint32Array(missingIndices))
        : shardPayloads

    const dataShardPayloads = completed.slice(0, this.dataShards)
    const originalLength = shards[0].originalLength
    return this.combineDataShards(dataShardPayloads, originalLength)
  }

  private splitIntoDataShards(data: Uint8Array): { shards: Uint8Array[]; shardSize: number } {
    const shardSize = Math.max(1, Math.ceil(data.length / this.dataShards))
    const shards = Array.from({ length: this.dataShards }, () => new Uint8Array(shardSize))

    for (let i = 0; i < data.length; i++) {
      const shardIndex = Math.floor(i / shardSize)
      const offset = i % shardSize
      shards[shardIndex][offset] = data[i]
    }

    return { shards, shardSize }
  }

  private wrapShardData(shardPayloads: Uint8Array[], originalLength: number): EncodedShard[] {
    const version = this.generateSessionId()
    return shardPayloads.map((bytes, idx) => ({
      id: this.generateShardId(idx),
      data: bytes,
      shardIndex: idx,
      index: idx,
      totalShards: this.totalShards,
      isParityShard: idx >= this.dataShards,
      checksum: this.computeChecksum(bytes),
      version,
      originalLength,
    }))
  }

  private combineDataShards(shards: Uint8Array[], desiredLength: number): Uint8Array {
    if (desiredLength === 0 || shards.length === 0) {
      return new Uint8Array(0)
    }

    const output = new Uint8Array(desiredLength)
    let offset = 0

    for (const shard of shards) {
      if (offset >= desiredLength) {
        break
      }
      const slice = shard.subarray(0, Math.min(shard.length, desiredLength - offset))
      output.set(slice, offset)
      offset += slice.length
    }

    return output
  }

  private encodeTwoPerson(data: Uint8Array): EncodedShard[] {
    const version = this.generateSessionId()
    const shards: EncodedShard[] = []
    for (let i = 0; i < 2; i++) {
      const shardData = data
      shards.push({
        id: this.generateShardId(i),
        data: shardData,
        shardIndex: i,
        index: i,
        totalShards: 2,
        isParityShard: i === 1,
        checksum: this.computeChecksum(shardData),
        version,
        originalLength: data.length,
      })
    }
    return shards
  }

  private async decodeTwoPerson(shards: EncodedShard[]): Promise<Uint8Array> {
    if (shards.length === 0) {
      throw new Error('NoShardsProvided')
    }

    for (const shard of shards) {
      this.verifyShard(shard)
    }

    const candidate = shards.find((s) => s.data.length > 0) ?? shards[0]
    return candidate.data.slice(0, candidate.originalLength)
  }

  private generateShardId(index: number): string {
    const randomBytes = crypto.randomBytes(16)
    return crypto
      .createHash('sha256')
      .update(`shard_${index}_${Date.now()}`)
      .update(randomBytes)
      .digest('hex')
      .slice(0, 16)
  }

  private generateSessionId(): string {
    const bytes = new Uint8Array(8)
    if (typeof crypto.getRandomValues === 'function') {
      crypto.getRandomValues(bytes)
    } else {
      bytes.set(crypto.randomBytes(8))
    }
    return Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('')
  }

  private computeChecksum(data: Uint8Array): string {
    return crypto.createHash('sha256').update(data).digest('hex')
  }

  async benchmark(testSizes: number[] = [1024, 10240, 102400, 1048576]): Promise<any> {
    const results = []

    for (const size of testSizes) {
      const testData = new Uint8Array(size)
      crypto.randomFillSync(testData)

      const encodeStart = performance.now()
      const shards = await this.encode(testData)
      const encodeTime = performance.now() - encodeStart

      const decodeStart = performance.now()
      const decoded = await this.decode(shards)
      const decodeTime = performance.now() - decodeStart

      const minShards = shards.slice(0, this.dataShards)
      const minDecodeStart = performance.now()
      const minDecoded = await this.decode(minShards)
      const minDecodeTime = performance.now() - minDecodeStart

      results.push({
        size,
        encodeTime,
        decodeTime,
        minDecodeTime,
        throughputMBps: size === 0 ? 0 : (size / 1024 / 1024) / (encodeTime / 1000),
        redundancy: this.parityShards / this.dataShards,
        verified: Buffer.compare(testData, decoded.slice(0, size)) === 0 &&
          Buffer.compare(testData, minDecoded.slice(0, size)) === 0,
      })
    }

    return results
  }

  verifyShard(shard: EncodedShard): true {
    const checksum = this.computeChecksum(shard.data)
    if (checksum !== shard.checksum) {
      throw new Error('CorruptedShardError')
    }
    return true
  }

  cleanup(): void {
    // no-op placeholder for future resource cleanup
  }
}
