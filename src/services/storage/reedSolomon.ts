import crypto from 'crypto'

export interface ReedSolomonConfig {
  dataShards: number
  parityShards: number
}

export interface EncodedShard {
  id: string
  data: Uint8Array
  shardIndex: number
  totalShards: number
  isParityShard: boolean
  checksum: string
}

export interface DecodedData {
  data: Uint8Array
  checksum: string
  recoveredShards: number[]
}

export class ReedSolomonEncoder {
  private dataShards: number
  private parityShards: number
  private totalShards: number
  private matrix: number[][]

  constructor(config: ReedSolomonConfig) {
    this.dataShards = config.dataShards
    this.parityShards = config.parityShards
    this.totalShards = this.dataShards + this.parityShards
    this.matrix = this.buildVandermondeMatrix()
  }

  async encode(data: Uint8Array): Promise<EncodedShard[]> {
    // Calculate shard size (pad if necessary)
    const shardSize = Math.ceil(data.length / this.dataShards)
    const paddedData = new Uint8Array(shardSize * this.dataShards)
    paddedData.set(data)

    // Create data shards
    const shards: EncodedShard[] = []
    for (let i = 0; i < this.dataShards; i++) {
      const shardData = paddedData.slice(i * shardSize, (i + 1) * shardSize)
      shards.push({
        id: this.generateShardId(i),
        data: shardData,
        shardIndex: i,
        totalShards: this.totalShards,
        isParityShard: false,
        checksum: this.computeChecksum(shardData)
      })
    }

    // Generate parity shards using Reed-Solomon encoding
    for (let i = 0; i < this.parityShards; i++) {
      const parityData = await this.generateParityShard(shards, i)
      shards.push({
        id: this.generateShardId(this.dataShards + i),
        data: parityData,
        shardIndex: this.dataShards + i,
        totalShards: this.totalShards,
        isParityShard: true,
        checksum: this.computeChecksum(parityData)
      })
    }

    return shards
  }

  async decode(shards: EncodedShard[]): Promise<DecodedData> {
    if (shards.length < this.dataShards) {
      throw new Error(`Insufficient shards: need ${this.dataShards}, have ${shards.length}`)
    }

    // Validate shards
    for (const shard of shards) {
      if (this.computeChecksum(shard.data) !== shard.checksum) {
        throw new Error(`Shard ${shard.id} checksum mismatch`)
      }
    }

    // Sort shards by index
    shards.sort((a, b) => a.shardIndex - b.shardIndex)

    const recoveredShards: number[] = []
    const decodedShards: Uint8Array[] = new Array(this.dataShards)

    // Use available data shards first
    for (const shard of shards) {
      if (!shard.isParityShard && shard.shardIndex < this.dataShards) {
        decodedShards[shard.shardIndex] = shard.data
      }
    }

    // Recover missing data shards using parity
    for (let i = 0; i < this.dataShards; i++) {
      if (!decodedShards[i]) {
        decodedShards[i] = await this.recoverDataShard(shards, i)
        recoveredShards.push(i)
      }
    }

    // Concatenate decoded data shards
    const totalLength = decodedShards.reduce((sum, shard) => sum + shard.length, 0)
    const result = new Uint8Array(totalLength)
    let offset = 0
    
    for (const shard of decodedShards) {
      result.set(shard, offset)
      offset += shard.length
    }

    return {
      data: result,
      checksum: this.computeChecksum(result),
      recoveredShards
    }
  }

  private async generateParityShard(dataShards: EncodedShard[], parityIndex: number): Promise<Uint8Array> {
    const shardSize = dataShards[0].data.length
    const parityData = new Uint8Array(shardSize)

    // Use Galois Field arithmetic for Reed-Solomon encoding
    for (let byteIndex = 0; byteIndex < shardSize; byteIndex++) {
      let parityByte = 0
      
      for (let dataIndex = 0; dataIndex < this.dataShards; dataIndex++) {
        const coefficient = this.matrix[this.dataShards + parityIndex][dataIndex]
        const dataByte = dataShards[dataIndex].data[byteIndex]
        parityByte ^= this.galoisMultiply(coefficient, dataByte)
      }
      
      parityData[byteIndex] = parityByte
    }

    return parityData
  }

