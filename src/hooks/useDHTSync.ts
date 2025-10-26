import { invoke } from '@tauri-apps/api/core';
import { Event, listen, UnlistenFn } from '@tauri-apps/api/event';
import { useSnackbar } from 'notistack';
import { useCallback, useEffect, useRef, useState } from 'react';

const isTauriEnvironment = () =>
  typeof window !== 'undefined' && Boolean((window as any)._TAURI_);

// DHT Event Types
export interface DHTSyncEvent {
  type: 
    | 'OrganizationCreated' | 'OrganizationUpdated' | 'OrganizationDeleted'
    | 'GroupCreated' | 'GroupUpdated' | 'GroupDeleted'
    | 'ProjectCreated' | 'ProjectUpdated' | 'ProjectDeleted'
    | 'MemberJoined' | 'MemberLeft' | 'MemberRoleChanged'
    | 'FileUploaded' | 'FileDeleted' | 'FileShared'
    | 'PeerConnected' | 'PeerDisconnected' | 'NetworkStatusChanged';
  
  // Event-specific data
  organization?: any;
  group?: any;
  project?: any;
  id?: string;
  entity_type?: string;
  entity_id?: string;
  member?: any;
  user_id?: string;
  new_role?: string;
  project_id?: string;
  file?: any;
  file_id?: string;
  shared_with?: string[];
  peer_id?: string;
  address?: string;
  status?: NetworkStatus;
}

export interface NetworkStatus {
  connected: boolean;
  peer_count: number;
  syncing: boolean;
  last_sync?: string;
}

export interface UseDHTSyncOptions {
  userId: string;
  entityIds?: string[];
  onEvent?: (event: DHTSyncEvent) => void;
  autoReconnect?: boolean;
  reconnectInterval?: number;
}

export interface DHTSyncState {
  connected: boolean;
  syncing: boolean;
  lastSync?: Date;
  peerCount: number;
  subscribedEntities: string[];
  pendingEvents: DHTSyncEvent[];
}

/**
 * Hook for real-time DHT synchronization
 */
