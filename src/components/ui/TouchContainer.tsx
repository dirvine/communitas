import {
    CloudDone as OnlineIcon, CloudOff as OfflineIcon, CloudSync as SyncIcon, Group as GroupIcon, Refresh as RefreshIcon, Storage as StorageIcon
} from '@mui/icons-material';
import {
    alpha, Box,
    BoxProps, Chip, CircularProgress, IconButton, SxProps,
    Theme, Tooltip, Typography, useTheme
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import React, { useCallback, useEffect, useRef, useState } from 'react';

// Auth context
import { useAuth } from '../../contexts/AuthContext';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

// Saorsa-Core integration hooks
import { DHTSyncEvent, useDHTSync } from '../../hooks/useDHTSync';
import { useFileStorage, useMarkdownStorage, useSaorsaStorage } from '../../hooks/useSaorsaStorage';

// Saorsa-Core types
import {
    StorageAddress,
    StorageEngineStats, StoragePolicy
} from '../../types/saorsa-storage';

export interface TouchContainerProps extends Omit<BoxProps, 'onScroll'> {
   /** Enable pull-to-refresh */
   enablePullToRefresh?: boolean;
   /** Pull-to-refresh threshold */
   pullThreshold?: number;
   /** Enable bounce scroll */
   enableBounceScroll?: boolean;
   /** Enable smooth scrolling */
   enableSmoothScroll?: boolean;
   /** Enable touch-friendly scrolling */
   enableTouchScroll?: boolean;
   /** Enable haptic feedback */
   hapticFeedback?: boolean;
   /** Custom haptic duration */
   hapticDuration?: number;
   /** Container height */
   height?: number | string;
   /** Container max height */
   maxHeight?: number | string;
   /** Enable sticky header */
   enableStickyHeader?: boolean;
   /** Sticky header height */
   stickyHeaderHeight?: number;
   /** Enable sticky footer */
   enableStickyFooter?: boolean;
   /** Sticky footer height */
   stickyFooterHeight?: number;
   /** Enable overscroll glow */
   enableOverscrollGlow?: boolean;
   /** Custom overscroll color */
   overscrollColor?: string;
   /** Pull-to-refresh handler */
   onPullToRefresh?: () => void;
   /** Scroll handler */
   onScroll?: (event: React.UIEvent<HTMLDivElement>) => void;
   /** Pull start handler */
   onPullStart?: () => void;
   /** Pull end handler */
   onPullEnd?: () => void;
   /** Reach top handler */
   onReachTop?: () => void;
   /** Reach bottom handler */
   onReachBottom?: () => void;
   /** Custom sx styles */
   sx?: SxProps<Theme>;

   // Saorsa-Core Integration
   /** Enable Saorsa-Core storage integration */
   enableStorage?: boolean;
   /** User ID for storage operations */
   userId?: string;
   /** Default storage policy */
   defaultStoragePolicy?: StoragePolicy;
   /** Enable real-time collaboration */
   enableCollaboration?: boolean;
   /** Enable offline-first mode */
   enableOfflineMode?: boolean;
   /** Auto-sync content changes */
   autoSync?: boolean;
   /** Show storage status indicators */
   showStorageStatus?: boolean;
   /** Show sync status indicators */
   showSyncStatus?: boolean;
   /** Storage event handlers */
   onStorageEvent?: (event: { type: string; data: any }) => void;
   onSyncEvent?: (event: DHTSyncEvent) => void;
   onContentStored?: (address: StorageAddress) => void;
   onContentRetrieved?: (content: string, metadata: any) => void;
 }

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Storage methods interface
export interface TouchContainerStorageMethods {
  storeContent: (content: string, contentType?: string) => Promise<StorageAddress | null>;
  retrieveContent: (address: StorageAddress) => Promise<string | null>;
  refreshContent: () => Promise<void>;
  syncStatus: 'online' | 'offline' | 'syncing';
  storageStats: StorageEngineStats | null;
  dhtSync: any;
}

// Enhanced touch-friendly container component with Saorsa-Core integration
export const TouchContainer: React.FC<TouchContainerProps> = ({
   enablePullToRefresh = false,
   pullThreshold = 80,
   enableBounceScroll = true,
   enableSmoothScroll = true,
   enableTouchScroll = true,
   hapticFeedback = false,
   hapticDuration = 50,
   height = '100%',
   maxHeight,
   enableStickyHeader = false,
   stickyHeaderHeight = 60,
   enableStickyFooter = false,
   stickyFooterHeight = 60,
   enableOverscrollGlow = true,
   overscrollColor,
   onPullToRefresh,
   onScroll,
   onPullStart,
   onPullEnd,
   onReachTop,
   onReachBottom,
   sx,
   children,
   // Saorsa-Core integration props
   enableStorage = false,
   userId,
   defaultStoragePolicy = 'PrivateMax',
   enableCollaboration = false,
   enableOfflineMode = true,
   autoSync = true,
   showStorageStatus = true,
   showSyncStatus = true,
   onStorageEvent,
   onSyncEvent,
   onContentStored,
   onContentRetrieved,
   ...props
 }) => {
   const theme = useTheme();
   const isTouch = useTouchDevice();
   const _touchSizing = useTouchFriendlySizing();
   const { authState } = useAuth();
   const containerRef = useRef<HTMLDivElement>(null);
   const [isPulling, setIsPulling] = useState(false);
   const [pullDistance, setPullDistance] = useState(0);
   const [_isRefreshing, setIsRefreshing] = useState(false);
   const [scrollTop, setScrollTop] = useState(0);

   // Get current user ID (prefer prop, fallback to auth)
   const currentUserId = userId || authState.user?.id;

   // Saorsa-Core integration state
   const [_contentCache, _setContentCache] = useState<Map<string, any>>(new Map());
   const [syncStatus, setSyncStatus] = useState<'online' | 'offline' | 'syncing'>('offline');
   const [storageStats, setStorageStats] = useState<StorageEngineStats | null>(null);
   const [pendingChanges, _setPendingChanges] = useState<any[]>([]);

   // Collaboration state
   const [_collaborators, setCollaborators] = useState<Map<string, any>>(new Map());
   const [_liveCursors, _setLiveCursors] = useState<Map<string, any>>(new Map());
   const [activeCollaborators, setActiveCollaborators] = useState<string[]>([]);
   const [collaborationEvents, setCollaborationEvents] = useState<any[]>([]);

   // Networking state
   const [_networkPeers, setNetworkPeers] = useState<Map<string, any>>(new Map());
   const [quicConnections, setQuicConnections] = useState<Map<string, any>>(new Map());
   const [_networkLatency, setNetworkLatency] = useState<Map<string, number>>(new Map());
   const [_touchGestures, setTouchGestures] = useState<any[]>([]);

   // Initialize Saorsa-Core hooks
   const storage = useSaorsaStorage();
   const markdownStorage = useMarkdownStorage();
   const _fileStorage = useFileStorage();

   const dhtSync = useDHTSync({
     userId: userId || '',
     entityIds: enableCollaboration ? ['touch-container'] : [],
     onEvent: (event) => {
       onSyncEvent?.(event);
       handleSyncEvent(event);
     },
     autoReconnect: true,
   });

   // Saorsa-Core event handlers
   const handleSyncEvent = useCallback((event: DHTSyncEvent) => {
     switch (event.type) {
       case 'NetworkStatusChanged':
         setSyncStatus(event.status?.connected ? 'online' : 'offline');
         break;
       case 'PeerConnected':
       setSyncStatus('online');
       // Add new collaborator
       if (event.peer_id !== undefined && event.address !== undefined) {
       setCollaborators(prev => new Map(prev).set(event.peer_id!, {
       id: event.peer_id!,
       address: event.address!,
       connected: true,
       lastSeen: new Date(),
       }));
       }
       break;
       case 'PeerDisconnected':
       setSyncStatus('offline');
       // Mark collaborator as disconnected
       if (event.peer_id !== undefined) {
       setCollaborators(prev => {
       const updated = new Map(prev);
       const collaborator = updated.get(event.peer_id!);
       if (collaborator) {
       updated.set(event.peer_id!, { ...collaborator, connected: false });
       }
       return updated;
       });
       }
       break;
       case 'FileUploaded':
       case 'FileShared':
         // Handle collaborative content updates
         if (event.file && autoSync) {
           refreshContent();
           setCollaborationEvents(prev => [...prev, {
             type: 'content-updated',
             data: event.file,
             timestamp: new Date(),
           }]);
         }
         break;
       case 'MemberJoined':
       // Handle new collaborator joining
       if (event.member && event.user_id !== undefined) {
       setActiveCollaborators(prev => [...prev, event.user_id!]);
       setCollaborationEvents(prev => [...prev, {
       type: 'member-joined',
       data: event.member,
       timestamp: new Date(),
       }]);
       }
       break;
       case 'MemberLeft':
       // Handle collaborator leaving
       if (event.user_id !== undefined) {
       setActiveCollaborators(prev => prev.filter(id => id !== event.user_id!));
       setCollaborationEvents(prev => [...prev, {
       type: 'member-left',
       data: { user_id: event.user_id! },
       timestamp: new Date(),
       }]);
       }
       break;
       case 'NetworkStatusChanged':
         // Handle network status changes
         if (event.status) {
           setNetworkPeers(prev => {
             const updated = new Map(prev);
             // Update peer connection status
             for (const [peerId, peer] of updated) {
               updated.set(peerId, { ...peer, connected: event.status?.connected || false });
             }
             return updated;
           });
         }
         break;
     }
   }, [autoSync]);

   // Storage operations with collaboration
   const _storeContent = useCallback(async (content: string, _contentType: string = 'text/markdown') => {
     if (!enableStorage || !userId) return null;

     try {
       const result = await markdownStorage.storeMarkdown(
         content,
         defaultStoragePolicy,
         'TouchContainer',
         userId,
         [`touch-container`, `timestamp:${Date.now()}`, `collaborative:${enableCollaboration}`]
       );

       onContentStored?.(result.address);
       onStorageEvent?.({ type: 'content-stored', data: result });

       // Broadcast collaborative update
       if (enableCollaboration && dhtSync.connected) {
         await broadcastCollaborativeUpdate({
           type: 'content-updated',
           contentId: result.address.content_id,
           userId,
           timestamp: new Date(),
         });
       }

       if (hapticFeedback && isTouch) {
         triggerHapticFeedback(hapticDuration);
       }

       return result.address;
     } catch (error) {
       console.error('Failed to store content:', error);
       onStorageEvent?.({ type: 'storage-error', data: error });
       return null;
     }
   }, [enableStorage, userId, markdownStorage, defaultStoragePolicy, onContentStored, onStorageEvent, hapticFeedback, isTouch, hapticDuration, enableCollaboration, dhtSync]);

   // Collaborative messaging functions
   const broadcastCollaborativeUpdate = useCallback(async (update: any) => {
     if (!enableCollaboration || !userId) return;

     try {
       // Use working send_message command from org_commands.rs
      await invoke('send_message', {
        request: {
          channel_id: 'touch-collaboration',
          author_id: currentUserId!,
          content: JSON.stringify(update),
          thread_id: null,
        },
      });
     } catch (error) {
       console.error('Failed to broadcast collaborative update:', error);
     }
   }, [enableCollaboration, currentUserId]);

   const sendCollaborativeMessage = useCallback(async (message: string, recipientIds?: string[]) => {
     if (!enableCollaboration || !currentUserId) return;

     try {
       if (recipientIds && recipientIds.length > 0) {
         // Send to specific recipients using gossip_send_direct_message
         // Note: gossip commands expect four-word addresses, not IDs
         // TODO: Convert recipientIds to four-word addresses if needed
         for (const recipientFourWords of recipientIds) {
           await invoke('gossip_send_direct_message', {
             recipient_four_words: recipientFourWords,
             content: message,
             encrypted: true,
           });
         }
       } else {
         // Send to collaboration channel using working send_message command
         await invoke('send_message', {
           request: {
             channel_id: 'touch-collaboration',
             author_id: currentUserId!,
             content: message,
             thread_id: null,
           },
         });
       }
     } catch (error) {
       console.error('Failed to send collaborative message:', error);
     }
   }, [enableCollaboration, currentUserId]);

   const subscribeToCollaborativeMessages = useCallback(async () => {
     if (!enableCollaboration || !currentUserId) return;

     try {
       // Use working gossip_subscribe_to_entity command
       await invoke('gossip_subscribe_to_entity', {
         entity_id: 'touch-collaboration',
         entity_type: 'channel',
       });
     } catch (error) {
       console.error('Failed to subscribe to collaborative messages:', error);
     }
   }, [enableCollaboration, currentUserId]);

   // Networking functions
   const establishQuicConnection = useCallback(async (peerId: string, address: string) => {
     if (!userId) return false;

     try {
       const connection = await invoke('sync_establish_quic_connection', {
         peer_id: peerId,
         address,
         user_id: userId,
       });

       setQuicConnections(prev => new Map(prev).set(peerId, {
         peerId,
         address,
         connection,
         established: new Date(),
       }));

       return true;
     } catch (error) {
       console.error(`Failed to establish QUIC connection to ${peerId}:`, error);
       return false;
     }
   }, [userId]);

   const sendTouchGesture = useCallback(async (gesture: any, targetPeerId?: string) => {
     if (!userId) return;

     try {
       const gestureData = {
         type: 'touch-gesture',
         gesture,
         sender: userId,
         timestamp: new Date(),
         target: targetPeerId,
       };

       if (targetPeerId && quicConnections.has(targetPeerId)) {
         // Send directly via QUIC
         await invoke('sync_send_quic_message', {
           peer_id: targetPeerId,
           message: JSON.stringify(gestureData),
           user_id: userId,
         });
       } else {
         // Broadcast via DHT
         await broadcastCollaborativeUpdate(gestureData);
       }

       setTouchGestures(prev => [...prev, gestureData]);
     } catch (error) {
       console.error('Failed to send touch gesture:', error);
     }
   }, [userId, quicConnections, broadcastCollaborativeUpdate]);

   const _measureNetworkLatency = useCallback(async (peerId: string) => {
     if (!userId || !quicConnections.has(peerId)) return;

     try {
       const startTime = Date.now();

       await invoke('sync_ping_peer', {
         peer_id: peerId,
         user_id: userId,
       });

       const latency = Date.now() - startTime;
       setNetworkLatency(prev => new Map(prev).set(peerId, latency));

       return latency;
     } catch (error) {
       console.error(`Failed to ping peer ${peerId}:`, error);
       return null;
     }
   }, [userId, quicConnections]);

   const _retrieveContent = useCallback(async (address: StorageAddress) => {
     if (!enableStorage || !userId) return null;

     try {
       const result = await markdownStorage.retrieveMarkdown(address, userId);

       onContentRetrieved?.(result.content, result.metadata);
       onStorageEvent?.({ type: 'content-retrieved', data: result });

       return result.content;
     } catch (error) {
       console.error('Failed to retrieve content:', error);
       onStorageEvent?.({ type: 'retrieval-error', data: error });
       return null;
     }
   }, [enableStorage, userId, markdownStorage, onContentRetrieved, onStorageEvent]);

   const refreshContent = useCallback(async () => {
     if (!enableStorage || !userId) return;

     try {
       const stats = await storage.getStats();
       setStorageStats(stats);

       // Refresh sync status
       await dhtSync.checkSyncStatus();
     } catch (error) {
       console.error('Failed to refresh content:', error);
     }
   }, [enableStorage, userId, storage, dhtSync]);

   // Auto-sync effect
   useEffect(() => {
     if (enableStorage && autoSync && userId) {
       refreshContent();

       // Set up periodic sync
       const syncInterval = setInterval(refreshContent, 30000); // Every 30 seconds

       return () => clearInterval(syncInterval);
     }
   }, [enableStorage, autoSync, userId, refreshContent]);

   // Collaboration setup effect
   useEffect(() => {
     if (enableCollaboration && userId) {
       subscribeToCollaborativeMessages();

       // Set up live cursor tracking
       const cursorInterval = setInterval(() => {
         updateLiveCursors();
       }, 1000); // Update every second

       return () => {
         clearInterval(cursorInterval);
       };
     }
   }, [enableCollaboration, userId, subscribeToCollaborativeMessages]);

   // Networking setup effect
   useEffect(() => {
     if (enableCollaboration && userId && dhtSync.connected) {
       // Discover and connect to peers
       const discoverPeers = async () => {
         try {
           const peers = await invoke<any[]>('sync_discover_peers', { user_id: userId });
           setNetworkPeers(prev => {
             const updated = new Map(prev);
             peers.forEach((peer: any) => {
               updated.set(peer.id, {
                 id: peer.id,
                 address: peer.address,
                 connected: false,
                 lastSeen: new Date(),
               });
             });
             return updated;
           });

           // Establish QUIC connections to discovered peers
           for (const peer of peers) {
             await establishQuicConnection(peer.id, peer.address);
           }
         } catch (error) {
           console.error('Failed to discover peers:', error);
         }
       };

       discoverPeers();

       // Set up periodic peer discovery
       const discoveryInterval = setInterval(discoverPeers, 60000); // Every minute

       return () => {
         clearInterval(discoveryInterval);
       };
     }
   }, [enableCollaboration, userId, dhtSync.connected, establishQuicConnection]);

   // Live cursor tracking
   const updateLiveCursors = useCallback(async () => {
     if (!enableCollaboration || !userId) return;

     try {
       // Get current cursor/selection state
       const cursorState = {
         userId,
         timestamp: new Date(),
         // Add cursor position, selection, etc.
       };

       // Broadcast cursor state
       await broadcastCollaborativeUpdate({
         type: 'cursor-update',
         data: cursorState,
       });
     } catch (error) {
       console.error('Failed to update live cursors:', error);
     }
   }, [enableCollaboration, userId, broadcastCollaborativeUpdate]);

  // Pull-to-refresh logic
  const handleTouchStart = useCallback((event: React.TouchEvent) => {
    if (!enablePullToRefresh || scrollTop > 0) return;

    const touch = event.touches[0];
    const startY = touch.clientY;

    const handleTouchMove = (moveEvent: TouchEvent) => {
      const currentY = moveEvent.touches[0].clientY;
      const distance = Math.max(0, currentY - startY);

      if (distance > 0) {
        setPullDistance(distance);
        setIsPulling(true);
        onPullStart?.();

        // Prevent default scroll behavior while pulling
        moveEvent.preventDefault();
      }
    };

    const handleTouchEnd = () => {
      if (pullDistance >= pullThreshold && onPullToRefresh) {
        setIsRefreshing(true);
        if (hapticFeedback && isTouch) {
          triggerHapticFeedback(hapticDuration);
        }
        onPullToRefresh();

        // Reset after refresh
        setTimeout(() => {
          setIsRefreshing(false);
          setPullDistance(0);
          setIsPulling(false);
          onPullEnd?.();
        }, 1000);
      } else {
        setPullDistance(0);
        setIsPulling(false);
        onPullEnd?.();
      }

      document.removeEventListener('touchmove', handleTouchMove);
      document.removeEventListener('touchend', handleTouchEnd);
    };

    document.addEventListener('touchmove', handleTouchMove, { passive: false });
    document.addEventListener('touchend', handleTouchEnd);
  }, [enablePullToRefresh, pullThreshold, onPullToRefresh, onPullStart, onPullEnd, scrollTop, hapticFeedback, isTouch, hapticDuration]);

  // Scroll handling
  const handleScroll = useCallback((event: React.UIEvent<HTMLDivElement>) => {
    const target = event.target as HTMLDivElement;
    const newScrollTop = target.scrollTop;
    const scrollHeight = target.scrollHeight;
    const clientHeight = target.clientHeight;

    setScrollTop(newScrollTop);

    // Trigger reach top/bottom handlers
    if (newScrollTop === 0) {
      onReachTop?.();
    } else if (newScrollTop + clientHeight >= scrollHeight - 1) {
      onReachBottom?.();
    }

    onScroll?.(event);
  }, [onScroll, onReachTop, onReachBottom]);

   // Enhanced touch-friendly styles with storage integration
   const containerStyles: SxProps<Theme> = {
     position: 'relative',
     height,
     maxHeight,
     overflow: enableTouchScroll ? 'auto' : 'hidden',
     // Touch-friendly scrolling
     WebkitOverflowScrolling: enableBounceScroll && isTouch ? 'touch' : 'auto',
     // Smooth scrolling
     scrollBehavior: enableSmoothScroll ? 'smooth' : 'auto',
     // Overscroll glow effect
     ...(enableOverscrollGlow && isTouch && {
       '&::-webkit-scrollbar': {
         display: 'none',
       },
       scrollbarWidth: 'none',
       msOverflowStyle: 'none',
     }),
     // Pull-to-refresh indicator
     ...(isPulling && {
       '&::before': {
         content: '""',
         position: 'absolute',
         top: -pullDistance,
         left: 0,
         right: 0,
         height: pullDistance,
         background: `linear-gradient(to bottom, ${alpha(theme.palette.primary.main, 0.1)}, transparent)`,
         zIndex: 10,
         borderRadius: '0 0 50% 50%',
       },
     }),
     // Storage integration status overlay
     ...(enableStorage && showStorageStatus && {
       '&::after': {
         content: '""',
         position: 'absolute',
         top: 0,
         right: 0,
         width: '4px',
         height: '100%',
         backgroundColor:
           syncStatus === 'online' ? theme.palette.success.main :
           syncStatus === 'syncing' ? theme.palette.warning.main :
           theme.palette.error.main,
         opacity: 0.6,
         zIndex: 5,
       },
     }),
     ...sx,
   };

   return (
     <Box
       {...props}
       ref={containerRef}
       sx={containerStyles}
       onScroll={handleScroll}
       onTouchStart={handleTouchStart}
     >
       {/* Enhanced Sticky Header with Storage Controls */}
       {enableStickyHeader && (
         <Box
           sx={{
             position: 'sticky',
             top: 0,
             height: stickyHeaderHeight,
             backgroundColor: theme.palette.background.paper,
             borderBottom: `1px solid ${theme.palette.divider}`,
             zIndex: 5,
             display: 'flex',
             alignItems: 'center',
             padding: theme.spacing(0, 2),
             gap: 1,
           }}
         >
           {/* Storage Status Indicators */}
           {enableStorage && showStorageStatus && (
             <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
               <Tooltip title={`Storage: ${syncStatus === 'online' ? 'Online' : syncStatus === 'syncing' ? 'Syncing' : 'Offline'}`}>
                 <IconButton size="small" onClick={refreshContent}>
                   {syncStatus === 'online' && <OnlineIcon color="success" fontSize="small" />}
                   {syncStatus === 'syncing' && <SyncIcon color="warning" fontSize="small" />}
                   {syncStatus === 'offline' && <OfflineIcon color="error" fontSize="small" />}
                 </IconButton>
               </Tooltip>

               {storageStats && (
                 <Tooltip title={`Storage: ${storageStats.total_content_items} items, ${storageStats.total_bytes_stored} bytes`}>
                   <Chip
                     icon={<StorageIcon />}
                     label={`${storageStats.total_content_items}`}
                     size="small"
                     variant="outlined"
                     color="primary"
                   />
                 </Tooltip>
               )}

               {enableCollaboration && (dhtSync.peerCount > 0 || activeCollaborators.length > 0) && (
                 <Tooltip title={`${activeCollaborators.length} active collaborators, ${dhtSync.peerCount} peers connected`}>
                   <Chip
                     icon={<GroupIcon />}
                     label={`${activeCollaborators.length}`}
                     size="small"
                     variant="outlined"
                     color="secondary"
                   />
                 </Tooltip>
               )}

               {/* Collaboration Events */}
               {enableCollaboration && collaborationEvents.length > 0 && (
                 <Tooltip title={`${collaborationEvents.length} recent collaboration events`}>
                   <Chip
                     label={`${collaborationEvents.length}`}
                     size="small"
                     color="info"
                     variant="outlined"
                   />
                 </Tooltip>
               )}

               {/* Network Status */}
               {enableCollaboration && quicConnections.size > 0 && (
                 <Tooltip title={`${quicConnections.size} QUIC connections active`}>
                   <Chip
                     icon={<SyncIcon />}
                     label={`${quicConnections.size}`}
                     size="small"
                     color="success"
                     variant="outlined"
                   />
                 </Tooltip>
               )}
             </Box>
           )}

           {/* Sync Status */}
           {enableStorage && showSyncStatus && (
             <Box sx={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 1 }}>
               {dhtSync.syncing && (
                 <CircularProgress size={16} />
               )}
               <Typography variant="caption" color="text.secondary">
                 {dhtSync.connected ? 'Connected' : 'Offline'}
               </Typography>
             </Box>
           )}
         </Box>
       )}

       {/* Main Content */}
       <Box
         sx={{
           minHeight: '100%',
           padding: theme.spacing(2),
           // Touch-friendly content spacing
           '& > * + *': {
             marginTop: theme.spacing(2),
           },
         }}
       >
         {children}
       </Box>

       {/* Enhanced Sticky Footer with Storage Actions */}
       {enableStickyFooter && (
         <Box
           sx={{
             position: 'sticky',
             bottom: 0,
             height: stickyFooterHeight,
             backgroundColor: theme.palette.background.paper,
             borderTop: `1px solid ${theme.palette.divider}`,
             zIndex: 5,
             display: 'flex',
             alignItems: 'center',
             justifyContent: 'space-between',
             padding: theme.spacing(0, 2),
             gap: 1,
           }}
         >
           {/* Storage and Collaboration Actions */}
           <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
             {enableStorage && (
               <Tooltip title="Refresh Content">
                 <IconButton
                   size="small"
                   onClick={refreshContent}
                   disabled={storage.state.isLoading}
                 >
                   <RefreshIcon fontSize="small" />
                 </IconButton>
               </Tooltip>
             )}

             {enableCollaboration && (
               <Tooltip title="Send Collaboration Message">
                 <IconButton
                   size="small"
                   onClick={() => sendCollaborativeMessage('Hello from TouchContainer!')}
                   disabled={!dhtSync.connected}
                 >
                   <GroupIcon fontSize="small" />
                 </IconButton>
               </Tooltip>
             )}

             {enableCollaboration && quicConnections.size > 0 && (
               <Tooltip title="Send Touch Gesture via QUIC">
                 <IconButton
                   size="small"
                   onClick={() => {
                     const peerIds = Array.from(quicConnections.keys());
                     if (peerIds.length > 0) {
                       sendTouchGesture({
                         type: 'tap',
                         x: Math.random() * 100,
                         y: Math.random() * 100,
                         timestamp: new Date(),
                       }, peerIds[0]);
                     }
                   }}
                   disabled={quicConnections.size === 0}
                 >
                   <SyncIcon fontSize="small" />
                 </IconButton>
               </Tooltip>
             )}

             {pendingChanges.length > 0 && (
               <Chip
                 label={`${pendingChanges.length} pending`}
                 size="small"
                 color="warning"
                 variant="outlined"
               />
             )}

             {/* Collaboration Status */}
             {enableCollaboration && (
               <Chip
                 label={dhtSync.connected ? 'Collaborating' : 'Offline'}
                 size="small"
                 color={dhtSync.connected ? 'success' : 'default'}
                 variant="outlined"
               />
             )}
           </Box>

           {/* Storage and Collaboration Info */}
           <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
             {enableStorage && storageStats && (
               <Typography variant="caption" color="text.secondary">
                 {storageStats.total_bytes_stored > 0 &&
                   `${Math.round(storageStats.total_bytes_stored / 1024)} KB used`
                 }
               </Typography>
             )}

             {enableCollaboration && activeCollaborators.length > 0 && (
               <Typography variant="caption" color="text.secondary">
                 {activeCollaborators.length} collaborating
               </Typography>
             )}

             {enableCollaboration && quicConnections.size > 0 && (
               <Typography variant="caption" color="text.secondary">
                 {quicConnections.size} QUIC peers
               </Typography>
             )}
           </Box>
         </Box>
       )}

      </Box>
    );
   };

   export default TouchContainer;
