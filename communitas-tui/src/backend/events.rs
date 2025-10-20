use communitas_core::crdt::EntityType;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{RwLock, mpsc};

/// Event types that can be subscribed to
#[derive(Debug, Clone, PartialEq)]
pub enum BackendEvent {
    /// Entity was created
    EntityCreated {
        entity_id: String,
        entity_type: EntityType,
        name: String,
    },
    /// Entity was updated
    EntityUpdated {
        entity_id: String,
        entity_type: EntityType,
    },
    /// Member was added to entity
    MemberAdded {
        entity_id: String,
        entity_type: EntityType,
        member_id: String,
    },
    /// Member was removed from entity
    MemberRemoved {
        entity_id: String,
        entity_type: EntityType,
        member_id: String,
    },
    /// Message was received
    MessageReceived {
        message_id: String,
        entity_id: String,
        author: String,
        text: String,
    },
    /// Message was sent
    MessageSent {
        message_id: String,
        entity_id: String,
    },
    /// Thread was created from a message
    ThreadCreated {
        thread_id: String,
        parent_message_id: String,
        entity_id: String,
    },
    /// Network peer connected
    PeerConnected { peer_id: String, address: String },
    /// Network peer disconnected
    PeerDisconnected { peer_id: String },
}

/// Event filter for subscription
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Filter by entity type (None = all types)
    pub entity_type: Option<EntityType>,
    /// Filter by entity ID (None = all entities)
    pub entity_id: Option<String>,
}

impl EventFilter {
    /// Create filter that matches all events
    pub fn all() -> Self {
        Self {
            entity_type: None,
            entity_id: None,
        }
    }

    /// Create filter for specific entity type
    pub fn entity_type(entity_type: EntityType) -> Self {
        Self {
            entity_type: Some(entity_type),
            entity_id: None,
        }
    }

    /// Create filter for specific entity ID
    pub fn entity_id(entity_id: String) -> Self {
        Self {
            entity_type: None,
            entity_id: Some(entity_id),
        }
    }

    /// Check if event matches this filter
    pub fn matches(&self, event: &BackendEvent) -> bool {
        // Extract entity type and ID from event
        let (event_type, event_id) = match event {
            BackendEvent::EntityCreated {
                entity_type,
                entity_id,
                ..
            } => (Some(*entity_type), Some(entity_id.as_str())),
            BackendEvent::EntityUpdated {
                entity_type,
                entity_id,
            } => (Some(*entity_type), Some(entity_id.as_str())),
            BackendEvent::MemberAdded {
                entity_type,
                entity_id,
                ..
            } => (Some(*entity_type), Some(entity_id.as_str())),
            BackendEvent::MemberRemoved {
                entity_type,
                entity_id,
                ..
            } => (Some(*entity_type), Some(entity_id.as_str())),
            BackendEvent::MessageReceived { entity_id, .. } => (None, Some(entity_id.as_str())),
            BackendEvent::MessageSent { entity_id, .. } => (None, Some(entity_id.as_str())),
            BackendEvent::ThreadCreated { entity_id, .. } => (None, Some(entity_id.as_str())),
            BackendEvent::PeerConnected { .. } | BackendEvent::PeerDisconnected { .. } => {
                (None, None)
            }
        };

        // Check entity type filter
        if let Some(filter_type) = &self.entity_type
            && event_type != Some(*filter_type)
        {
            return false;
        }

        // Check entity ID filter
        if let Some(filter_id) = &self.entity_id
            && event_id != Some(filter_id.as_str())
        {
            return false;
        }

        true
    }
}

/// Subscription to backend events
pub struct Subscription {
    pub id: u64,
    pub sender: mpsc::Sender<BackendEvent>,
    pub filter: EventFilter,
}

/// Event subscription manager
///
/// Performance characteristics:
/// - Subscription: O(1) with write lock
/// - Unsubscription: O(1) with write lock
/// - Event publish: O(n) where n = number of subscribers, with read lock
/// - Queue operations: O(1) with VecDeque
pub struct EventManager {
    /// Next subscription ID
    next_id: AtomicU64,
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashMap<u64, Subscription>>>,
    /// Event queue for offline support (None = queueing disabled)
    /// Uses VecDeque for O(1) pop_front when at capacity
    event_queue: Arc<RwLock<Option<VecDeque<BackendEvent>>>>,
    /// Maximum queue size
    max_queue_size: usize,
}

impl EventManager {
    /// Create new event manager
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_queue: Arc::new(RwLock::new(None)),
            max_queue_size: 0,
        }
    }

    /// Enable event queue with specified size
    ///
    /// Uses VecDeque for efficient O(1) pop_front when queue is full.
    pub async fn enable_queue(&mut self, max_size: usize) {
        self.max_queue_size = max_size;
        let mut queue = self.event_queue.write().await;
        *queue = Some(VecDeque::with_capacity(max_size));
    }

    /// Subscribe to events with filter
    ///
    /// Returns subscription ID that can be used to unsubscribe later.
    /// Queued events matching the filter will be sent immediately upon subscription.
    pub async fn subscribe(&self, sender: mpsc::Sender<BackendEvent>, filter: EventFilter) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // Store reference to sender before moving into subscription
        let sender_ref = sender.clone();

        let subscription = Subscription {
            id,
            sender,
            filter: filter.clone(),
        };

        // Add to subscriptions
        let mut subs = self.subscriptions.write().await;
        subs.insert(id, subscription);

        // Send queued events to new subscriber if queue is enabled
        let queue = self.event_queue.read().await;
        if let Some(events) = queue.as_ref() {
            for event in events {
                // Filter and send queued events (only clone if filter matches)
                if filter.matches(event) {
                    let _ = sender_ref.send(event.clone()).await;
                }
            }
        }

        id
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, id: u64) -> bool {
        let mut subs = self.subscriptions.write().await;
        subs.remove(&id).is_some()
    }

    /// Publish event to all matching subscribers
    ///
    /// Performance optimizations:
    /// - Only clones events when filter matches
    /// - Uses VecDeque::pop_front() for O(1) queue management
    /// - Releases read lock before sending to reduce contention
    pub async fn publish(&self, event: BackendEvent) {
        // Collect matching senders (minimize lock hold time)
        let matching_senders: Vec<mpsc::Sender<BackendEvent>> = {
            let subs = self.subscriptions.read().await;

            // Queue event if queueing enabled and no active subscribers
            if subs.is_empty() {
                let mut queue = self.event_queue.write().await;
                if let Some(events) = queue.as_mut() {
                    // Add to queue, removing oldest if at capacity (O(1) with VecDeque)
                    if events.len() >= self.max_queue_size {
                        events.pop_front();
                    }
                    events.push_back(event.clone());
                }
            }

            // Collect senders that match filter (don't clone event yet)
            subs.values()
                .filter(|sub| sub.filter.matches(&event))
                .map(|sub| sub.sender.clone())
                .collect()
        }; // Read lock released here

        // Send to matching subscribers (lock released, can send in parallel)
        for sender in matching_senders {
            // Ignore send errors (subscriber may have dropped receiver)
            let _ = sender.send(event.clone()).await;
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}
