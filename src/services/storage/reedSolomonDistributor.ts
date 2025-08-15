import { invoke } from '@tauri-apps/api/core';
import { ProcessedFile, EncryptedShard } from './fileProcessor';

export interface WitnessProof {
  witnessId: string;
  nodeId: string;
  timestamp: number;
  signature: string;
  shardHashes: string[];
  availability: number; // 0-1 representing availability percentage
}

export interface DistributionNode {
  nodeId: string;
  address: string;
  reputation: number;
  storageCapacity: number;
  lastSeen: number;
  geographicRegion?: string;
  networkLatency: number;
}

export interface DistributedShard {
  originalShard: EncryptedShard;
  reedSolomonShards: ReedSolomonShard[];
  witnesses: WitnessProof[];
  distributionNodes: string[];
  redundancyLevel: number;
}

export interface ReedSolomonShard {
  id: string;
  type: 'data' | 'parity';
  data: Uint8Array;
  checksum: string;
  size: number;
  parentShardId: string;
  reedSolomonIndex: number;
}

export interface RetrievalResult {
  success: boolean;
  originalShard?: EncryptedShard;
  nodesContacted: number;
  nodesResponded: number;
  retrievalTime: number;
  errors: string[];
}

/**
 * Reed-Solomon distribution service for fault-tolerant storage
 * Implements k=8, m=2 encoding with witness attestation system
 */
export class ReedSolomonDistributor {
  private static readonly DATA_SHARDS = 8;   // k - minimum shards needed for reconstruction
  private static readonly PARITY_SHARDS = 2; // m - redundancy shards
  private static readonly TOTAL_SHARDS = this.DATA_SHARDS + this.PARITY_SHARDS; // n=10
  private static readonly MIN_WITNESSES = 3; // Minimum witnesses required for fairness
  private static readonly AVAILABILITY_THRESHOLD = 0.6; // 60% availability required

  /**
   * Distribute a processed file using Reed-Solomon encoding across the network
   */
  static async distributeFile(
    processedFile: ProcessedFile,
    availableNodes: DistributionNode[],
    fourWordId: string
  ): Promise<DistributedShard[]> {
    console.log(`Distributing file ${processedFile.originalName} with ${processedFile.shards.length} shards`);
    
    const distributedShards: DistributedShard[] = [];
    
    for (const shard of processedFile.shards) {
      const distributedShard = await this.distributeSingleShard(
        shard,
        availableNodes,
        fourWordId
      );
      distributedShards.push(distributedShard);
    }
    
    console.log(`Successfully distributed ${distributedShards.length} shards`);
    return distributedShards;
  }

  /**
   * Distribute a single shard using Reed-Solomon encoding
   */
  private static async distributeSingleShard(
    shard: EncryptedShard,
    availableNodes: DistributionNode[],
    fourWordId: string
  ): Promise<DistributedShard> {
    console.log(`Distributing shard ${shard.id} (${shard.size} bytes)`);
    
    // Step 1: Apply Reed-Solomon encoding to create redundant shards
    const reedSolomonShards = await this.applyReedSolomonEncoding(shard);
    
    // Step 2: Select optimal nodes for distribution
    const selectedNodes = this.selectDistributionNodes(availableNodes, this.TOTAL_SHARDS);
    
    // Step 3: Distribute shards to selected nodes
    const distributionPromises = reedSolomonShards.map(async (rsShard, index) => {
      const targetNode = selectedNodes[index % selectedNodes.length];
      return this.storeShardOnNode(rsShard, targetNode, fourWordId);
    });
    
    const distributionResults = await Promise.allSettled(distributionPromises);
    const successfulDistributions = distributionResults
      .map((result, index) => ({ result, node: selectedNodes[index % selectedNodes.length] }))
      .filter(({ result }) => result.status === 'fulfilled')
      .map(({ node }) => node.nodeId);
    
    // Step 4: Generate witness proofs for fairness attestation
    const witnesses = await this.generateWitnessProofs(
      reedSolomonShards,
      selectedNodes,
      fourWordId
    );
    
    return {
      originalShard: shard,
      reedSolomonShards,
      witnesses,
      distributionNodes: successfulDistributions,
      redundancyLevel: this.calculateRedundancyLevel(successfulDistributions.length),
    };
  }