  private async recoverDataShard(availableShards: EncodedShard[], missingIndex: number): Promise<Uint8Array> {
    const shardSize = availableShards[0].data.length
    const recoveredData = new Uint8Array(shardSize)

    // Build system of linear equations using available shards
    const equations: number[][] = []
    const results: Uint8Array[] = []

    for (const shard of availableShards.slice(0, this.dataShards)) {
      equations.push(this.matrix[shard.shardIndex])
      results.push(shard.data)
    }

    // Solve for each byte position
    for (let byteIndex = 0; byteIndex < shardSize; byteIndex++) {
      const rightSide = results.map(shard => shard[byteIndex])
      const solution = this.solveLinearSystem(equations, rightSide)
      recoveredData[byteIndex] = solution[missingIndex]
    }

    return recoveredData
  }

  private buildVandermondeMatrix(): number[][] {
    const matrix: number[][] = []
    
    // Create Vandermonde matrix for Reed-Solomon
    for (let row = 0; row < this.totalShards; row++) {
      matrix[row] = []
      for (let col = 0; col < this.dataShards; col++) {
        matrix[row][col] = this.galoisPower(row + 1, col)
      }
    }
    
    return matrix
  }

  private galoisMultiply(a: number, b: number): number {
    // Galois Field GF(2^8) multiplication
    if (a === 0 || b === 0) return 0
    
    const logA = this.getLogTable()[a]
    const logB = this.getLogTable()[b]
    const sum = (logA + logB) % 255
    
    return this.getExpTable()[sum]
  }

  private galoisPower(base: number, exp: number): number {
    if (exp === 0) return 1
    if (base === 0) return 0
    
    let result = 1
    let currentBase = base
    let currentExp = exp
    
    while (currentExp > 0) {
      if (currentExp & 1) {
        result = this.galoisMultiply(result, currentBase)
      }
      currentBase = this.galoisMultiply(currentBase, currentBase)
      currentExp >>= 1
    }
    
    return result
  }

  private solveLinearSystem(equations: number[][], results: number[]): number[] {
    // Gaussian elimination in Galois Field
    const n = equations.length
    const augmented = equations.map((row, i) => [...row, results[i]])
    
    // Forward elimination
    for (let i = 0; i < n; i++) {
      // Find pivot
      let pivotRow = i
      for (let k = i + 1; k < n; k++) {
        if (augmented[k][i] !== 0) {
          pivotRow = k
          break
        }
      }
      
      if (pivotRow !== i) {
        [augmented[i], augmented[pivotRow]] = [augmented[pivotRow], augmented[i]]
      }
      
      // Eliminate column
      const pivot = augmented[i][i]
      if (pivot === 0) continue
      
      const pivotInv = this.galoisInverse(pivot)
      
      for (let k = i + 1; k < n; k++) {
        const factor = this.galoisMultiply(augmented[k][i], pivotInv)
        for (let j = 0; j <= n; j++) {
          augmented[k][j] ^= this.galoisMultiply(factor, augmented[i][j])
        }
      }
    }
    
    // Back substitution
    const solution = new Array(n).fill(0)
    for (let i = n - 1; i >= 0; i--) {
      solution[i] = augmented[i][n]
      for (let j = i + 1; j < n; j++) {
        solution[i] ^= this.galoisMultiply(augmented[i][j], solution[j])
      }
      if (augmented[i][i] !== 0) {
        solution[i] = this.galoisMultiply(solution[i], this.galoisInverse(augmented[i][i]))
      }
    }
    
    return solution
  }

  private galoisInverse(a: number): number {
    if (a === 0) throw new Error('Cannot invert 0 in Galois Field')
    
    const logA = this.getLogTable()[a]
    const invLog = (255 - logA) % 255
    
    return this.getExpTable()[invLog]
  }

  private getLogTable(): number[] {
    // Pre-computed logarithm table for GF(2^8)
    return [
      0, 0, 1, 25, 2, 50, 26, 198, 3, 223, 51, 238, 27, 104, 199, 75,
      4, 100, 224, 14, 52, 141, 239, 129, 28, 193, 105, 248, 200, 8, 76, 113,
      5, 138, 101, 47, 225, 36, 15, 33, 53, 147, 142, 218, 240, 18, 130, 69,
      29, 181, 194, 125, 106, 39, 249, 185, 201, 154, 9, 120, 77, 228, 114, 166,
      6, 191, 139, 98, 102, 221, 48, 253, 226, 152, 37, 179, 16, 145, 34, 136,
      54, 208, 148, 206, 143, 150, 219, 189, 241, 210, 19, 92, 131, 56, 70, 64,
      30, 66, 182, 163, 195, 72, 126, 110, 107, 58, 40, 84, 250, 133, 186, 61,
      202, 94, 155, 159, 10, 21, 121, 43, 78, 212, 229, 172, 115, 243, 167, 87,
      7, 112, 192, 247, 140, 128, 99, 13, 103, 74, 222, 237, 49, 197, 254, 24,
      227, 165, 153, 119, 38, 184, 180, 124, 17, 68, 146, 217, 35, 32, 137, 46,
      55, 63, 209, 91, 149, 188, 207, 205, 144, 135, 151, 178, 220, 252, 190, 97,
      242, 86, 211, 171, 20, 42, 93, 158, 132, 60, 57, 83, 71, 109, 65, 162,
      31, 45, 67, 216, 183, 123, 164, 118, 196, 23, 73, 236, 127, 12, 111, 246,
      108, 161, 59, 82, 41, 157, 85, 170, 251, 96, 134, 177, 187, 204, 62, 90,
      203, 89, 95, 176, 156, 169, 160, 81, 11, 245, 22, 235, 122, 117, 44, 215,
      79, 174, 213, 233, 230, 231, 173, 232, 116, 214, 244, 234, 168, 80, 88, 175
    ]
  }

