# Channels & Threads for Communitas

## Core Insight

x0x already has everything needed. A channel is just a named gossip topic + metadata
in a KvStore. A thread is just a sub-topic scoped to a parent message ID. No API
changes required - pure application-layer composition.

## Topic Naming Convention

```
Space: x0x.group.{group_id_prefix}.chat/{channel}
                                        /general        (default, auto-created)
                                        /announcements  (creator-only posting)
                                        /dev            (user-created)

Thread: x0x.group.{group_id_prefix}.thread/{msg_id}
```

Topics are exact-match (no wildcards), so apps subscribe to each channel individually.

## Mapping to x0x Primitives

| Concept | x0x Implementation |
|---------|-------------------|
| Workspace/Space | Named group (`POST /groups`) |
| Channel | Gossip topic: `x0x.group.{id}.chat/{channel-name}` |
| Channel metadata | KvStore entry: key=`channel:{name}`, value=JSON |
| Channel membership | KvStore allowlist or metadata JSON |
| Thread | Gossip topic: `x0x.group.{id}.thread/{parent-msg-id}` |
| Thread metadata | KvStore entry: key=`thread:{msg-id}`, value=JSON |
| "Also send to channel" | Publish to BOTH thread topic AND channel topic |
| Categories | KvStore entry: key=`categories`, value=JSON array |
| Pinned messages | KvStore entry: key=`pins:{channel}`, value=JSON |

## Channel Metadata (in Space's KvStore)

```json
// KvStore key: "channel:dev"
{
  "name": "dev",
  "description": "Development discussion",
  "creator": "agent_id_hex",
  "created_at": 1774525000000,
  "topic": "x0x.group.e0511563b44806e6.chat/dev",
  "is_private": false,
  "is_archived": false,
  "post_policy": "members",
  "allowed_posters": [],
  "pinned_messages": ["msg_id_1"]
}

// KvStore key: "channels_index"
{
  "channels": ["general", "dev", "announcements"],
  "categories": {
    "General": ["general", "announcements"],
    "Engineering": ["dev"]
  }
}
```

## Message Format

```json
{
  "id": "uuid",
  "text": "message content",
  "sender_name": "David",
  "sender_id": "agent_id_hex",
  "timestamp": 1774525000000,
  "channel": "dev",
  "thread_root": null,
  "broadcast": false,
  "reply_count": 0,
  "reactions": {}
}
```

## Thread Metadata

```json
// KvStore key: "thread:{parent_msg_id}"
{
  "parent_msg_id": "abc123",
  "parent_text": "Should we add WebRTC support?",
  "parent_author": "agent_id",
  "channel": "dev",
  "reply_count": 3,
  "participants": ["agent_1", "agent_2"],
  "last_reply_at": 1774525500000,
  "topic": "x0x.group.e0511563b44806e6.thread/abc123"
}
```

## Broadcast ("Also Send to Channel")

When replying in a thread with broadcast, publish to both:
1. Thread topic (for thread subscribers)
2. Channel topic with `broadcast: true` flag

Channel renders as collapsed "Alice replied in thread: ..."
