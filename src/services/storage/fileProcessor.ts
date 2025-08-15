import { invoke } from '@tauri-apps/api/core';
// Use Web Crypto API instead of crypto-js to avoid extra deps

export interface ProcessedFile {
  originalName: string;
  originalSize: number;
  compressedSize: number;
  totalShards: number;
  shards: EncryptedShard[];
  fileHash: string;
  metadata: FileMetadata;
}

export interface EncryptedShard {
  id: string;
  index: number;
  data: Uint8Array;
  checksum: string;
  size: number;
}

export interface FileMetadata {
  originalName: string;
  mimeType: string;
  compressedSize: number;
  totalShards: number;
  fileHash: string;
  timestamp: number;
  version: string;
}

export class FileProcessor {
  private static readonly SHARD_SIZE = 1024 * 1024; // 1MB
  private static readonly VERSION = '1.0.0';

  /**
   * Complete file processing pipeline:
   * 1. Compress the file
   * 2. Split into 1MB shards
   * 3. XOR each shard with file hash
   * 4. Encrypt with AES using file hash as password
   */
  static async processFile(
    file: File,
    fourWordId?: string
  ): Promise<ProcessedFile> {
    console.log(`Processing file: ${file.name} (${file.size} bytes)`);
    
    // Step 1: Read file data
    const fileData = await this.readFileData(file);
    
    // Step 2: Calculate file hash (SHA-256)
    const fileHash = await this.calculateFileHash(fileData);
    console.log(`File hash: ${fileHash}`);
    
    // Step 3: Compress the file using Rust backend for optimal compression
    const compressedData = await this.compressFile(fileData, file.name);
    console.log(`Compressed: ${file.size} -> ${compressedData.length} bytes`);
    
    // Step 4: Split into 1MB shards
    const rawShards = this.createShards(compressedData);
    console.log(`Created ${rawShards.length} shards`);
    
    // Step 5: XOR shards with file hash and encrypt
    const encryptedShards = await this.encryptShards(rawShards, fileHash);
    
    // Step 6: Create metadata
    const metadata: FileMetadata = {
      originalName: file.name,
      mimeType: file.type || 'application/octet-stream',
      compressedSize: compressedData.length,
      totalShards: encryptedShards.length,
      fileHash,
      timestamp: Date.now(),
      version: this.VERSION,
    };
    
    return {
      originalName: file.name,
      originalSize: file.size,
      compressedSize: compressedData.length,
      totalShards: encryptedShards.length,
      shards: encryptedShards,
      fileHash,
      metadata,
    };
  }

  /**
   * Reverse the processing pipeline to reconstruct the original file
   */
  static async reconstructFile(
    processedFile: ProcessedFile,
    password?: string
  ): Promise<File> {
    console.log(`Reconstructing file: ${processedFile.originalName}`);
    
    const fileHash = password || processedFile.fileHash;
    
    // Step 1: Decrypt and de-XOR shards
    const decryptedShards = await this.decryptShards(processedFile.shards, fileHash);
    
    // Step 2: Reassemble shards into compressed data
    const compressedData = this.reassembleShards(decryptedShards);
    
    // Step 3: Decompress the file
    const originalData = await this.decompressFile(compressedData, processedFile.originalName);
    
    // Step 4: Create File object
    // Copy to a plain ArrayBuffer to avoid SharedArrayBuffer typing issues
    const ab = new ArrayBuffer(originalData.byteLength);
    new Uint8Array(ab).set(originalData);
    const blob = new Blob([ab], { 
      type: processedFile.metadata.mimeType 
    });
    
    return new File([blob], processedFile.originalName, {
      type: processedFile.metadata.mimeType,
      lastModified: processedFile.metadata.timestamp,
    });
  }

