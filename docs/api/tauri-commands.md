# Tauri Commands API Reference

Complete reference for all Tauri commands available in Communitas desktop application.

## Overview

Tauri commands provide the IPC (Inter-Process Communication) interface between the React frontend and the Rust backend. All commands are invoked using the `invoke` function from `@tauri-apps/api/tauri`.

**Total Commands**: 90+

**Command Modules**:
- [Authentication Commands](#authentication-commands) (40+)
- [Organization & Channel Commands](#organization--channel-commands) (20+)
- [Project & Issue Commands](#project--issue-commands) (15+)
- [Member Management Commands](#member-management-commands) (10+)
- [Document Sync Commands](#document-sync-commands) (8+)
- [Utility Commands](#utility-commands) (5+)

## Usage Pattern

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Basic invocation
const result = await invoke('command_name', {
  parameter1: value1,
  parameter2: value2
});

// With error handling
try {
  const result = await invoke('auth_login', {
    fourWords: 'ocean-forest-moon-star',
    password: 'secure-password'
  });
  console.log('Success:', result);
} catch (error) {
  console.error('Error:', error);
}
```

---

## Authentication Commands

### Module: `auth`

Authentication and encrypted storage management using the shared `AuthService`.

#### Core Authentication

##### `auth_initialize`

Initialize the authentication service. Must be called before any other auth commands.

**Parameters**: None

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_initialize');
```

**Errors**:
- `"Failed to initialize storage: [reason]"` - Storage initialization failed

---

##### `auth_create_vault`

Create a new encrypted vault for a four-word identity.

Uses ChaCha20-Poly1305 encryption with PBKDF2 key derivation (100,000 iterations).

**Parameters**:
```typescript
{
  fourWords: string;    // Four-word address (e.g., "ocean-forest-moon-star")
  password: string;     // Vault password (minimum 8 characters recommended)
  displayName: string;  // User's display name
}
```

**Returns**: `Result<string, String>` - Vault ID (UUID)

**Example**:
```typescript
const vaultId = await invoke('auth_create_vault', {
  fourWords: 'ocean-forest-moon-star',
  password: 'MySecurePassword123!',
  displayName: 'Alice Johnson'
});
console.log('Vault created:', vaultId);
```

**Errors**:
- `"Auth service not initialized. Call auth_initialize first."` - Service not initialized
- `"Failed to create vault: [reason]"` - Vault creation failed
- `"Vault already exists"` - Four-word address already has a vault

---

##### `auth_login`

Login with four-word address and password.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address
  password: string;   // Vault password
}
```

**Returns**: `Result<SessionInfo, String>`

```typescript
interface SessionInfo {
  sessionId: string;      // Unique session identifier
  fourWords: string;      // Four-word address
  displayName: string;    // User's display name
}
```

**Example**:
```typescript
const session = await invoke('auth_login', {
  fourWords: 'ocean-forest-moon-star',
  password: 'MySecurePassword123!'
});
console.log('Logged in:', session.displayName);
```

**Errors**:
- `"Auth service not initialized"` - Call `auth_initialize` first
- `"Login failed: Invalid password"` - Incorrect password
- `"Login failed: Vault not found"` - Four-word address doesn't exist

---

##### `auth_login_password_only`

Login using password only (searches all vaults).

Useful when user doesn't remember their four-word address.

**Parameters**:
```typescript
{
  password: string;  // Password to search for
}
```

**Returns**: `Result<SessionInfo, String>`

**Example**:
```typescript
const session = await invoke('auth_login_password_only', {
  password: 'MySecurePassword123!'
});
console.log('Found identity:', session.fourWords);
```

**Errors**:
- `"Password-only login failed: No vault found with this password"` - Password doesn't match any vault
- `"Multiple vaults found with this password"` - Ambiguous (user must specify four-word address)

---

##### `auth_logout`

Logout the current session.

Clears the active session and securely zeros sensitive data in memory.

**Parameters**: None

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_logout');
console.log('Logged out successfully');
```

**Errors**:
- `"Logout failed: [reason]"` - Logout operation failed

---

##### `auth_get_session`

Get the current active session.

**Parameters**: None

**Returns**: `Result<Option<SessionInfo>, String>` - Session info or `null` if not logged in

**Example**:
```typescript
const session = await invoke('auth_get_session');
if (session) {
  console.log('Current user:', session.displayName);
} else {
  console.log('No active session');
}
```

---

##### `auth_check_session`

Check if a session is still valid.

**Parameters**:
```typescript
{
  sessionId: string;  // Session ID to check
}
```

**Returns**: `Result<boolean, String>` - `true` if session exists and is valid

**Example**:
```typescript
const isValid = await invoke('auth_check_session', {
  sessionId: 'session-uuid-here'
});
if (!isValid) {
  // Redirect to login
}
```

---

#### Vault Management

##### `auth_list_vaults`

List all available vaults on this device.

Returns metadata for all encrypted vaults (storage containers) on the device.

**Parameters**: None

**Returns**: `Result<VaultInfo[], String>`

```typescript
interface VaultInfo {
  fourWords: string;     // Four-word address
  displayName: string;   // Display name
  createdAt: number;     // Unix timestamp (seconds)
  lastAccessed: number;  // Unix timestamp (seconds)
  sizeBytes: number;     // Vault size in bytes
}
```

**Example**:
```typescript
const vaults = await invoke('auth_list_vaults');
vaults.forEach(vault => {
  console.log(`${vault.displayName} (${vault.fourWords})`);
  console.log(`Last accessed: ${new Date(vault.lastAccessed * 1000)}`);
});
```

---

##### `auth_delete_vault`

Delete a vault and its associated identity.

**WARNING**: This permanently deletes:
- The encrypted vault file
- Identity from recent identities list
- Any passkey/biometric data
- Keyring stored passwords

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address of vault to delete
  password: string;   // Password confirmation (must be correct)
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_delete_vault', {
  fourWords: 'old-test-account-name',
  password: 'CorrectPassword123!'
});
console.log('Vault deleted permanently');
```

**Errors**:
- `"Failed to delete vault: Invalid password"` - Password verification failed
- `"Failed to delete vault: Vault not found"` - Four-word address doesn't exist

---

##### `auth_list_old_vaults`

Get list of old/stale vaults that can be cleaned up.

Returns vaults that haven't been accessed in the specified number of days.

**Parameters**:
```typescript
{
  daysSinceAccess: number;  // Number of days of inactivity
}
```

**Returns**: `Result<VaultInfo[], String>`

**Example**:
```typescript
// Find vaults not accessed in last 90 days
const oldVaults = await invoke('auth_list_old_vaults', {
  daysSinceAccess: 90
});
console.log(`Found ${oldVaults.length} old vaults to clean up`);
```

---

#### Configuration & Settings

##### `auth_get_config`

Get application configuration.

**Parameters**: None

**Returns**: `Result<AppConfig, String>`

```typescript
interface AppConfig {
  autoLoginEnabled: boolean;
  keyringEnabled: boolean;
  sessionTimeout: number;      // Minutes
  maxSessions: number;          // Per device
  passkeyEnabled: boolean;
}
```

**Example**:
```typescript
const config = await invoke('auth_get_config');
console.log('Auto-login:', config.autoLoginEnabled);
```

---

##### `auth_set_auto_login`

Enable or disable auto-login.

**Parameters**:
```typescript
{
  enabled: boolean;  // true to enable, false to disable
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_set_auto_login', { enabled: true });
console.log('Auto-login enabled');
```

---

##### `auth_set_keyring_enabled`

Enable or disable keyring password storage.

**Parameters**:
```typescript
{
  enabled: boolean;  // true to enable, false to disable
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_set_keyring_enabled', { enabled: true });
console.log('Keyring storage enabled');
```

---

#### Recent Identities

##### `auth_get_recent_identities`

Get recent identities for quick access.

**Parameters**: None

**Returns**: `Result<RecentIdentity[], String>`

```typescript
interface RecentIdentity {
  fourWords: string;
  displayName: string;
  lastUsed: number;       // Unix timestamp
  hasPasskey: boolean;    // Has biometric authentication
}
```

**Example**:
```typescript
const recents = await invoke('auth_get_recent_identities');
recents.forEach(identity => {
  console.log(`${identity.displayName} - ${identity.fourWords}`);
});
```

---

##### `auth_remove_recent_identity`

Remove a recent identity from the list.

**Note**: This does NOT delete the vault, only removes it from the recent list.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address to remove
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_remove_recent_identity', {
  fourWords: 'old-identity-not-used'
});
```

---

##### `auth_try_auto_login`

Try auto-login with last used identity.

**Parameters**: None

**Returns**: `Result<Option<SessionInfo>, String>` - Session info or `null` if auto-login not available

**Example**:
```typescript
const session = await invoke('auth_try_auto_login');
if (session) {
  console.log('Auto-logged in as:', session.displayName);
} else {
  console.log('Auto-login not available, showing login screen');
}
```

---

#### Passkey/Biometric Authentication

##### `auth_passkey_register`

Register a passkey/biometric for an identity (legacy - without WebAuthn).

Allows biometric authentication (Touch ID, Face ID, Windows Hello).

**Parameters**:
```typescript
{
  fourWords: string;   // Four-word address
  deviceName: string;  // Device identifier (e.g., "MacBook Pro")
  password: string;    // Current password (for keyring storage)
}
```

**Returns**: `Result<PasskeyInfo, String>`

```typescript
interface PasskeyInfo {
  fourWords: string;
  registeredAt: number;    // Unix timestamp
  lastUsed: number | null; // Unix timestamp or null
  deviceName: string;
}
```

**Example**:
```typescript
const passkey = await invoke('auth_passkey_register', {
  fourWords: 'ocean-forest-moon-star',
  deviceName: 'MacBook Pro',
  password: 'MySecurePassword123!'
});
console.log('Passkey registered:', passkey.registeredAt);
```

---

##### `auth_passkey_register_webauthn`

Register a passkey with WebAuthn credential.

Stores the WebAuthn credential data for true biometric authentication.

**Parameters**:
```typescript
{
  fourWords: string;       // Four-word address
  deviceName: string;      // Device identifier
  credentialData: string;  // WebAuthn credential as JSON string
}
```

**Returns**: `Result<PasskeyInfo, String>`

**Example**:
```typescript
// Frontend creates WebAuthn credential
const credential = await navigator.credentials.create({
  publicKey: {
    challenge: new Uint8Array(32),
    rp: { name: "Communitas" },
    user: {
      id: new Uint8Array(16),
      name: "ocean-forest-moon-star",
      displayName: "Alice"
    },
    pubKeyCredParams: [{ alg: -7, type: "public-key" }],
    authenticatorSelection: {
      authenticatorAttachment: "platform",
      userVerification: "required"
    }
  }
});

const passkey = await invoke('auth_passkey_register_webauthn', {
  fourWords: 'ocean-forest-moon-star',
  deviceName: 'MacBook Pro',
  credentialData: JSON.stringify(credential)
});
```

---

##### `auth_passkey_authenticate`

Authenticate using passkey/biometric (legacy).

Retrieves password from keyring after OS-level biometric verification.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address
}
```

**Returns**: `Result<SessionInfo, String>`

**Example**:
```typescript
const session = await invoke('auth_passkey_authenticate', {
  fourWords: 'ocean-forest-moon-star'
});
console.log('Biometric login successful');
```

---

##### `auth_passkey_authenticate_webauthn`

Authenticate using WebAuthn passkey.

Verifies the WebAuthn credential and creates a session.

**Parameters**:
```typescript
{
  fourWords: string;      // Four-word address
  assertionData: string;  // WebAuthn assertion as JSON string
}
```

**Returns**: `Result<SessionInfo, String>`

**Example**:
```typescript
// Frontend gets WebAuthn assertion
const assertion = await navigator.credentials.get({
  publicKey: {
    challenge: new Uint8Array(32),
    rpId: "communitas.life",
    userVerification: "required"
  }
});

const session = await invoke('auth_passkey_authenticate_webauthn', {
  fourWords: 'ocean-forest-moon-star',
  assertionData: JSON.stringify(assertion)
});
```

---

##### `auth_passkey_has_passkey`

Check if a passkey is registered for an identity.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address
}
```

**Returns**: `Result<boolean, String>`

**Example**:
```typescript
const hasPasskey = await invoke('auth_passkey_has_passkey', {
  fourWords: 'ocean-forest-moon-star'
});
if (hasPasskey) {
  console.log('Show biometric login option');
}
```

---

##### `auth_passkey_get_info`

Get passkey information for an identity.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address
}
```

**Returns**: `Result<PasskeyInfo, String>`

**Example**:
```typescript
const info = await invoke('auth_passkey_get_info', {
  fourWords: 'ocean-forest-moon-star'
});
console.log(`Passkey registered on: ${info.deviceName}`);
console.log(`Last used: ${new Date(info.lastUsed * 1000)}`);
```

---

##### `auth_passkey_delete`

Delete passkey registration for an identity.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('auth_passkey_delete', {
  fourWords: 'ocean-forest-moon-star'
});
console.log('Passkey deleted');
```

---

#### macOS Touch ID Commands

##### `auth_touchid_register` (macOS only)

Register a passkey using native macOS Touch ID.

Uses the macOS Security Framework to trigger a native Touch ID prompt.

**Parameters**:
```typescript
{
  fourWords: string;    // Four-word address
  deviceName: string;   // Device identifier
  password: string;     // Current password
  reason?: string;      // Optional reason to show user
}
```

**Returns**: `Result<PasskeyInfo, String>`

**Example**:
```typescript
const passkey = await invoke('auth_touchid_register', {
  fourWords: 'ocean-forest-moon-star',
  deviceName: 'MacBook Pro',
  password: 'MySecurePassword123!',
  reason: 'Register Touch ID for quick access'
});
console.log('Touch ID registered');
```

**Errors**:
- `"Touch ID is not available on this device"` - No Touch ID hardware
- `"Touch ID authentication was cancelled or failed"` - User cancelled or failed
- `"Failed to trigger Touch ID: [reason]"` - System error

---

##### `auth_touchid_authenticate` (macOS only)

Authenticate using native macOS Touch ID.

**Parameters**:
```typescript
{
  fourWords: string;  // Four-word address
  reason?: string;    // Optional reason to show user
}
```

**Returns**: `Result<SessionInfo, String>`

**Example**:
```typescript
const session = await invoke('auth_touchid_authenticate', {
  fourWords: 'ocean-forest-moon-star',
  reason: 'Sign in to Communitas'
});
console.log('Touch ID authentication successful');
```

---

#### Utility Commands

##### `get_os_username`

Get OS username for default display name.

Returns the current OS user's display name to use as default when creating new identities.

**Parameters**: None

**Returns**: `Result<string, String>` - OS username or "User" as fallback

**Example**:
```typescript
const username = await invoke('get_os_username');
console.log('Suggested display name:', username);
```

**Platform Behavior**:
- **macOS**: Uses `id -F` (full name) or `$USER`
- **Windows**: Uses `%USERNAME%`
- **Linux**: Uses GECOS field or `$USER`

---

## Organization & Channel Commands

### Module: `org_commands`

Channel messaging, threads, and member management.

#### Channel Management

##### `create_channel`

Create a new channel in an organization.

**Parameters**:
```typescript
{
  orgId: string;          // Organization ID
  name: string;           // Channel name
  description?: string;   // Optional description
  createdBy: string;      // Creator user ID
}
```

**Returns**: `Result<Channel, String>`

```typescript
interface Channel {
  id: string;             // Channel UUID
  orgId: string;
  name: string;
  description: string | null;
  createdAt: number;      // Unix timestamp
  createdBy: string;
  memberCount: number;
}
```

**Example**:
```typescript
const channel = await invoke('create_channel', {
  orgId: 'org-uuid',
  name: 'General',
  description: 'General discussion for everyone',
  createdBy: 'user-uuid'
});
console.log('Channel created:', channel.name);
```

---

##### `get_channel`

Get channel by ID.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
}
```

**Returns**: `Result<Channel | null, String>`

**Example**:
```typescript
const channel = await invoke('get_channel', {
  channelId: 'channel-uuid'
});
if (channel) {
  console.log('Channel:', channel.name);
}
```

---

##### `list_channels`

List all channels in an organization.

**Parameters**:
```typescript
{
  orgId: string;  // Organization UUID
}
```

**Returns**: `Result<Channel[], String>`

**Example**:
```typescript
const channels = await invoke('list_channels', {
  orgId: 'org-uuid'
});
channels.forEach(ch => console.log(ch.name));
```

---

#### Messaging

##### `send_message`

Send a message to a channel.

**Parameters**:
```typescript
{
  channelId: string;    // Channel UUID
  authorId: string;     // Author user UUID
  content: string;      // Message content
  threadId?: string;    // Optional thread UUID for replies
}
```

**Returns**: `Result<Message, String>`

```typescript
interface Message {
  id: string;               // Message UUID
  channelId: string;
  authorId: string;
  content: string;
  threadId: string | null;
  createdAt: number;        // Unix timestamp
  updatedAt: number | null; // Unix timestamp or null
  isDeleted: boolean;
  replyCount: number;
}
```

**Example**:
```typescript
const message = await invoke('send_message', {
  channelId: 'channel-uuid',
  authorId: 'user-uuid',
  content: 'Hello, everyone!',
  threadId: null
});
console.log('Message sent:', message.id);
```

---

##### `edit_message`

Edit an existing message.

**Parameters**:
```typescript
{
  messageId: string;    // Message UUID
  newContent: string;   // New message content
}
```

**Returns**: `Result<Message, String>`

**Example**:
```typescript
const edited = await invoke('edit_message', {
  messageId: 'message-uuid',
  newContent: 'Updated message content'
});
console.log('Message edited at:', edited.updatedAt);
```

---

##### `delete_message`

Delete a message (soft delete - marks as deleted).

**Parameters**:
```typescript
{
  messageId: string;  // Message UUID
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('delete_message', {
  messageId: 'message-uuid'
});
console.log('Message deleted');
```

---

##### `get_messages`

Get messages from a channel with pagination.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
  limit?: number;     // Maximum messages to return (default: 50)
  offset?: number;    // Offset for pagination (default: 0)
}
```

**Returns**: `Result<Message[], String>`

**Example**:
```typescript
// Get first 50 messages
const messages = await invoke('get_messages', {
  channelId: 'channel-uuid',
  limit: 50,
  offset: 0
});

// Get next 50 messages
const moreMessages = await invoke('get_messages', {
  channelId: 'channel-uuid',
  limit: 50,
  offset: 50
});
```

---

#### Threads

##### `create_thread`

Create a thread from a message.

**Parameters**:
```typescript
{
  parentMessageId: string;  // Parent message UUID
}
```

**Returns**: `Result<Thread, String>`

```typescript
interface Thread {
  id: string;             // Thread UUID
  channelId: string;
  parentMessageId: string;
  createdAt: number;      // Unix timestamp
  replyCount: number;
}
```

**Example**:
```typescript
const thread = await invoke('create_thread', {
  parentMessageId: 'message-uuid'
});
console.log('Thread created:', thread.id);
```

---

##### `get_thread_replies`

Get all replies in a thread.

**Parameters**:
```typescript
{
  threadId: string;  // Thread UUID
}
```

**Returns**: `Result<Message[], String>`

**Example**:
```typescript
const replies = await invoke('get_thread_replies', {
  threadId: 'thread-uuid'
});
console.log(`Thread has ${replies.length} replies`);
```

---

#### Channel Members

##### `add_channel_member`

Add a member to a channel.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
  userId: string;     // User UUID to add
  role: string;       // Member role ("admin", "member", "viewer")
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('add_channel_member', {
  channelId: 'channel-uuid',
  userId: 'user-uuid',
  role: 'member'
});
console.log('Member added to channel');
```

---

##### `remove_channel_member`

Remove a member from a channel.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
  userId: string;     // User UUID to remove
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('remove_channel_member', {
  channelId: 'channel-uuid',
  userId: 'user-uuid'
});
console.log('Member removed from channel');
```

---

##### `get_channel_members`

Get all members of a channel.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
}
```

**Returns**: `Result<[string, string][], String>` - Array of `[userId, role]` tuples

**Example**:
```typescript
const members = await invoke('get_channel_members', {
  channelId: 'channel-uuid'
});
members.forEach(([userId, role]) => {
  console.log(`User ${userId}: ${role}`);
});
```

---

## Project & Issue Commands

### Module: `org_commands`

Project management and issue tracking.

#### Projects

##### `create_project`

Create a new project in an organization.

**Parameters**:
```typescript
{
  orgId: string;          // Organization UUID
  name: string;           // Project name
  description?: string;   // Optional description
  icon?: string;          // Optional emoji icon
  color?: string;         // Optional hex color (#RGB or #RRGGBB)
  createdBy: string;      // Creator user UUID
}
```

**Returns**: `Result<Project, String>`

```typescript
interface Project {
  id: string;             // Project UUID
  orgId: string;
  name: string;
  description: string | null;
  icon: string | null;
  color: string | null;
  createdAt: number;      // Unix timestamp
  createdBy: string;
  issueCount: number;
}
```

**Example**:
```typescript
const project = await invoke('create_project', {
  orgId: 'org-uuid',
  name: 'Q1 Launch',
  description: 'Q1 product launch tracking',
  icon: '🚀',
  color: '#4285f4',
  createdBy: 'user-uuid'
});
console.log('Project created:', project.name);
```

---

##### `get_project`

Get project by ID.

**Parameters**:
```typescript
{
  projectId: string;  // Project UUID
}
```

**Returns**: `Result<Project | null, String>`

**Example**:
```typescript
const project = await invoke('get_project', {
  projectId: 'project-uuid'
});
```

---

##### `list_projects`

List all projects in an organization.

**Parameters**:
```typescript
{
  orgId: string;  // Organization UUID
}
```

**Returns**: `Result<Project[], String>`

**Example**:
```typescript
const projects = await invoke('list_projects', {
  orgId: 'org-uuid'
});
projects.forEach(p => console.log(p.name));
```

---

#### Issues

##### `create_issue`

Create a new issue in a project.

**Parameters**:
```typescript
{
  projectId: string;      // Project UUID
  title: string;          // Issue title
  description?: string;   // Optional description
  priority: string;       // "urgent", "high", "medium", "low"
  reporterId: string;     // Reporter user UUID
}
```

**Returns**: `Result<Issue, String>`

```typescript
interface Issue {
  id: string;               // Issue UUID
  projectId: string;
  title: string;
  description: string | null;
  status: string;           // "backlog", "todo", "in-progress", "done", "canceled"
  priority: string;         // "urgent", "high", "medium", "low"
  reporterId: string;
  assigneeId: string | null;
  createdAt: number;        // Unix timestamp
  updatedAt: number;        // Unix timestamp
  commentCount: number;
}
```

**Example**:
```typescript
const issue = await invoke('create_issue', {
  projectId: 'project-uuid',
  title: 'Implement user authentication',
  description: 'Add password and biometric authentication support',
  priority: 'high',
  reporterId: 'user-uuid'
});
console.log('Issue created:', issue.id);
```

---

##### `get_issue`

Get issue by ID.

**Parameters**:
```typescript
{
  issueId: string;  // Issue UUID
}
```

**Returns**: `Result<Issue | null, String>`

**Example**:
```typescript
const issue = await invoke('get_issue', {
  issueId: 'issue-uuid'
});
```

---

##### `list_issues`

List all issues in a project.

**Parameters**:
```typescript
{
  projectId: string;  // Project UUID
}
```

**Returns**: `Result<Issue[], String>`

**Example**:
```typescript
const issues = await invoke('list_issues', {
  projectId: 'project-uuid'
});
console.log(`Project has ${issues.length} issues`);
```

---

##### `list_issues_by_status`

List issues filtered by status.

**Parameters**:
```typescript
{
  projectId: string;  // Project UUID
  status: string;     // "backlog", "todo", "in-progress", "done", "canceled"
}
```

**Returns**: `Result<Issue[], String>`

**Example**:
```typescript
const inProgress = await invoke('list_issues_by_status', {
  projectId: 'project-uuid',
  status: 'in-progress'
});
console.log(`${inProgress.length} issues in progress`);
```

---

##### `update_issue_status`

Update the status of an issue.

**Parameters**:
```typescript
{
  issueId: string;   // Issue UUID
  newStatus: string; // "backlog", "todo", "in-progress", "done", "canceled"
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('update_issue_status', {
  issueId: 'issue-uuid',
  newStatus: 'done'
});
console.log('Issue marked as done');
```

---

##### `assign_issue`

Assign an issue to a user.

**Parameters**:
```typescript
{
  issueId: string;     // Issue UUID
  assigneeId: string;  // User UUID to assign
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('assign_issue', {
  issueId: 'issue-uuid',
  assigneeId: 'user-uuid'
});
console.log('Issue assigned');
```

---

##### `update_issue_priority`

Update the priority of an issue.

**Parameters**:
```typescript
{
  issueId: string;  // Issue UUID
  priority: string; // "urgent", "high", "medium", "low"
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('update_issue_priority', {
  issueId: 'issue-uuid',
  priority: 'urgent'
});
console.log('Priority updated to urgent');
```

---

##### `add_issue_comment`

Add a comment to an issue.

**Parameters**:
```typescript
{
  issueId: string;   // Issue UUID
  authorId: string;  // Comment author UUID
  content: string;   // Comment content
}
```

**Returns**: `Result<IssueComment, String>`

```typescript
interface IssueComment {
  id: string;         // Comment UUID
  issueId: string;
  authorId: string;
  content: string;
  createdAt: number;  // Unix timestamp
  updatedAt: number | null;
}
```

**Example**:
```typescript
const comment = await invoke('add_issue_comment', {
  issueId: 'issue-uuid',
  authorId: 'user-uuid',
  content: 'Working on this now. Will have PR ready tomorrow.'
});
console.log('Comment added:', comment.id);
```

---

##### `get_issue_comments`

Get all comments for an issue.

**Parameters**:
```typescript
{
  issueId: string;  // Issue UUID
}
```

**Returns**: `Result<IssueComment[], String>`

**Example**:
```typescript
const comments = await invoke('get_issue_comments', {
  issueId: 'issue-uuid'
});
console.log(`Issue has ${comments.length} comments`);
```

---

## Member Management Commands

### Module: `member_commands`

CRDT-based member management system.

##### `create_member`

Create a new member.

**Parameters**:
```typescript
{
  fourWords: string;      // Four-word address
  displayName: string;    // Member display name
  publicKey: string;      // Base64-encoded public key
  capabilities: object;   // Member capabilities
}
```

**Returns**: `Result<Member, String>`

**Example**:
```typescript
const member = await invoke('create_member', {
  fourWords: 'valley-river-cloud-wind',
  displayName: 'Bob Smith',
  publicKey: 'base64_public_key_here',
  capabilities: {
    canInvite: true,
    canManageRoles: false
  }
});
```

---

##### `get_member`

Get member by ID.

**Parameters**:
```typescript
{
  memberId: string;  // Member UUID
}
```

**Returns**: `Result<Member | null, String>`

**Example**:
```typescript
const member = await invoke('get_member', {
  memberId: 'member-uuid'
});
```

---

##### `list_members`

List all members.

**Parameters**: None

**Returns**: `Result<Member[], String>`

**Example**:
```typescript
const members = await invoke('list_members');
console.log(`Total members: ${members.length}`);
```

---

##### `update_member_role`

Update a member's role.

**Parameters**:
```typescript
{
  memberId: string;  // Member UUID
  newRole: string;   // New role ("admin", "member", "viewer")
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('update_member_role', {
  memberId: 'member-uuid',
  newRole: 'admin'
});
```

---

## Document Sync Commands

### Module: `org_commands`

CRDT synchronization using state vectors and diffs.

##### `get_channel_state_vector`

Get the current state vector for a channel's CRDT document.

State vectors are compact representations of document state for efficient sync.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
}
```

**Returns**: `Result<Uint8Array, String>` - Binary state vector

**Example**:
```typescript
const stateVector = await invoke('get_channel_state_vector', {
  channelId: 'channel-uuid'
});
// Send to remote peer for comparison
```

---

##### `get_channel_diff`

Get the diff between local state and a remote state vector.

Returns only the operations the remote peer is missing.

**Parameters**:
```typescript
{
  channelId: string;          // Channel UUID
  remoteStateVector: Uint8Array;  // Remote peer's state vector
}
```

**Returns**: `Result<Uint8Array, String>` - Binary diff (CRDT update)

**Example**:
```typescript
const diff = await invoke('get_channel_diff', {
  channelId: 'channel-uuid',
  remoteStateVector: receivedStateVector
});
// Send diff to remote peer
```

---

##### `apply_channel_diff`

Apply a diff received from a remote peer.

**Parameters**:
```typescript
{
  channelId: string;      // Channel UUID
  diff: Uint8Array;  // Binary diff from remote peer
}
```

**Returns**: `Result<AppliedDiffResult, String>`

```typescript
interface AppliedDiffResult {
  applied: boolean;         // Whether diff was successfully applied
  newMessageCount: number;  // Number of new messages added
  conflicts: number;        // Number of conflicts resolved
}
```

**Example**:
```typescript
const result = await invoke('apply_channel_diff', {
  channelId: 'channel-uuid',
  diff: receivedDiff
});
console.log(`Applied ${result.newMessageCount} new messages`);
```

---

##### `get_channel_sync_update`

Get full sync update for a channel (less efficient than state vector approach).

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
}
```

**Returns**: `Result<Uint8Array, String>` - Full CRDT update

**Example**:
```typescript
const update = await invoke('get_channel_sync_update', {
  channelId: 'channel-uuid'
});
```

---

##### `apply_channel_sync_update`

Apply a full sync update to a channel.

**Parameters**:
```typescript
{
  channelId: string;  // Channel UUID
  update: Uint8Array; // Full CRDT update
}
```

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('apply_channel_sync_update', {
  channelId: 'channel-uuid',
  update: receivedUpdate
});
```

---

##### `get_issue_sync_update`

Get full sync update for an issue.

**Parameters**:
```typescript
{
  issueId: string;  // Issue UUID
}
```

**Returns**: `Result<Uint8Array, String>`

---

##### `apply_issue_sync_update`

Apply a full sync update to an issue.

**Parameters**:
```typescript
{
  issueId: string;    // Issue UUID
  update: Uint8Array; // Full CRDT update
}
```

**Returns**: `Result<(), String>`

---

## Utility Commands

### General Commands

##### `health`

Health check endpoint.

**Parameters**: None

**Returns**: `Result<object, String>`

```typescript
{
  status: "ok",
  app: "0.1.0"  // Application version
}
```

**Example**:
```typescript
const health = await invoke('health');
console.log('App version:', health.app);
```

---

##### `get_app_version`

Get application version.

**Parameters**: None

**Returns**: `Result<string, String>` - Version string (e.g., "0.1.0")

**Example**:
```typescript
const version = await invoke('get_app_version');
console.log('Running version:', version);
```

---

## Error Handling

All Tauri commands return `Result<T, String>` where errors are formatted strings.

### Common Error Patterns

```typescript
// Service not initialized
"Auth service not initialized. Call auth_initialize first."

// Invalid credentials
"Login failed: Invalid password"
"Login failed: Vault not found"

// Session errors
"Session not found or expired"

// Four-word validation
"Invalid four-word address: word not in dictionary"

// Not found errors
"Channel not found"
"Issue not found"
"Member not found"

// Permission errors
"Insufficient permissions"
"User not authorized"

// Network errors
"Failed to connect: connection timeout"
"Peer not reachable"
```

### Error Handling Best Practices

```typescript
try {
  const result = await invoke('command_name', params);
  // Handle success
} catch (error) {
  // error is a string
  if (error.includes('not initialized')) {
    // Reinitialize service
    await invoke('auth_initialize');
    // Retry command
  } else if (error.includes('Invalid password')) {
    // Show password error to user
    showError('Invalid password. Please try again.');
  } else {
    // Generic error handling
    console.error('Command failed:', error);
    showError(`Operation failed: ${error}`);
  }
}
```

---

## Performance Tips

### Efficient CRDT Synchronization

**Bad** (Full sync every time):
```typescript
const update = await invoke('get_channel_sync_update', { channelId });
await sendToPeer(update);
```

**Good** (State vector diff):
```typescript
const localStateVector = await invoke('get_channel_state_vector', { channelId });
const remoteStateVector = await receiveFromPeer();
const diff = await invoke('get_channel_diff', { channelId, remoteStateVector });
await sendToPeer(diff);
```

### Batch Operations

**Bad** (N round trips):
```typescript
for (const channelId of channelIds) {
  const channel = await invoke('get_channel', { channelId });
  channels.push(channel);
}
```

**Good** (1 round trip):
```typescript
const channels = await invoke('list_channels', { orgId });
```

### Pagination

Always use `limit` and `offset` for large result sets:

```typescript
// Load first page
const messages = await invoke('get_messages', {
  channelId,
  limit: 50,
  offset: 0
});

// Load more on scroll
const moreMessages = await invoke('get_messages', {
  channelId,
  limit: 50,
  offset: 50
});
```

---

## See Also

- [Frontend API Reference](frontend-api.md) - TypeScript/React APIs
- [Core API Reference](core-api.md) - Rust library APIs
- [Bridge API Reference](bridge-api.md) - HTTP/REST APIs
- [Authentication Guide](../guides/authentication.md) - Security details
- [Testing Guide](../guides/testing.md) - Testing strategies

---

**Tauri Commands**: Desktop application integration with type-safe IPC. 🚀