export const useDHTSync = ({
  userId,
  entityIds = [],
  onEvent,
  autoReconnect = true,
  reconnectInterval = 5000,
}: UseDHTSyncOptions) => {
  const { enqueueSnackbar } = useSnackbar();
  const [state, setState] = useState<DHTSyncState>({
    connected: false,
    syncing: false,
    peerCount: 0,
    subscribedEntities: [],
    pendingEvents: [],
  });

  const unlistenersRef = useRef<UnlistenFn[]>([]);
  const reconnectTimerRef = useRef<NodeJS.Timeout>();
  const subscribedEntitiesRef = useRef<Set<string>>(new Set());
  const stateRef = useRef(state);

  // Keep stateRef in sync with state
  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  // Subscribe to an entity for real-time updates
  const subscribeToEntity = useCallback(async (entityId: string) => {
    if (!isTauriEnvironment()) {
      console.debug('[DHT] Skipping subscribeToEntity outside Tauri runtime');
      return;
    }

    if (subscribedEntitiesRef.current.has(entityId)) {
      return; // Already subscribed
    }

    // Check CURRENT connection state (not stale closure)
    if (!stateRef.current.connected) {
      console.debug(`[DHT] Skipping subscription to ${entityId} - not connected`);
      return;
    }

    try {
      // Use working gossip_subscribe_to_entity command
      // Note: entity_type should be derived from context (channel/project/group)
      await invoke('gossip_subscribe_to_entity', {
        entity_id: entityId,
        entity_type: 'channel', // Default to 'channel' - may need dynamic type detection
      });

      subscribedEntitiesRef.current.add(entityId);
      setState(prev => ({
        ...prev,
        subscribedEntities: Array.from(subscribedEntitiesRef.current),
      }));

      console.log(`Subscribed to entity: ${entityId}`);
    } catch (error) {
      console.error(`Failed to subscribe to entity ${entityId}:`, error);
      // Don't show error notifications - subscription failures are expected in offline mode
      // and will be handled by the backend gracefully
    }
  }, [userId, enqueueSnackbar]);

  // Unsubscribe from an entity
  const unsubscribeFromEntity = useCallback(async (entityId: string) => {
    if (!isTauriEnvironment()) {
      console.debug('[DHT] Skipping unsubscribeFromEntity outside Tauri runtime');
      return;
    }

    if (!subscribedEntitiesRef.current.has(entityId)) {
      return; // Not subscribed
    }

    try {
      // Use working gossip_leave_entity command
      await invoke('gossip_leave_entity', {
        entity_id: entityId,
      });

      subscribedEntitiesRef.current.delete(entityId);
      setState(prev => ({
        ...prev,
        subscribedEntities: Array.from(subscribedEntitiesRef.current),
      }));

      console.log(`Unsubscribed from entity: ${entityId}`);
    } catch (error) {
      console.error(`Failed to unsubscribe from entity ${entityId}:`, error);
    }
  }, [userId]);

  // Process incoming DHT event
  const processEvent = useCallback((event: DHTSyncEvent) => {
    console.log('Processing DHT event:', event);

    // Update state based on event type
    switch (event.type) {
      case 'NetworkStatusChanged':
        setState(prev => ({
          ...prev,
          connected: event.status?.connected || false,
          syncing: event.status?.syncing || false,
          peerCount: event.status?.peer_count || 0,
          lastSync: event.status?.last_sync ? new Date(event.status.last_sync) : undefined,
        }));
        break;

      case 'PeerConnected':
        setState(prev => ({
          ...prev,
          peerCount: prev.peerCount + 1,
        }));
        enqueueSnackbar(`Peer connected: ${event.address}`, {
          variant: 'info',
          autoHideDuration: 3000,
        });
        break;

      case 'PeerDisconnected':
        setState(prev => ({
          ...prev,
          peerCount: Math.max(0, prev.peerCount - 1),
        }));
        break;

      default:
        // Add to pending events for processing by parent component
        setState(prev => ({
          ...prev,
          pendingEvents: [...prev.pendingEvents, event],
        }));
    }

    // Call custom event handler
    if (onEvent) {
      onEvent(event);
    }
  }, [onEvent, enqueueSnackbar]);

  // Clear pending events
  const clearPendingEvents = useCallback(() => {
    setState(prev => ({
      ...prev,
      pendingEvents: [],
    }));
  }, []);

  // Check sync status
  const checkSyncStatus = useCallback(async () => {
    try {
      if (!isTauriEnvironment()) {
        return;
      }

      const status: NetworkStatus = await invoke('get_sync_status');
      setState(prev => ({
        ...prev,
        connected: status.connected,
        syncing: status.syncing,
        peerCount: status.peer_count,
        lastSync: status.last_sync ? new Date(status.last_sync) : undefined,
      }));
    } catch (error) {
      console.error('Failed to get sync status:', error);
      setState(prev => ({
        ...prev,
        connected: false,
        syncing: false,
      }));
    }
  }, []);

  // Setup event listeners
  useEffect(() => {
    if (!isTauriEnvironment()) {
      console.debug('[DHT] Sync disabled: not running inside Tauri runtime');
      return;
    }

    const setupListeners = async () => {
      try {
        // Listen for global DHT events
        const globalUnlisten = await listen<DHTSyncEvent>('dht-sync-event', (event: Event<DHTSyncEvent>) => {
          processEvent(event.payload);
        });
        unlistenersRef.current.push(globalUnlisten);

        // Listen for user-specific events
        const userUnlisten = await listen<DHTSyncEvent>(`dht-sync-event:${userId}`, (event: Event<DHTSyncEvent>) => {
          processEvent(event.payload);
        });
        unlistenersRef.current.push(userUnlisten);

        console.log('DHT sync listeners setup complete');
      } catch (error) {
        console.error('Failed to setup DHT sync listeners:', error);
      }
    };

    setupListeners();

    // Initial status check (do this first to determine if we're connected)
    checkSyncStatus().then(() => {
      // Only subscribe to initial entities if we're connected
      // The subscription will be handled by the effect below once connection is established
    });

    // Setup periodic status check
    const statusInterval = setInterval(checkSyncStatus, 30000); // Check every 30 seconds

    // Cleanup
    return () => {
      clearInterval(statusInterval);
      
      // Unsubscribe from all entities
      subscribedEntitiesRef.current.forEach(entityId => {
        unsubscribeFromEntity(entityId);
      });

      // Remove event listeners
      unlistenersRef.current.forEach(unlisten => unlisten());
      unlistenersRef.current = [];

      // Clear reconnect timer
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
      }
    };
  }, [userId]);

  // Handle entity subscription changes (only when connected)
  useEffect(() => {
    // Skip if not connected
    if (!state.connected) {
      console.debug('[DHT] Skipping entity subscriptions - not connected');
      return;
    }

    const currentEntities = new Set(entityIds);
    const subscribedEntities = subscribedEntitiesRef.current;

    // Subscribe to new entities
    currentEntities.forEach(entityId => {
      if (!subscribedEntities.has(entityId)) {
        subscribeToEntity(entityId);
      }
    });

    // Unsubscribe from removed entities
    subscribedEntities.forEach(entityId => {
      if (!currentEntities.has(entityId)) {
        unsubscribeFromEntity(entityId);
      }
    });
  }, [entityIds, state.connected, subscribeToEntity, unsubscribeFromEntity]);

  // Auto-reconnect logic
  useEffect(() => {
    if (!isTauriEnvironment()) {
      return;
    }

    if (!state.connected && autoReconnect) {
      console.log('Connection lost, attempting to reconnect...');
      
      reconnectTimerRef.current = setTimeout(() => {
        checkSyncStatus();
      }, reconnectInterval);
    }

    return () => {
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
      }
    };
  }, [state.connected, autoReconnect, reconnectInterval, checkSyncStatus]);

  return {
    ...state,
    subscribeToEntity,
    unsubscribeFromEntity,
    clearPendingEvents,
    checkSyncStatus,
  };
};

