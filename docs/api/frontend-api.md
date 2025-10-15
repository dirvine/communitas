# Frontend API Reference

TypeScript/React API reference for the Communitas frontend.

## Overview

The frontend provides React Context APIs, service modules, and utility functions for building the Communitas user interface. This document covers the public TypeScript API for frontend development.

**Tech Stack**:
- React 18 with TypeScript
- Material-UI (MUI) components
- Vite build system
- Tauri v2 for desktop integration

---

## React Context APIs

### AuthContext

Authentication state and user session management.

```typescript
import { useAuth } from '@/contexts/AuthContext';

interface AuthContextType {
  // State
  user: SessionInfo | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  // Actions
  login: (fourWords: string, password: string) => Promise<SessionInfo>;
  logout: () => Promise<void>;
  createVault: (fourWords: string, password: string, displayName: string) => Promise<string>;

  // Passkey/Biometric
  registerPasskey: (fourWords: string, deviceName: string, password: string) => Promise<void>;
  loginWithPasskey: (fourWords: string) => Promise<SessionInfo>;

  // Auto-login
  tryAutoLogin: () => Promise<SessionInfo | null>;
}
```

**Usage**:
```typescript
function LoginForm() {
  const { login, isLoading } = useAuth();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await login(fourWords, password);
      // Redirect to dashboard
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <input name="fourWords" placeholder="ocean-forest-moon-star" />
      <input type="password" name="password" />
      <button type="submit" disabled={isLoading}>Login</button>
    </form>
  );
}
```

---

### NavigationContext

App navigation and routing state.

```typescript
import { useNavigation } from '@/contexts/NavigationContext';

interface NavigationContextType {
  // Current state
  currentPage: string;
  currentChannel: string | null;
  currentProject: string | null;

  // Actions
  navigateTo: (page: string) => void;
  setCurrentChannel: (channelId: string) => void;
  setCurrentProject: (projectId: string) => void;
  goBack: () => void;
}
```

**Usage**:
```typescript
function ChannelList() {
  const { setCurrentChannel, navigateTo } = useNavigation();

  const handleChannelClick = (channelId: string) => {
    setCurrentChannel(channelId);
    navigateTo('channel');
  };

  return (
    <div>
      {channels.map(ch => (
        <div key={ch.id} onClick={() => handleChannelClick(ch.id)}>
          {ch.name}
        </div>
      ))}
    </div>
  );
}
```

---

### EncryptionContext

Client-side encryption and cryptography.

```typescript
import { useEncryption } from '@/contexts/EncryptionContext';

interface EncryptionContextType {
  // Encryption
  encrypt: (plaintext: string, key: Uint8Array) => Promise<Uint8Array>;
  decrypt: (ciphertext: Uint8Array, key: Uint8Array) => Promise<string>;

  // Key derivation
  deriveKey: (password: string, salt: Uint8Array) => Promise<Uint8Array>;

  // Hashing
  hash: (data: Uint8Array) => Promise<Uint8Array>;
}
```

**Usage**:
```typescript
function SecureMessageSender() {
  const { encrypt, deriveKey } = useEncryption();

  const sendEncrypted = async (message: string, password: string) => {
    const salt = crypto.getRandomValues(new Uint8Array(16));
    const key = await deriveKey(password, salt);
    const encrypted = await encrypt(message, key);
    await sendToBackend(encrypted);
  };

  return <MessageComposer onSend={sendEncrypted} />;
}
```

---

## Service APIs

### BackendService

Main interface for Tauri backend communication.

```typescript
import { BackendService } from '@/services/api/BackendService';

class BackendService {
  // Authentication
  static async initialize(): Promise<void>;
  static async login(fourWords: string, password: string): Promise<SessionInfo>;
  static async logout(): Promise<void>;

  // Channels
  static async createChannel(orgId: string, name: string, description?: string): Promise<Channel>;
  static async listChannels(orgId: string): Promise<Channel[]>;
  static async getMessages(channelId: string, limit?: number, offset?: number): Promise<Message[]>;

  // Messages
  static async sendMessage(channelId: string, authorId: string, content: string): Promise<Message>;
  static async editMessage(messageId: string, newContent: string): Promise<Message>;
  static async deleteMessage(messageId: string): Promise<void>;

  // Projects
  static async createProject(orgId: string, name: string, options: ProjectOptions): Promise<Project>;
  static async listIssues(projectId: string): Promise<Issue[]>;
}
```