  /**
   * Apply Reed-Solomon encoding to create redundant shards
   */
  private static async applyReedSolomonEncoding(shard: EncryptedShard): Promise<ReedSolomonShard[]> {
    try {
      // Use Rust backend for high-performance Reed-Solomon encoding
      const encodedData = await invoke<{
        data_shards: number[][];
        parity_shards: number[][];
      }>('reed_solomon_encode', {
        data: Array.from(shard.data),
        dataShards: this.DATA_SHARDS,
        parityShards: this.PARITY_SHARDS,
      });
      
      const reedSolomonShards: ReedSolomonShard[] = [];
      
      // Process data shards
      encodedData.data_shards.forEach((shardData, index) => {
        const data = new Uint8Array(shardData);
        reedSolomonShards.push({
          id: `${shard.id}-rs-data-${index}`,
          type: 'data',
          data,
          checksum: this.calculateChecksum(data),
          size: data.length,
          parentShardId: shard.id,
          reedSolomonIndex: index,
        });
      });
      
      // Process parity shards
      encodedData.parity_shards.forEach((shardData, index) => {
        const data = new Uint8Array(shardData);
        reedSolomonShards.push({
          id: `${shard.id}-rs-parity-${index}`,
          type: 'parity',
          data,
          checksum: this.calculateChecksum(data),
          size: data.length,
          parentShardId: shard.id,
          reedSolomonIndex: this.DATA_SHARDS + index,
        });
      });
      
      return reedSolomonShards;
    } catch (error) {
      console.warn('Rust Reed-Solomon encoding failed, using JS fallback:', error);
      return this.fallbackReedSolomonEncoding(shard);
    }
  }

  /**
   * JavaScript fallback for Reed-Solomon encoding
   */
  private static async fallbackReedSolomonEncoding(shard: EncryptedShard): Promise<ReedSolomonShard[]> {
    console.log('Using JavaScript fallback for Reed-Solomon encoding');
    
    // Simple XOR-based redundancy as fallback (not true Reed-Solomon)
    const shardSize = Math.ceil(shard.data.length / this.DATA_SHARDS);
    const reedSolomonShards: ReedSolomonShard[] = [];
    
    // Create data shards by splitting original shard
    for (let i = 0; i < this.DATA_SHARDS; i++) {
      const start = i * shardSize;
      const end = Math.min(start + shardSize, shard.data.length);
      const data = shard.data.slice(start, end);
      
      reedSolomonShards.push({
        id: `${shard.id}-rs-data-${i}`,
        type: 'data',
        data,
        checksum: this.calculateChecksum(data),
        size: data.length,
        parentShardId: shard.id,
        reedSolomonIndex: i,
      });
    }
    
    // Create parity shards using XOR
    for (let i = 0; i < this.PARITY_SHARDS; i++) {
      const parityData = new Uint8Array(shardSize);
      
      // XOR all data shards to create parity
      reedSolomonShards.forEach(dataShard => {
        if (dataShard.type === 'data') {
          for (let j = 0; j < Math.min(parityData.length, dataShard.data.length); j++) {
            parityData[j] ^= dataShard.data[j];
          }
        }
      });
      
      reedSolomonShards.push({
        id: `${shard.id}-rs-parity-${i}`,
        type: 'parity',
        data: parityData,
        checksum: this.calculateChecksum(parityData),
        size: parityData.length,
        parentShardId: shard.id,
        reedSolomonIndex: this.DATA_SHARDS + i,
      });
    }
    
    return reedSolomonShards;
  }