  private getExpTable(): number[] {
    // Pre-computed exponential table for GF(2^8)
    return [
      1, 2, 4, 8, 16, 32, 64, 128, 29, 58, 116, 232, 205, 135, 19, 38,
      76, 152, 45, 90, 180, 117, 234, 201, 143, 3, 6, 12, 24, 48, 96, 192,
      157, 39, 78, 156, 37, 74, 148, 53, 106, 212, 181, 119, 238, 193, 159, 35,
      70, 140, 5, 10, 20, 40, 80, 160, 93, 186, 105, 210, 185, 111, 222, 161,
      95, 190, 97, 194, 153, 47, 94, 188, 101, 202, 137, 15, 30, 60, 120, 240,
      253, 231, 211, 187, 107, 214, 177, 127, 254, 225, 223, 163, 91, 182, 113, 226,
      217, 175, 67, 134, 17, 34, 68, 136, 13, 26, 52, 104, 208, 189, 103, 206,
      129, 31, 62, 124, 248, 237, 199, 147, 59, 118, 236, 197, 151, 51, 102, 204,
      133, 23, 46, 92, 184, 109, 218, 169, 79, 158, 33, 66, 132, 21, 42, 84,
      168, 77, 154, 41, 82, 164, 85, 170, 73, 146, 57, 114, 228, 213, 183, 115,
      230, 209, 191, 99, 198, 145, 63, 126, 252, 229, 215, 179, 123, 246, 241, 255,
      227, 219, 171, 75, 150, 49, 98, 196, 149, 55, 110, 220, 165, 87, 174, 65,
      130, 25, 50, 100, 200, 141, 7, 14, 28, 56, 112, 224, 221, 167, 83, 166,
      81, 162, 89, 178, 121, 242, 249, 239, 195, 155, 43, 86, 172, 69, 138, 9,
      18, 36, 72, 144, 61, 122, 244, 245, 247, 243, 251, 235, 203, 139, 11, 22,
      44, 88, 176, 125, 250, 233, 207, 131, 27, 54, 108, 216, 173, 71, 142, 1
    ]
  }

  private generateShardId(index: number): string {
    return crypto.createHash('sha256')
      .update(`shard_${index}_${Date.now()}`)
      .digest('hex')
      .slice(0, 16)
  }

  private computeChecksum(data: Uint8Array): string {
    return crypto.createHash('blake3', { outputLength: 32 })
      .update(data)
      .digest('hex')
  }

  // Performance benchmarking
  async benchmark(testSizes: number[] = [1024, 10240, 102400, 1048576]): Promise<any> {
    const results = []
    
    for (const size of testSizes) {
      const testData = new Uint8Array(size)
      crypto.randomFillSync(testData)
      
      const encodeStart = performance.now()
      const shards = await this.encode(testData)
      const encodeTime = performance.now() - encodeStart
      
      // Test decoding with all shards
      const decodeStart = performance.now()
      const decoded = await this.decode(shards)
      const decodeTime = performance.now() - decodeStart
      
      // Test decoding with minimum shards
      const minShards = shards.slice(0, this.dataShards)
      const minDecodeStart = performance.now()
      const minDecoded = await this.decode(minShards)
      const minDecodeTime = performance.now() - minDecodeStart
      
      results.push({
        size,
        encodeTime,
        decodeTime,
        minDecodeTime,
        throughputMBps: (size / 1024 / 1024) / (encodeTime / 1000),
        redundancy: this.parityShards / this.dataShards,
        verified: Buffer.compare(testData, decoded.data.slice(0, size)) === 0
      })
    }
    
    return results
  }
}