**Usage**:
```typescript
async function loadChannelMessages(channelId: string) {
  try {
    const messages = await BackendService.getMessages(channelId, 50, 0);
    setMessages(messages);
  } catch (error) {
    console.error('Failed to load messages:', error);
  }
}
```

---

### NetworkConnectionService

Network connectivity and P2P status.

```typescript
import { networkService } from '@/services/network/NetworkConnectionService';

class NetworkConnectionService {
  // Connection management
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  retry(): Promise<void>;

  // State
  getState(): NetworkState;
  isConnected(): boolean;

  // Subscriptions
  subscribe(callback: (state: NetworkState) => void): () => void;
}

interface NetworkState {
  status: 'connecting' | 'connected' | 'offline' | 'local' | 'error';
  peerCount: number;
  lastError: string | null;
}
```

**Usage**:
```typescript
function NetworkStatusIndicator() {
  const [state, setState] = useState(networkService.getState());

  useEffect(() => {
    const unsubscribe = networkService.subscribe(setState);
    return unsubscribe;
  }, []);

  return (
    <div className={state.status}>
      {state.status === 'connected' ? `${state.peerCount} peers` : state.status}
    </div>
  );
}
```

---

### OfflineStorageService

Offline-first local storage with sync.

```typescript
import { offlineStorage } from '@/services/storage/OfflineStorageService';

class OfflineStorageService {
  // Storage
  store(key: string, data: any, options?: StorageOptions): Promise<void>;
  get(key: string): Promise<any | null>;
  delete(key: string): Promise<void>;
  list(prefix: string): Promise<string[]>;

  // Sync queue
  queueForSync(operation: SyncOperation): Promise<void>;
  processSyncQueue(): Promise<void>;

  // Cache
  clearCache(): Promise<void>;
}

interface StorageOptions {
  ttl?: number;           // Time-to-live in milliseconds
  encrypt?: boolean;      // Encrypt data
  syncOnline?: boolean;   // Sync when network returns
}
```

**Usage**:
```typescript
async function saveMessageDraft(content: string) {
  await offlineStorage.store('message-draft', content, {
    ttl: 3600000, // 1 hour
    syncOnline: true
  });
}

async function loadMessageDraft(): Promise<string | null> {
  return await offlineStorage.get('message-draft');
}
```

---

### ChannelService

Channel-specific operations.

```typescript
import { channelService } from '@/services/channelService';

class ChannelService {
  // Channel CRUD
  createChannel(orgId: string, name: string, description?: string): Promise<Channel>;
  getChannel(channelId: string): Promise<Channel | null>;
  updateChannel(channelId: string, updates: Partial<Channel>): Promise<Channel>;

  // Messages
  sendMessage(channelId: string, content: string, threadId?: string): Promise<Message>;
  getMessages(channelId: string, options: PaginationOptions): Promise<Message[]>;
  subscribeToMessages(channelId: string, callback: (message: Message) => void): () => void;

  // Members
  addMember(channelId: string, userId: string, role: string): Promise<void>;
  removeMember(channelId: string, userId: string): Promise<void>;
  getMembers(channelId: string): Promise<ChannelMember[]>;
}
```

**Usage**:
```typescript
function ChannelMessages({ channelId }: { channelId: string }) {
  const [messages, setMessages] = useState<Message[]>([]);

  useEffect(() => {
    // Load initial messages
    channelService.getMessages(channelId, { limit: 50 })
      .then(setMessages);

    // Subscribe to new messages
    const unsubscribe = channelService.subscribeToMessages(
      channelId,
      (message) => setMessages(prev => [...prev, message])
    );

    return unsubscribe;
  }, [channelId]);

  return (
    <div>
      {messages.map(msg => (
        <MessageCard key={msg.id} message={msg} />
      ))}
    </div>
  );
}
```

---

### ChannelSyncService

Real-time CRDT synchronization for channels.

```typescript
import { channelSyncService } from '@/services/channelSyncService';

class ChannelSyncService {
  // Sync operations
  syncChannel(channelId: string): Promise<void>;
  getStateVector(channelId: string): Promise<Uint8Array>;
  applyDiff(channelId: string, diff: Uint8Array): Promise<void>;

  // Subscriptions
  onSync(channelId: string, callback: () => void): () => void;
}
```