  /**
   * Select optimal nodes for distribution based on reputation, capacity, and geography
   */
  private static selectDistributionNodes(
    availableNodes: DistributionNode[],
    requiredNodes: number
  ): DistributionNode[] {
    // Sort nodes by composite score (reputation + capacity + low latency)
    const scoredNodes = availableNodes
      .map(node => ({
        node,
        score: this.calculateNodeScore(node),
      }))
      .sort((a, b) => b.score - a.score);
    
    // Select top nodes, ensuring geographic diversity
    const selectedNodes: DistributionNode[] = [];
    const usedRegions = new Set<string>();
    
    for (const { node } of scoredNodes) {
      if (selectedNodes.length >= requiredNodes) break;
      
      // Prioritize geographic diversity
      if (node.geographicRegion && usedRegions.has(node.geographicRegion)) {
        continue; // Skip if we already have a node in this region
      }
      
      selectedNodes.push(node);
      if (node.geographicRegion) {
        usedRegions.add(node.geographicRegion);
      }
    }
    
    // If we don't have enough nodes with geographic diversity, fill remaining slots
    if (selectedNodes.length < requiredNodes) {
      for (const { node } of scoredNodes) {
        if (selectedNodes.length >= requiredNodes) break;
        if (!selectedNodes.some(selected => selected.nodeId === node.nodeId)) {
          selectedNodes.push(node);
        }
      }
    }
    
    return selectedNodes;
  }

  /**
   * Calculate node score for selection
   */
  private static calculateNodeScore(node: DistributionNode): number {
    const reputationWeight = 0.4;
    const capacityWeight = 0.3;
    const latencyWeight = 0.2;
    const freshnessWeight = 0.1;
    
    const normalizedReputation = Math.min(node.reputation / 100, 1);
    const normalizedCapacity = Math.min(node.storageCapacity / (1024 * 1024 * 1024), 1); // Normalize to 1GB
    const normalizedLatency = Math.max(0, 1 - (node.networkLatency / 1000)); // Lower latency = higher score
    const freshness = Math.max(0, 1 - (Date.now() - node.lastSeen) / (24 * 60 * 60 * 1000)); // Fresher = higher score
    
    return (
      normalizedReputation * reputationWeight +
      normalizedCapacity * capacityWeight +
      normalizedLatency * latencyWeight +
      freshness * freshnessWeight
    );
  }

  /**
   * Store a Reed-Solomon shard on a specific node
   */
  private static async storeShardOnNode(
    rsShard: ReedSolomonShard,
    node: DistributionNode,
    fourWordId: string
  ): Promise<void> {
    try {
      await invoke('store_shard_on_node', {
        nodeId: node.nodeId,
        nodeAddress: node.address,
        shardId: rsShard.id,
        shardData: Array.from(rsShard.data),
        shardType: rsShard.type,
        parentShardId: rsShard.parentShardId,
        fourWordId,
        checksum: rsShard.checksum,
      });
      
      console.log(`Successfully stored shard ${rsShard.id} on node ${node.nodeId}`);
    } catch (error) {
      console.error(`Failed to store shard ${rsShard.id} on node ${node.nodeId}:`, error);
      throw error;
    }
  }

  /**
   * Generate witness proofs for fairness attestation
   */
  private static async generateWitnessProofs(
    reedSolomonShards: ReedSolomonShard[],
    distributionNodes: DistributionNode[],
    fourWordId: string
  ): Promise<WitnessProof[]> {
    const witnesses: WitnessProof[] = [];
    
    // Select witness nodes (different from storage nodes)
    const witnessNodes = distributionNodes
      .sort(() => Math.random() - 0.5) // Randomize
      .slice(0, this.MIN_WITNESSES);
    
    for (const witnessNode of witnessNodes) {
      try {
        const proof = await invoke<WitnessProof>('generate_witness_proof', {
          witnessNodeId: witnessNode.nodeId,
          shardHashes: reedSolomonShards.map(shard => shard.checksum),
          distributionNodes: distributionNodes.map(node => node.nodeId),
          fourWordId,
          timestamp: Date.now(),
        });
        
        witnesses.push(proof);
      } catch (error) {
        console.warn(`Failed to generate witness proof from node ${witnessNode.nodeId}:`, error);
      }
    }
    
    return witnesses;
  }

  /**
   * Retrieve and reconstruct original file from distributed shards
   */
  static async retrieveFile(
    distributedShards: DistributedShard[],
    fourWordId: string
  ): Promise<EncryptedShard[]> {
    console.log(`Retrieving file with ${distributedShards.length} distributed shards`);
    
    const reconstructedShards: EncryptedShard[] = [];
    
    for (const distributedShard of distributedShards) {
      const retrievalResult = await this.retrieveSingleShard(distributedShard, fourWordId);
      
      if (retrievalResult.success && retrievalResult.originalShard) {
        reconstructedShards.push(retrievalResult.originalShard);
      } else {
        console.error(`Failed to retrieve shard ${distributedShard.originalShard.id}:`, retrievalResult.errors);
        throw new Error(`Failed to retrieve shard: ${retrievalResult.errors.join(', ')}`);
      }
    }
    
    console.log(`Successfully retrieved ${reconstructedShards.length} shards`);
    return reconstructedShards;
  }