/**
 * Hook for organization-specific DHT sync
 */
export const useOrganizationSync = (organizationId: string, userId: string) => {
  const [organization, setOrganization] = useState<any>(null);
  const [groups, setGroups] = useState<any[]>([]);
  const [projects, setProjects] = useState<any[]>([]);

  const handleEvent = useCallback((event: DHTSyncEvent) => {
    switch (event.type) {
      case 'OrganizationUpdated':
        if (event.organization?.id === organizationId) {
          setOrganization(event.organization);
        }
        break;

      case 'GroupCreated':
        if (event.group?.organization_id === organizationId) {
          setGroups(prev => [...prev, event.group]);
        }
        break;

      case 'GroupUpdated':
        if (event.group?.organization_id === organizationId) {
          setGroups(prev => prev.map(g => 
            g.id === event.group.id ? event.group : g
          ));
        }
        break;

      case 'GroupDeleted':
        setGroups(prev => prev.filter(g => g.id !== event.id));
        break;

      case 'ProjectCreated':
        if (event.project?.organization_id === organizationId) {
          setProjects(prev => [...prev, event.project]);
        }
        break;

      case 'ProjectUpdated':
        if (event.project?.organization_id === organizationId) {
          setProjects(prev => prev.map(p => 
            p.id === event.project.id ? event.project : p
          ));
        }
        break;

      case 'ProjectDeleted':
        setProjects(prev => prev.filter(p => p.id !== event.id));
        break;
    }
  }, [organizationId]);

  const sync = useDHTSync({
    userId,
    entityIds: [organizationId],
    onEvent: handleEvent,
  });

  return {
    ...sync,
    organization,
    groups,
    projects,
  };
};

export default useDHTSync;
