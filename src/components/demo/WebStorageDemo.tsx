import React, { useState, useEffect } from 'react'
import { Box, Typography, Button, Alert, CircularProgress } from '@mui/material'
import { WebStorageWorkspace } from '../unified/WebStorageWorkspace'
import { CompleteStorageSystem } from '../../services/storage/CompleteStorageSystem'
import { DHTWebRouter } from '../../services/dht/DHTWebRouter'
import { DHTStorage } from '../../services/storage/dhtStorage'
import { NetworkIdentity, PersonalUser } from '../../types/collaboration'

/**
 * Demonstration component showcasing the world-class web storage system
 * Features:
 * - Real-time collaborative editing with Monaco Editor
 * - Distributed storage with Reed-Solomon encoding
 * - DHT routing with home.md as entry point
 * - End-to-end encryption
 * - Four-word human-readable addresses
 */
export const WebStorageDemo: React.FC = () => {
  const [storageSystem, setStorageSystem] = useState<CompleteStorageSystem | null>(null)
  const [dhtRouter, setDHTRouter] = useState<DHTWebRouter | null>(null)
  const [currentUser, setCurrentUser] = useState<PersonalUser | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Initialize the demo system
  useEffect(() => {
    const initializeDemo = async () => {
      try {
        setLoading(true)

        // Create storage system
        const storage = new CompleteStorageSystem({
          bootstrapNodes: ['localhost:5001', 'localhost:5002'],
          replicationFactor: 3
        })
        
        await storage.initialize()
        setStorageSystem(storage)

        // Create DHT router
        const dht = new DHTStorage({
          identity: {
            fourWords: 'demo-system-node-one',
            publicKey: 'pk_demo_system',
            dhtAddress: 'dht://demo-system-node-one'
          },
          bootstrapNodes: ['localhost:5001'],
          replicationFactor: 3
        })
        
        await dht.connect()
        
        const router = new DHTWebRouter(dht)
        setDHTRouter(router)

        // Create demo user
        const user = await storage.createUser({
          name: 'Demo User',
          email: 'demo@communitas.network'
        })
        
        setCurrentUser(user)

        // Set up demo content
        const userStorage = await storage.getPersonalStorage(user.id)
        
        // Create home.md if it doesn't exist
        try {
          await userStorage.readFile('/web/home.md')
        } catch {
          await userStorage.createFile('/web/home.md', `# Welcome to ${user.networkIdentity.fourWords}

This is your personal space on the Communitas P2P network! 

## Your Network Identity
- **Four-word address**: ${user.networkIdentity.fourWords}
- **DHT address**: ${user.networkIdentity.dhtAddress}

## Features Demonstrated

### 🎯 World-Class Editor
- **Monaco Editor** with full syntax highlighting
- **Real-time collaboration** with cursor tracking
- **Live preview** with instant markdown rendering
- **Auto-save** with visual save indicators

### 🌐 Distributed Storage
- **Reed-Solomon encoding** with 60% redundancy (10+6)
- **AES-256-GCM encryption** for all content
- **DHT distribution** across peer nodes
- **Self-healing** when nodes go offline

### 🔗 Human-Readable Addressing
- **Four-word addresses** instead of cryptographic hashes
- **Automatic home.md routing** for website entry points
- **Cross-entity linking** between different identities
- **DNS-free navigation** through the DHT

### 👥 Real-Time Collaboration
- **Yjs CRDT** for conflict-free collaborative editing
- **WebRTC connections** for peer-to-peer sync
- **User awareness** with colored cursors and selections
- **Offline-first** with automatic sync when reconnected

## Try It Out!

1. **Switch to Editor tab** to edit this content
2. **Make changes** and watch them auto-save
3. **Switch to Browser tab** to see the rendered website
4. **Publish to DHT** to make it available network-wide

---

*This demo showcases the Minimum Viable Best (MVB) quality implementation of a fully decentralized, collaborative web storage system.*
`)
        }

        setLoading(false)
      } catch (err) {
        console.error('Demo initialization failed:', err)
        setError(err instanceof Error ? err.message : 'Initialization failed')
        setLoading(false)
      }
    }

    initializeDemo()
  }, [])

  if (loading) {
    return (
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100vh',
          gap: 2
        }}
      >
        <CircularProgress size={60} />
        <Typography variant="h6">Initializing Web Storage System...</Typography>
        <Typography variant="body2" color="text.secondary">
          Setting up distributed storage, DHT routing, and collaboration...
        </Typography>
      </Box>
    )
  }

  if (error) {
    return (
      <Box sx={{ p: 3 }}>
        <Alert severity="error" sx={{ mb: 2 }}>
          <Typography variant="h6">Demo Initialization Failed</Typography>
          <Typography>{error}</Typography>
        </Alert>
        <Button 
          variant="contained" 
          onClick={() => window.location.reload()}
        >
          Retry
        </Button>
      </Box>
    )
  }

  if (!storageSystem || !dhtRouter || !currentUser) {
    return (
      <Alert severity="warning">
        System not properly initialized. Please refresh the page.
      </Alert>
    )
  }

  return (
    <Box sx={{ height: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider', background: 'primary.main', color: 'white' }}>
        <Typography variant="h4" component="h1">
          Communitas Web Storage System
        </Typography>
        <Typography variant="subtitle1">
          Minimum Viable Best (MVB) Implementation Demo
        </Typography>
        <Box sx={{ mt: 1, display: 'flex', gap: 2, flexWrap: 'wrap' }}>
          <Typography variant="caption" sx={{ background: 'rgba(255,255,255,0.2)', px: 1, borderRadius: 1 }}>
            ✅ Real-time Collaboration
          </Typography>
          <Typography variant="caption" sx={{ background: 'rgba(255,255,255,0.2)', px: 1, borderRadius: 1 }}>
            ✅ Distributed Storage (Reed-Solomon 10+6)
          </Typography>
          <Typography variant="caption" sx={{ background: 'rgba(255,255,255,0.2)', px: 1, borderRadius: 1 }}>
            ✅ End-to-End Encryption
          </Typography>
          <Typography variant="caption" sx={{ background: 'rgba(255,255,255,0.2)', px: 1, borderRadius: 1 }}>
            ✅ Four-Word Addresses
          </Typography>
          <Typography variant="caption" sx={{ background: 'rgba(255,255,255,0.2)', px: 1, borderRadius: 1 }}>
            ✅ home.md DHT Routing
          </Typography>
        </Box>
      </Box>

      {/* Main Workspace */}
      <Box sx={{ flex: 1 }}>
        <WebStorageWorkspace
          storageSystem={storageSystem}
          dhtRouter={dhtRouter}
          currentUser={currentUser.networkIdentity}
          initialEntity={currentUser}
          initialFile="home.md"
        />
      </Box>
    </Box>
  )
}