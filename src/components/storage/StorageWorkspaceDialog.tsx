import React, { useEffect, useMemo, useState } from 'react'
import { Dialog, DialogTitle, DialogContent, IconButton, Box, CircularProgress, Alert } from '@mui/material'
import CloseIcon from '@mui/icons-material/Close'
import { WebStorageWorkspace } from '../unified/WebStorageWorkspace'
import { CompleteStorageSystem } from '../../services/storage/CompleteStorageSystem'
import { DHTWebRouter } from '../../services/dht/DHTWebRouter'
import { DHTStorage } from '../../services/storage/dhtStorage'
import { NetworkIdentity, PersonalUser, Organization, Project } from '../../types/collaboration'

export interface StorageEntityRef {
  entityId: string
  entityType: 'personal' | 'organization' | 'project' | 'group' | 'channel'
  entityName?: string
  fourWords?: string
}

interface StorageWorkspaceDialogProps {
  open: boolean
  onClose: () => void
  entity: StorageEntityRef | null
}

export const StorageWorkspaceDialog: React.FC<StorageWorkspaceDialogProps> = ({ open, onClose, entity }) => {
  const [storageSystem, setStorageSystem] = useState<CompleteStorageSystem | null>(null)
  const [dhtRouter, setDHTRouter] = useState<DHTWebRouter | null>(null)
  const [currentUser, setCurrentUser] = useState<NetworkIdentity | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Create or memoize a pseudo-identity when not provided
  const entityIdentity: NetworkIdentity | null = useMemo(() => {
    if (!entity) return null
    const four = entity.fourWords || `${entity.entityType}-${entity.entityId}`
    return {
      fourWords: four,
      publicKey: `pk_${four}`,
      dhtAddress: `dht://${four}`,
    }
  }, [entity])

  useEffect(() => {
    let cancelled = false
    const init = async () => {
      if (!open || !entity || !entityIdentity) return
      setLoading(true)
      setError(null)
      try {
        // Reuse existing system if already initialized
        let sys = storageSystem
        if (!sys) {
          sys = new CompleteStorageSystem({ bootstrapNodes: ['localhost:5001'], replicationFactor: 3 })
          await sys.initialize()
          if (cancelled) return
          setStorageSystem(sys)
        }

        // DHT router
        let router = dhtRouter
        if (!router) {
          const dht = new DHTStorage({ identity: entityIdentity, bootstrapNodes: ['localhost:5001'], replicationFactor: 3 })
          await dht.connect()
          if (cancelled) return
          router = new DHTWebRouter(dht)
          setDHTRouter(router)
        }

        setCurrentUser(entityIdentity)
      } catch (e: any) {
        if (!cancelled) setError(e?.message || 'Failed to initialize storage workspace')
      } finally {
        if (!cancelled) setLoading(false)
      }
    }
    init()
    return () => { cancelled = true }
  }, [open, entity, entityIdentity])

  // Compose a basic entity object for the workspace's initialEntity prop
  const initialEntity: PersonalUser | Organization | Project | null = useMemo(() => {
    if (!entity || !entityIdentity) return null
    const base = {
      id: entity.entityId,
      name: entity.entityName || entityIdentity.fourWords,
      description: undefined as string | undefined,
      avatar: undefined as string | undefined,
      networkIdentity: entityIdentity,
      capabilities: { videoCall: true, audioCall: true, screenShare: true, fileShare: true, websitePublish: true },
      createdAt: new Date(),
      updatedAt: new Date(),
    }
    switch (entity.entityType) {
      case 'organization':
        return { ...base, type: 'organization', owners: [], channels: [], groups: [], users: [], projects: [], settings: { allowGuestAccess: false, defaultChannelPermissions: [], websitePublishingEnabled: true } }
      case 'project':
        return { ...base, type: 'project', organizationId: 'unknown', leads: [], members: [], status: 'active', milestones: [] }
      case 'personal':
      default:
        return { ...base, type: 'personal_user', userId: entity.entityId, relationship: 'contact' }
    }
  }, [entity, entityIdentity])

  return (
    <Dialog open={open} onClose={onClose} maxWidth="lg" fullWidth PaperProps={{ sx: { height: '90vh' } }}>
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        {entity?.entityName || entityIdentity?.fourWords || 'Storage Workspace'}
        <IconButton onClick={onClose} size="small">
          <CloseIcon />
        </IconButton>
      </DialogTitle>
      <DialogContent sx={{ p: 0 }}>
        {loading || !storageSystem || !dhtRouter || !currentUser || !initialEntity ? (
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
            {error ? <Alert severity="error">{error}</Alert> : <CircularProgress />}
          </Box>
        ) : (
          <WebStorageWorkspace
            storageSystem={storageSystem}
            dhtRouter={dhtRouter}
            currentUser={currentUser}
            initialEntity={initialEntity}
            initialFile="home.md"
            className="storage-workspace"
          />
        )}
      </DialogContent>
    </Dialog>
  )
}

export default StorageWorkspaceDialog