**Usage**:
```typescript
useEffect(() => {
  // Sync channel on load
  channelSyncService.syncChannel(channelId);

  // Subscribe to sync events
  const unsubscribe = channelSyncService.onSync(channelId, () => {
    console.log('Channel synced');
    refreshMessages();
  });

  return unsubscribe;
}, [channelId]);
```

---

## Utility Functions

### Input Sanitization

```typescript
import { sanitizeInput, sanitizeHTML } from '@/services/security/inputSanitization';

// Sanitize user input
const safe = sanitizeInput(userInput);

// Sanitize HTML (removes scripts, dangerous attributes)
const safeHTML = sanitizeHTML(htmlContent);
```

### Cryptography

```typescript
import { CryptoManager } from '@/services/security/cryptoManager';

const crypto = new CryptoManager();

// Generate keypair
const { publicKey, privateKey } = await crypto.generateKeyPair();

// Encrypt/Decrypt
const encrypted = await crypto.encrypt(message, publicKey);
const decrypted = await crypto.decrypt(encrypted, privateKey);

// Sign/Verify
const signature = await crypto.sign(message, privateKey);
const valid = await crypto.verify(message, signature, publicKey);
```

### Four-Word Validation

```typescript
import { validateFourWords, suggestCorrections } from '@/utils/fourWords';

// Validate address
if (!validateFourWords('ocean-forest-moon-star')) {
  console.error('Invalid four-word address');
}

// Get suggestions for typos
const suggestions = suggestCorrections('occean-forest-moon-star');
// → ['ocean-forest-moon-star']
```

---

## Type Definitions

```typescript
// Session
interface SessionInfo {
  sessionId: string;
  fourWords: string;
  displayName: string;
}

// Channel
interface Channel {
  id: string;
  orgId: string;
  name: string;
  description: string | null;
  createdAt: number;
  memberCount: number;
}

// Message
interface Message {
  id: string;
  channelId: string;
  authorId: string;
  content: string;
  createdAt: number;
  updatedAt: number | null;
  threadId: string | null;
  isDeleted: boolean;
}

// Project
interface Project {
  id: string;
  orgId: string;
  name: string;
  description: string | null;
  icon: string | null;
  color: string | null;
  createdAt: number;
}

// Issue
interface Issue {
  id: string;
  projectId: string;
  title: string;
  description: string | null;
  status: 'backlog' | 'todo' | 'in-progress' | 'done' | 'canceled';
  priority: 'urgent' | 'high' | 'medium' | 'low';
  assigneeId: string | null;
  createdAt: number;
}
```

---

## Testing

### Component Testing

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { AuthProvider } from '@/contexts/AuthContext';

describe('LoginForm', () => {
  it('should login successfully', async () => {
    render(
      <AuthProvider>
        <LoginForm />
      </AuthProvider>
    );

    // Fill form
    const fourWordsInput = screen.getByPlaceholderText(/four-word/i);
    const passwordInput = screen.getByLabelText(/password/i);
    userEvent.type(fourWordsInput, 'ocean-forest-moon-star');
    userEvent.type(passwordInput, 'password123');

    // Submit
    const submitButton = screen.getByRole('button', { name: /login/i });
    userEvent.click(submitButton);

    // Wait for success
    await waitFor(() => {
      expect(screen.getByText(/welcome/i)).toBeInTheDocument();
    });
  });
});
```

### Service Testing

```typescript
import { BackendService } from '@/services/api/BackendService';
import { vi } from 'vitest';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn()
}));

describe('BackendService', () => {
  it('should create channel', async () => {
    const mockChannel = { id: 'ch-1', name: 'General' };
    invoke.mockResolvedValueOnce(mockChannel);

    const channel = await BackendService.createChannel('org-1', 'General');

    expect(invoke).toHaveBeenCalledWith('create_channel', {
      orgId: 'org-1',
      name: 'General'
    });
    expect(channel).toEqual(mockChannel);
  });
});
```

---

## See Also

- [Tauri Commands API](tauri-commands.md) - Backend IPC interface
- [Core API](core-api.md) - Rust library API
- [Authentication Guide](../guides/authentication.md) - Security details
- [Testing Guide](../guides/testing.md) - Testing strategies

---

**Frontend API**: Build beautiful UIs with React and TypeScript. ⚛️✨