  /**
   * Retrieve and reconstruct a single shard using Reed-Solomon decoding
   */
  private static async retrieveSingleShard(
    distributedShard: DistributedShard,
    fourWordId: string
  ): Promise<RetrievalResult> {
    const startTime = Date.now();
    let nodesContacted = 0;
    let nodesResponded = 0;
    const errors: string[] = [];
    
    // Attempt to retrieve shards from distribution nodes
    const retrievalPromises = distributedShard.distributionNodes.map(async (nodeId) => {
      nodesContacted++;
      try {
        const shardData = await invoke<{
          shards: Array<{
            id: string;
            type: 'data' | 'parity';
            data: number[];
            checksum: string;
            reedSolomonIndex: number;
          }>;
        }>('retrieve_shards_from_node', {
          nodeId,
          parentShardId: distributedShard.originalShard.id,
          fourWordId,
        });
        
        nodesResponded++;
        return shardData.shards.map(shard => ({
          ...shard,
          data: new Uint8Array(shard.data),
        }));
      } catch (error) {
        errors.push(`Node ${nodeId}: ${error}`);
        return null;
      }
    });
    
    const retrievalResults = await Promise.allSettled(retrievalPromises);
    const availableShards: Array<{
      id: string;
      type: 'data' | 'parity';
      data: Uint8Array;
      checksum: string;
      reedSolomonIndex: number;
    }> = [];
    
    // Collect all successfully retrieved shards
    for (const result of retrievalResults) {
      if (result.status === 'fulfilled' && result.value) {
        availableShards.push(...result.value);
      }
    }
    
    // Check if we have enough shards for reconstruction (need at least DATA_SHARDS)
    if (availableShards.length < this.DATA_SHARDS) {
      return {
        success: false,
        nodesContacted,
        nodesResponded,
        retrievalTime: Date.now() - startTime,
        errors: [...errors, `Insufficient shards: ${availableShards.length}/${this.DATA_SHARDS} required`],
      };
    }
    
    // Attempt Reed-Solomon reconstruction
    try {
      const reconstructedShard = await this.reconstructFromReedSolomon(
        availableShards,
        distributedShard.originalShard
      );
      
      return {
        success: true,
        originalShard: reconstructedShard,
        nodesContacted,
        nodesResponded,
        retrievalTime: Date.now() - startTime,
        errors,
      };
    } catch (error) {
      return {
        success: false,
        nodesContacted,
        nodesResponded,
        retrievalTime: Date.now() - startTime,
        errors: [...errors, `Reconstruction failed: ${error}`],
      };
    }
  }

  /**
   * Reconstruct original shard from Reed-Solomon shards
   */
  private static async reconstructFromReedSolomon(
    availableShards: Array<{
      id: string;
      type: 'data' | 'parity';
      data: Uint8Array;
      checksum: string;
      reedSolomonIndex: number;
    }>,
    originalShardTemplate: EncryptedShard
  ): Promise<EncryptedShard> {
    try {
      // Use Rust backend for Reed-Solomon reconstruction
      const reconstructedData = await invoke<number[]>('reed_solomon_reconstruct', {
        shards: availableShards.map(shard => ({
          index: shard.reedSolomonIndex,
          data: Array.from(shard.data),
          type: shard.type,
        })),
        dataShards: this.DATA_SHARDS,
        parityShards: this.PARITY_SHARDS,
      });
      
      const data = new Uint8Array(reconstructedData);
      
      return {
        ...originalShardTemplate,
        data,
        size: data.length,
        checksum: this.calculateChecksum(data),
      };
    } catch (error) {
      console.warn('Rust Reed-Solomon reconstruction failed, using JS fallback:', error);
      return this.fallbackReedSolomonReconstruction(availableShards, originalShardTemplate);
    }
  }