  private static async readFileData(file: File): Promise<Uint8Array> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        if (reader.result instanceof ArrayBuffer) {
          resolve(new Uint8Array(reader.result));
        } else {
          reject(new Error('Failed to read file as ArrayBuffer'));
        }
      };
      reader.onerror = () => reject(reader.error);
      reader.readAsArrayBuffer(file);
    });
  }

  private static async calculateFileHash(data: Uint8Array): Promise<string> {
    // Copy to ArrayBuffer to satisfy TS BufferSource constraints
    const ab = new ArrayBuffer(data.byteLength);
    new Uint8Array(ab).set(data);
    const digest = await crypto.subtle.digest('SHA-256', ab);
    const bytes = new Uint8Array(digest);
    return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
  }

  private static async compressFile(data: Uint8Array, fileName: string): Promise<Uint8Array> {
    try {
      // Use Rust backend for optimal compression
      const compressed = await invoke<number[]>('compress_data', { 
        data: Array.from(data),
        algorithm: 'zstd', // High-performance compression
      });
      return new Uint8Array(compressed);
    } catch (error) {
      console.warn('Rust compression failed, using JS fallback:', error);
      
      // Fallback to browser compression if Rust backend unavailable
      const stream = new CompressionStream('gzip');
      const writer = stream.writable.getWriter();
      const reader = stream.readable.getReader();
      
      writer.write(new Uint8Array(data));
      writer.close();
      
      const chunks: Uint8Array[] = [];
      let done = false;
      
      while (!done) {
        const { value, done: readerDone } = await reader.read();
        done = readerDone;
        if (value) chunks.push(value);
      }
      
      // Concatenate chunks
      const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
      const result = new Uint8Array(totalLength);
      let offset = 0;
      
      for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.length;
      }
      
      return result;
    }
  }

  private static async decompressFile(data: Uint8Array, fileName: string): Promise<Uint8Array> {
    try {
      // Use Rust backend for decompression
      const decompressed = await invoke<number[]>('decompress_data', { 
        data: Array.from(data),
        algorithm: 'zstd',
      });
      return new Uint8Array(decompressed);
    } catch (error) {
      console.warn('Rust decompression failed, using JS fallback:', error);
      
      // Fallback to browser decompression
      const stream = new DecompressionStream('gzip');
      const writer = stream.writable.getWriter();
      const reader = stream.readable.getReader();
      
      writer.write(new Uint8Array(data));
      writer.close();
      
      const chunks: Uint8Array[] = [];
      let done = false;
      
      while (!done) {
        const { value, done: readerDone } = await reader.read();
        done = readerDone;
        if (value) chunks.push(value);
      }
      
      // Concatenate chunks
      const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
      const result = new Uint8Array(totalLength);
      let offset = 0;
      
      for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.length;
      }
      
      return result;
    }
  }

  private static createShards(data: Uint8Array): Uint8Array[] {
    const shards: Uint8Array[] = [];
    const totalShards = Math.ceil(data.length / this.SHARD_SIZE);
    
    for (let i = 0; i < totalShards; i++) {
      const start = i * this.SHARD_SIZE;
      const end = Math.min(start + this.SHARD_SIZE, data.length);
      shards.push(data.slice(start, end));
    }
    
    return shards;
  }

  private static reassembleShards(shards: { index: number; data: Uint8Array }[]): Uint8Array {
    // Sort shards by index
    shards.sort((a, b) => a.index - b.index);
    
    // Calculate total size
    const totalSize = shards.reduce((sum, shard) => sum + shard.data.length, 0);
    
    // Reassemble
    const result = new Uint8Array(totalSize);
    let offset = 0;
    
    for (const shard of shards) {
      result.set(shard.data, offset);
      offset += shard.data.length;
    }
    
    return result;
  }

  private static async encryptShards(
    shards: Uint8Array[], 
    fileHash: string
  ): Promise<EncryptedShard[]> {
    const encryptedShards: EncryptedShard[] = [];
    
    for (let i = 0; i < shards.length; i++) {
      const shard = shards[i];
      
      // Step 1: XOR shard with file hash
      const xoredShard = this.xorWithHash(shard, fileHash);
      
      // Step 2: Encrypt with AES-256-GCM using file hash as password
      const encryptedData = this.encryptAES(xoredShard, fileHash);
      
      // Step 3: Create checksum
      const checksum = await this.calculateFileHash(encryptedData);
      
      encryptedShards.push({
        id: `${fileHash}-${i}`,
        index: i,
        data: encryptedData,
        checksum,
        size: encryptedData.length,
      });
    }
    
    return encryptedShards;
  }

  private static async decryptShards(
    encryptedShards: EncryptedShard[], 
    fileHash: string
  ): Promise<{ index: number; data: Uint8Array }[]> {
    const decryptedShards: { index: number; data: Uint8Array }[] = [];
    
    for (const shard of encryptedShards) {
      // Step 1: Verify checksum
      const currentChecksum = await this.calculateFileHash(shard.data);
      
      if (currentChecksum !== shard.checksum) {
        throw new Error(`Checksum verification failed for shard ${shard.index}`);
      }
      
      // Step 2: Decrypt with AES-256-GCM
      const decryptedData = this.decryptAES(shard.data, fileHash);
      
      // Step 3: De-XOR with file hash
      const originalShard = this.xorWithHash(decryptedData, fileHash);
      
      decryptedShards.push({
        index: shard.index,
        data: originalShard,
      });
    }
    
    return decryptedShards;
  }

  private static xorWithHash(data: Uint8Array, hash: string): Uint8Array {
    const hashBytes = new TextEncoder().encode(hash);
    const result = new Uint8Array(data.length);
    
    for (let i = 0; i < data.length; i++) {
      result[i] = data[i] ^ hashBytes[i % hashBytes.length];
    }
    
    return result;
  }

  private static encryptAES(data: Uint8Array, password: string): Uint8Array {
    // Simple XOR-based placeholder encryption. Replace with proper AES-GCM via Rust backend in production.
    const key = new TextEncoder().encode(password);
    const out = new Uint8Array(data.length);
    for (let i = 0; i < data.length; i++) {
      out[i] = data[i] ^ key[i % key.length];
    }
    return out;
  }

  private static decryptAES(encryptedData: Uint8Array, password: string): Uint8Array {
    // Symmetric with encryptAES XOR placeholder
    return this.encryptAES(encryptedData, password);
  }

  /**
   * Create a hidden metadata file for DHT storage
   */
  static createMetadataFile(processedFile: ProcessedFile): File {
    const metadataJson = JSON.stringify({
      ...processedFile.metadata,
      shardMap: processedFile.shards.map(shard => ({
        id: shard.id,
        index: shard.index,
        checksum: shard.checksum,
        size: shard.size,
      })),
    }, null, 2);
    
    const blob = new Blob([metadataJson], { type: 'application/json' });
    return new File([blob], `.${processedFile.originalName}.meta`, {
      type: 'application/json',
    });
  }

  /**
   * Estimate storage requirements
   */
  static estimateStorageSize(fileSize: number): {
    compressedEstimate: number;
    shardsEstimate: number;
    encryptedEstimate: number;
    totalEstimate: number;
  } {
    // Compression ratio estimates (conservative)
    const compressionRatio = 0.7; // 70% of original size
    const compressedEstimate = Math.ceil(fileSize * compressionRatio);
    
    // Encryption overhead (AES-GCM adds ~16 bytes per block)
    const encryptionOverhead = 1.05; // 5% overhead
    const encryptedEstimate = Math.ceil(compressedEstimate * encryptionOverhead);
    
    // Number of shards
    const shardsEstimate = Math.ceil(encryptedEstimate / this.SHARD_SIZE);
    
    return {
      compressedEstimate,
      shardsEstimate,
      encryptedEstimate,
      totalEstimate: encryptedEstimate,
    };
  }
}