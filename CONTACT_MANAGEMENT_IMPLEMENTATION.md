# Contact Management Implementation Plan

## Overview
Complete implementation of contact management with MRU sorting, starring, reactions, and menu actions in ModernShellPrototype.

## Components Created

### 1. ContactManagementDialogs.tsx
- ✅ `AddContactDialog`: Form with name and four-word validation
- ✅ `EditContactDialog`: Pre-filled editing with validation
- ✅ `DeleteContactDialog`: Confirmation dialog
- ✅ `Contact` type with starred and lastMessageTime fields

### 2. MessageReactionPicker.tsx
- ✅ `MessageReactionPicker`: Popover with common emoji reactions
- ✅ `MessageReactionsDisplay`: Display reaction counts with user indication
- ✅ Click to toggle reactions

## Changes to ModernShellPrototype.tsx

### Type Updates (DONE)
- ✅ Added `starred?: boolean` to Conversation type
- ✅ Added `lastMessageTime?: number` to Conversation type
- ✅ Added `fourWords?: string` to Conversation type
- ✅ Added `userReacted?: boolean` to Message reactions
- ✅ Imported new dialog and reaction components

### State Management (TODO)
Need to add to component state (after line 347):
```typescript
// Contact management state
const [contactDialogMode, setContactDialogMode] = useState<'add' | 'edit' | 'delete' | null>(null)
const [selectedContact, setSelectedContact] = useState<Conversation | null>(null)

// Convert conversations from useMemo to useState for mutability
const [conversations, setConversations] = useState<Conversation[]>([...initial data with timestamps...])
const [messages, setMessages] = useState<Message[]>([...initial messages...])
```

### Handler Functions (TODO)
Add after state declarations:

```typescript
// Contact CRUD handlers
const handleAddContact = (contactData: Omit<Contact, 'id' | 'lastMessageTime'>) => {
  const newContact: Conversation = {
    id: `contact-${Date.now()}`,
    name: contactData.name,
    type: 'person',
    snippet: contactData.snippet,
    time: contactData.time,
    online: contactData.online,
    starred: contactData.starred,
    fourWords: contactData.fourWords,
    lastMessageTime: 0, // No messages yet
    org: 'Direct messages',
    status: 'read',
  }
  setConversations(prev => [newContact, ...prev])
  console.log('✅ Added contact:', newContact)
}

const handleEditContact = (id: string, updates: Partial<Contact>) => {
  setConversations(prev =>
    prev.map(c => (c.id === id ? { ...c, ...updates } : c))
  )
  console.log('✅ Updated contact:', id, updates)
}

const handleDeleteContact = (id: string) => {
  setConversations(prev => prev.filter(c => c.id !== id))
  console.log('✅ Deleted contact:', id)
}

// Star/unstar handler
const handleToggleStar = (id: string) => {
  setConversations(prev =>
    prev.map(c => (c.id === id ? { ...c, starred: !c.starred } : c))
  )
}

// Message reaction handler
const handleMessageReaction = (messageId: string, emoji: string) => {
  setMessages(prev =>
    prev.map(msg => {
      if (msg.id !== messageId) return msg

      const existingReaction = msg.reactions?.find(r => r.emoji === emoji)

      if (existingReaction) {
        // Toggle user's reaction
        if (existingReaction.userReacted) {
          // Remove reaction
          return {
            ...msg,
            reactions: msg.reactions!.map(r =>
              r.emoji === emoji
                ? { ...r, count: r.count - 1, userReacted: false }
                : r
            ).filter(r => r.count > 0),
          }
        } else {
          // Add user's reaction
          return {
            ...msg,
            reactions: msg.reactions!.map(r =>
              r.emoji === emoji
                ? { ...r, count: r.count + 1, userReacted: true }
                : r
            ),
          }
        }
      } else {
        // Add new reaction
        return {
          ...msg,
          reactions: [...(msg.reactions || []), { emoji, count: 1, userReacted: true }],
        }
      }
    })
  )

  // Update last message time for MRU sorting
  setConversations(prev =>
    prev.map(c =>
      c.id === selectedConversationId
        ? { ...c, lastMessageTime: Date.now() }
        : c
    )
  )
}

// Message menu handlers
const handleReplyToMessage = (message: Message) => {
  console.log('Reply to:', message)
  // TODO: Focus composer with reply context
}

const handleForwardMessage = (message: Message) => {
  console.log('Forward:', message)
  // TODO: Open forward dialog
}

const handleCopyMessage = (message: Message) => {
  navigator.clipboard.writeText(message.text)
  console.log('✅ Copied message to clipboard')
}

const handleDeleteMessage = (message: Message) => {
  setMessages(prev => prev.filter(m => m.id !== message.id))
  console.log('✅ Deleted message:', message.id)
}
```

### Sorting Logic (TODO)
Update filteredConversations to include MRU sorting:

```typescript
const filteredConversations = useMemo(() => {
  let filtered: Conversation[]

  switch (activeFilter) {
    case 'unread':
      filtered = conversations.filter(c => (c.unread ?? 0) > 0)
      break
    case 'favourites':
      filtered = conversations.filter(c => c.starred) // Changed from pinned
      break
    case 'groups':
      filtered = conversations.filter(c => c.type === 'group')
      break
    case 'projects':
      filtered = conversations.filter(c => c.type === 'project')
      break
    case 'people':
      filtered = conversations.filter(c => c.type === 'person')
      break
    default:
      filtered = conversations
  }

  // Sort by MRU (Most Recently Used)
  return filtered.sort((a, b) => {
    const timeA = a.lastMessageTime ?? 0
    const timeB = b.lastMessageTime ?? 0
    return timeB - timeA // Descending order (most recent first)
  })
}, [conversations, activeFilter])
```

### UI Updates (TODO)

#### 1. Add button handler (line ~800)
Replace console.log with:
```typescript
onClick={() => setContactDialogMode('add')}
```

#### 2. Add star icon to conversation list items (in conversation map)
Add after avatar/badge:
```tsx
{conversation.type === 'person' && (
  <IconButton
    size="small"
    onClick={(e) => {
      e.stopPropagation()
      handleToggleStar(conversation.id)
    }}
    sx={{ ml: 'auto' }}
  >
    {conversation.starred ? <Star sx={{ color: TOKENS.accent }} /> : <StarOutlineIcon />}
  </IconButton>
)}
```

#### 3. Add reaction picker to messages (in message bubble rendering)
After message text:
```tsx
<Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5, mt: 0.5 }}>
  {message.reactions && (
    <MessageReactionsDisplay
      reactions={message.reactions}
      onReactionClick={(emoji) => handleMessageReaction(message.id, emoji)}
    />
  )}
  {hoveredMessageId === message.id && !message.system && (
    <MessageReactionPicker
      messageId={message.id}
      onReact={handleMessageReaction}
      existingReactions={message.reactions}
    />
  )}
</Box>
```

#### 4. Update message menu (line ~900+)
Replace console.log actions with real handlers:
```tsx
<MenuItem onClick={() => { handleReplyToMessage(messageMenu.message!); setMessageMenu({ anchorEl: null }) }}>
  <ListItemIcon><ReplyOutlined /></ListItemIcon>
  <ListItemText>Reply</ListItemText>
</MenuItem>
<MenuItem onClick={() => { handleForwardMessage(messageMenu.message!); setMessageMenu({ anchorEl: null }) }}>
  <ListItemIcon><ForwardOutlined /></ListItemIcon>
  <ListItemText>Forward</ListItemText>
</MenuItem>
<MenuItem onClick={() => { handleCopyMessage(messageMenu.message!); setMessageMenu({ anchorEl: null }) }}>
  <ListItemIcon><ContentCopyOutlined /></ListItemIcon>
  <ListItemText>Copy</ListItemText>
</MenuItem>
<Divider />
<MenuItem onClick={() => { handleDeleteMessage(messageMenu.message!); setMessageMenu({ anchorEl: null }) }} sx={{ color: 'error.main' }}>
  <ListItemIcon><DeleteOutline color="error" /></ListItemIcon>
  <ListItemText>Delete</ListItemText>
</MenuItem>
```

#### 5. Add dialogs at end of component (before closing tag)
```tsx
{/* Contact Management Dialogs */}
<AddContactDialog
  open={contactDialogMode === 'add'}
  onClose={() => setContactDialogMode(null)}
  onSave={handleAddContact}
/>

<EditContactDialog
  open={contactDialogMode === 'edit'}
  contact={selectedContact as Contact}
  onClose={() => {
    setContactDialogMode(null)
    setSelectedContact(null)
  }}
  onSave={handleEditContact}
/>

<DeleteContactDialog
  open={contactDialogMode === 'delete'}
  contact={selectedContact as Contact}
  onClose={() => {
    setContactDialogMode(null)
    setSelectedContact(null)
  }}
  onConfirm={handleDeleteContact}
/>
```

## Initial Data Updates (TODO)
Update mock conversations to include timestamps:

```typescript
const now = Date.now()
{
  id: 'ben-thomson',
  name: 'Ben Thomson',
  type: 'person',
  snippet: 'Ok I will put the kilt away then 😄😄',
  time: '18:21',
  status: 'delivered',
  org: 'Direct messages',
  online: true,
  starred: false,
  fourWords: 'ocean-blue-mountain-star',
  lastMessageTime: now - 3600000, // 1 hour ago
},
{
  id: 'lauren',
  name: 'Lauren McFadyen',
  type: 'person',
  snippet: "That's OK Lauren, no worries",
  time: 'Yesterday',
  status: 'read',
  org: 'Direct messages',
  online: false,
  starred: true, // Starred contact
  fourWords: 'forest-river-cloud-moon',
  lastMessageTime: now - 86400000, // 1 day ago
},
```

## Testing Plan
1. Add new contact with four-word validation
2. Star/unstar contacts
3. Filter by favourites
4. Edit contact name and four-words
5. Delete contact with confirmation
6. Add reactions to messages
7. Toggle reactions (add/remove)
8. Verify MRU sorting after new messages
9. Test message menu (reply, forward, copy, delete)
10. Verify all actions work across different entity types

## Next Steps
1. Apply all state management changes
2. Update UI with new handlers
3. Add dialogs to render
4. Update initial data with timestamps
5. Test with Chrome DevTools MCP