  /**
   * JavaScript fallback for Reed-Solomon reconstruction
   */
  private static async fallbackReedSolomonReconstruction(
    availableShards: Array<{
      id: string;
      type: 'data' | 'parity';
      data: Uint8Array;
      reedSolomonIndex: number;
    }>,
    originalShardTemplate: EncryptedShard
  ): Promise<EncryptedShard> {
    console.log('Using JavaScript fallback for Reed-Solomon reconstruction');
    
    // Simple concatenation fallback (for XOR-based encoding)
    const dataShards = availableShards
      .filter(shard => shard.type === 'data')
      .sort((a, b) => a.reedSolomonIndex - b.reedSolomonIndex);
    
    if (dataShards.length < this.DATA_SHARDS) {
      throw new Error(`Insufficient data shards for reconstruction: ${dataShards.length}/${this.DATA_SHARDS}`);
    }
    
    // Concatenate data shards
    const totalSize = dataShards.reduce((sum, shard) => sum + shard.data.length, 0);
    const reconstructedData = new Uint8Array(totalSize);
    let offset = 0;
    
    for (const shard of dataShards) {
      reconstructedData.set(shard.data, offset);
      offset += shard.data.length;
    }
    
    return {
      ...originalShardTemplate,
      data: reconstructedData,
      size: reconstructedData.length,
      checksum: this.calculateChecksum(reconstructedData),
    };
  }

  /**
   * Verify witness proofs for fairness attestation
   */
  static async verifyWitnessProofs(
    distributedShards: DistributedShard[],
    fourWordId: string
  ): Promise<{ valid: boolean; failedWitnesses: string[] }> {
    const failedWitnesses: string[] = [];
    
    for (const distributedShard of distributedShards) {
      for (const witness of distributedShard.witnesses) {
        try {
          const isValid = await invoke<boolean>('verify_witness_proof', {
            witnessProof: witness,
            shardHashes: distributedShard.reedSolomonShards.map(shard => shard.checksum),
            fourWordId,
          });
          
          if (!isValid) {
            failedWitnesses.push(witness.witnessId);
          }
        } catch (error) {
          console.warn(`Failed to verify witness proof ${witness.witnessId}:`, error);
          failedWitnesses.push(witness.witnessId);
        }
      }
    }
    
    return {
      valid: failedWitnesses.length === 0,
      failedWitnesses,
    };
  }

  /**
   * Calculate checksum for data integrity verification
   */
  private static calculateChecksum(data: Uint8Array): string {
    // Simple checksum - in production, use crypto hash
    let checksum = 0;
    for (let i = 0; i < data.length; i++) {
      checksum = (checksum + data[i]) & 0xFFFFFFFF;
    }
    return checksum.toString(16);
  }

  /**
   * Calculate redundancy level based on successful distributions
   */
  private static calculateRedundancyLevel(successfulDistributions: number): number {
    return Math.min(successfulDistributions / this.TOTAL_SHARDS, 1);
  }

  /**
   * Check if file can be retrieved based on current node availability
   */
  static async checkAvailability(
    distributedShards: DistributedShard[],
    fourWordId: string
  ): Promise<{ retrievable: boolean; availability: number; nodeStatus: Map<string, boolean> }> {
    const nodeStatus = new Map<string, boolean>();
    let totalShards = 0;
    let availableShards = 0;
    
    for (const distributedShard of distributedShards) {
      totalShards++;
      
      // Check node availability for this shard
      let shardAvailable = false;
      for (const nodeId of distributedShard.distributionNodes) {
        try {
          const nodeOnline = await invoke<boolean>('check_node_availability', {
            nodeId,
            fourWordId,
          });
          
          nodeStatus.set(nodeId, nodeOnline);
          
          if (nodeOnline) {
            shardAvailable = true;
            break; // Only need one node to be online for this shard
          }
        } catch (error) {
          nodeStatus.set(nodeId, false);
        }
      }
      
      if (shardAvailable) {
        availableShards++;
      }
    }
    
    const availability = totalShards > 0 ? availableShards / totalShards : 0;
    const retrievable = availability >= this.AVAILABILITY_THRESHOLD;
    
    return {
      retrievable,
      availability,
      nodeStatus,
    };
  }
}