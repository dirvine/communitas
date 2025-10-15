# Coding Standards

Comprehensive coding standards for Communitas development.

## Table of Contents

- [General Principles](#general-principles)
- [Rust Standards](#rust-standards)
- [TypeScript Standards](#typescript-standards)
- [Testing Standards](#testing-standards)
- [Documentation Standards](#documentation-standards)
- [Git Standards](#git-standards)
- [Security Standards](#security-standards)

---

## General Principles

### Code Philosophy

**Clarity over Cleverness**:
- Write code that is easy to understand
- Favor explicit over implicit
- Use descriptive names
- Add comments for complex logic

**Correctness over Performance**:
- Make it work correctly first
- Profile before optimizing
- Document performance-critical sections
- Maintain test coverage during optimization

**Safety First**:
- Validate all inputs
- Handle all error cases
- Use type system to prevent bugs
- Never suppress compiler warnings without good reason

---

## Rust Standards

### Error Handling

**ZERO TOLERANCE for Production Code**:
```rust
// ❌ FORBIDDEN - These patterns are BANNED in production code
value.unwrap()                    // Panic on None/Err
value.expect("message")           // Panic with message
panic!("error")                   // Direct panic
todo!()                          // Unimplemented code marker
unimplemented!()                 // Unimplemented code marker
unreachable!()                   // Unless proven unreachable
```

**REQUIRED - Proper Error Handling**:
```rust
// ✅ CORRECT - Use Result and ? operator
pub fn process_data(input: &str) -> Result<String, Error> {
    let value = parse_value(input)?;
    let result = transform(value)?;
    Ok(result)
}

// ✅ CORRECT - Convert Option to Result
pub fn get_item(id: &str) -> Result<Item, Error> {
    items.get(id)
        .ok_or(Error::NotFound { id: id.to_string() })?
        .clone()
}

// ✅ CORRECT - Map errors with context
pub fn load_config(path: &Path) -> Result<Config, Error> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::ConfigLoad {
            path: path.to_path_buf(),
            source: e,
        })?;

    serde_json::from_str(&content)
        .map_err(Error::ConfigParse)?
}
```

**EXCEPTION - Test Code Only**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        // ✅ OK in tests - Makes tests more readable
        let result = parse("test").unwrap();
        assert_eq!(result.value, "expected");

        // ✅ OK in tests
        let item = get_item("id").expect("item should exist in test");
        assert!(item.is_valid());
    }
}
```

### Error Types

**Use thiserror for Library Code**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication failed: invalid credentials")]
    InvalidCredentials,

    #[error("Vault not found: {four_words}")]
    VaultNotFound { four_words: String },

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Cryptography error: {0}")]
    Crypto(#[from] CryptoError),
}

// Usage
pub fn login(four_words: &str, password: &str) -> Result<Session, AuthError> {
    let vault = storage.get_vault(four_words)
        .ok_or(AuthError::VaultNotFound {
            four_words: four_words.to_string()
        })?;

    vault.verify_password(password)
        .ok_or(AuthError::InvalidCredentials)?;

    Ok(Session::new(vault))
}
```

**Use anyhow for Application Code**:
```rust
use anyhow::{Context, Result};

pub async fn initialize_app(config_path: &Path) -> Result<App> {
    let config = load_config(config_path)
        .context("Failed to load configuration")?;

    let storage = EncryptedStorageManager::new(config.storage)
        .context("Failed to initialize storage")?;

    let auth = AuthService::new(storage)
        .context("Failed to initialize auth service")?;

    Ok(App { config, auth })
}
```

### Naming Conventions

**Functions and Methods**:
```rust
// ✅ CORRECT - snake_case, verb-based
pub fn create_vault(name: &str) -> Result<Vault>;
pub fn get_user_by_id(id: &str) -> Option<User>;
pub async fn send_message(channel_id: &str, content: &str) -> Result<Message>;

// ❌ INCORRECT - Wrong case or naming
pub fn CreateVault(name: &str) -> Result<Vault>;  // PascalCase
pub fn User(id: &str) -> Option<User>;             // Noun, not verb
```

**Types and Structs**:
```rust
// ✅ CORRECT - PascalCase, descriptive nouns
pub struct AuthService;
pub struct EncryptedVault;
pub enum NetworkStatus { Connected, Disconnected, Error }

// ❌ INCORRECT - Wrong case
pub struct auth_service;  // snake_case
pub enum network_status;  // snake_case
```

**Constants**:
```rust
// ✅ CORRECT - SCREAMING_SNAKE_CASE
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;  // 1 MB
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ❌ INCORRECT - Wrong case
pub const maxMessageSize: usize = 1024;  // camelCase
pub const default_timeout: Duration = ...;  // snake_case
```

**Modules**:
```rust
// ✅ CORRECT - snake_case, descriptive
mod encrypted_storage;
mod auth_service;
mod member_manager;

// ❌ INCORRECT - Wrong case
mod EncryptedStorage;  // PascalCase
mod authService;       // camelCase
```

### Code Organization

**Module Structure**:
```rust
// src/auth_service.rs

// Imports - grouped and sorted
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::encrypted_storage::EncryptedStorageManager;
use crate::types::{SessionInfo, VaultInfo};

// Type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub four_words: String,
    pub display_name: String,
}

// Error types
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication failed")]
    AuthenticationFailed,
}

// Public structs
pub struct AuthService {
    storage: EncryptedStorageManager,
    sessions: HashMap<String, SessionInfo>,
}

// Implementation
impl AuthService {
    // Associated functions first (constructors)
    pub fn new(storage: EncryptedStorageManager) -> Self {
        Self {
            storage,
            sessions: HashMap::new(),
        }
    }

    // Public methods
    pub async fn login(&mut self, four_words: &str, password: &str) -> Result<SessionInfo, AuthError> {
        // Implementation
    }

    // Private methods
    fn validate_credentials(&self, four_words: &str, password: &str) -> bool {
        // Implementation
    }
}

// Tests at the end
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        // Tests
    }
}
```

### Clippy Configuration

**Required Lints** (Enforced in CI):
```bash
cargo clippy --all-features --all-targets -- \
  -D clippy::panic \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D warnings
```

**Allowed Suppressions** (Rare, with Justification):
```rust
// ✅ CORRECT - Specific suppression with reason
#[allow(clippy::too_many_arguments)]  // Constructor requires all parameters
pub fn new(
    param1: String,
    param2: usize,
    // ... 8 total parameters
) -> Self {
    // Implementation
}

// ❌ INCORRECT - Blanket suppression
#![allow(clippy::all)]  // Too broad, no justification
```

**NOT Required**:
- `clippy::pedantic` - Too strict for practical development
- Encourages good practices but not enforced

### Formatting

**rustfmt Configuration** (.rustfmt.toml):
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

**Format Before Every Commit**:
```bash
cargo fmt --all
```

### Async Code

**Async Function Patterns**:
```rust
// ✅ CORRECT - Proper async/await
pub async fn send_message(channel_id: &str, content: &str) -> Result<Message> {
    let channel = get_channel(channel_id).await?;
    let message = channel.create_message(content).await?;
    message.broadcast().await?;
    Ok(message)
}

// ✅ CORRECT - Spawn for concurrent operations
pub async fn sync_all_channels(channels: Vec<String>) -> Result<()> {
    let handles: Vec<_> = channels
        .into_iter()
        .map(|ch| tokio::spawn(sync_channel(ch)))
        .collect();

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

// ❌ INCORRECT - Blocking in async
pub async fn read_file(path: &Path) -> Result<String> {
    // ❌ Blocks the async runtime
    std::fs::read_to_string(path)
        .map_err(Into::into)
}

// ✅ CORRECT - Use async file operations
pub async fn read_file(path: &Path) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(Into::into)
}
```

---

## TypeScript Standards

### Type Safety

**NO `any` Type**:
```typescript
// ❌ FORBIDDEN
function process(data: any) {
  return data.value;
}

// ✅ CORRECT - Define proper types
interface Data {
  id: string;
  value: number;
}

function process(data: Data): number {
  return data.value;
}

// ✅ CORRECT - Use generics for flexibility
function process<T extends { value: number }>(data: T): number {
  return data.value;
}
```

**NO Type Suppression**:
```typescript
// ❌ FORBIDDEN - Suppressing TypeScript errors
// @ts-ignore
const value = dangerousFunction();

// @ts-expect-error
const result = brokenFunction();

// ✅ CORRECT - Fix the root cause
const value = safeFunction() as ExpectedType;

// ✅ CORRECT - Use type guards
if (isExpectedType(value)) {
  const result = value.property;
}
```

### Naming Conventions

**Variables and Functions**:
```typescript
// ✅ CORRECT - camelCase, descriptive
const userId = 'user-123';
const messageCount = 42;

function getUserById(id: string): User | null {
  return users.find(u => u.id === id) ?? null;
}

async function sendMessage(channelId: string, content: string): Promise<Message> {
  // Implementation
}

// ❌ INCORRECT - Wrong naming
const user_id = 'user-123';  // snake_case
const msg_cnt = 42;          // Abbreviated
function User(id: string) { }  // PascalCase for function
```

**Types and Interfaces**:
```typescript
// ✅ CORRECT - PascalCase, descriptive
interface UserProfile {
  id: string;
  name: string;
  email: string;
}

type NetworkStatus = 'connected' | 'disconnected' | 'error';

class AuthService {
  // Implementation
}

// ❌ INCORRECT - Wrong case or naming
interface user_profile { }  // snake_case
type networkStatus = ...;   // camelCase
class auth_service { }      // snake_case
```

**Constants**:
```typescript
// ✅ CORRECT - SCREAMING_SNAKE_CASE for true constants
const MAX_RETRIES = 3;
const API_BASE_URL = 'https://api.example.com';
const DEFAULT_TIMEOUT = 30000;

// ✅ CORRECT - camelCase for config objects
const apiConfig = {
  baseUrl: 'https://api.example.com',
  timeout: 30000,
  retries: 3,
};

// ❌ INCORRECT - Inconsistent naming
const max_retries = 3;       // snake_case
const Api_Base_Url = '...';  // Mixed case
```

### React Patterns

**Function Components** (Preferred):
```typescript
// ✅ CORRECT - Function component with TypeScript
interface LoginFormProps {
  onSubmit: (credentials: Credentials) => Promise<void>;
  isLoading?: boolean;
}

export const LoginForm: FC<LoginFormProps> = ({ onSubmit, isLoading = false }) => {
  const [fourWords, setFourWords] = useState('');
  const [password, setPassword] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSubmit({ fourWords, password });
  };

  return (
    <form onSubmit={handleSubmit}>
      <input
        value={fourWords}
        onChange={(e) => setFourWords(e.target.value)}
        placeholder="ocean-forest-moon-star"
      />
      <input
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
      />
      <button type="submit" disabled={isLoading}>
        {isLoading ? 'Loading...' : 'Login'}
      </button>
    </form>
  );
};
```

**Hooks Best Practices**:
```typescript
// ✅ CORRECT - Extract custom hooks
function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}

// ✅ CORRECT - Proper dependency arrays
useEffect(() => {
  const fetchData = async () => {
    const data = await loadMessages(channelId);
    setMessages(data);
  };

  fetchData();
}, [channelId]);  // Include all dependencies

// ❌ INCORRECT - Missing dependencies
useEffect(() => {
  loadMessages(channelId);
}, []);  // Missing channelId - will use stale value
```

### Code Organization

**File Structure**:
```typescript
// src/components/LoginForm.tsx

// Imports - grouped and sorted
import { FC, useState } from 'react';
import { Button, TextField } from '@mui/material';

import { useAuth } from '@/contexts/AuthContext';
import { validateFourWords } from '@/utils/validation';
import type { Credentials } from '@/types/auth';

// Constants
const MIN_PASSWORD_LENGTH = 8;

// Types
interface LoginFormProps {
  onSuccess?: () => void;
}

// Component
export const LoginForm: FC<LoginFormProps> = ({ onSuccess }) => {
  // Implementation
};

// Styles (if using styled-components)
const StyledForm = styled.form`
  /* Styles */
`;
```

### Async Patterns

**Promise Handling**:
```typescript
// ✅ CORRECT - Async/await with proper error handling
async function loadUserData(userId: string): Promise<User> {
  try {
    const user = await invoke<User>('get_user', { userId });
    return user;
  } catch (error) {
    console.error('Failed to load user:', error);
    throw new Error('User load failed');
  }
}

// ✅ CORRECT - Parallel operations
async function loadDashboardData(): Promise<DashboardData> {
  const [user, messages, notifications] = await Promise.all([
    loadUser(),
    loadMessages(),
    loadNotifications(),
  ]);

  return { user, messages, notifications };
}

// ❌ INCORRECT - Sequential when could be parallel
async function loadDashboardData(): Promise<DashboardData> {
  const user = await loadUser();
  const messages = await loadMessages();  // Could run in parallel
  const notifications = await loadNotifications();  // Could run in parallel

  return { user, messages, notifications };
}
```

---

## Testing Standards

### Test Organization

**File Naming**:
```
src/
├── auth_service.rs           # Implementation
└── auth_service.test.rs      # Unit tests (Rust - alternative)

src/components/
├── LoginForm.tsx             # Implementation
└── LoginForm.test.tsx        # Unit tests (TypeScript)

tests/
├── integration_auth.rs       # Integration tests (Rust)
└── e2e/
    └── auth.spec.ts         # E2E tests (TypeScript)
```

### Test Structure

**Rust Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures
    fn setup_test_storage() -> EncryptedStorageManager {
        // Setup
    }

    // Unit tests
    #[test]
    fn test_create_vault_success() {
        let mut storage = setup_test_storage();

        let result = storage.create_vault("test", "password", "User");

        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_create_vault_duplicate() {
        let mut storage = setup_test_storage();
        storage.create_vault("test", "password", "User").unwrap();

        let result = storage.create_vault("test", "password", "User");

        assert!(result.is_err());
    }

    // Async tests
    #[tokio::test]
    async fn test_async_operation() {
        let service = setup_service().await;

        let result = service.perform_operation().await;

        assert!(result.is_ok());
    }
}
```

**TypeScript Tests**:
```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LoginForm } from './LoginForm';

describe('LoginForm', () => {
  // Test setup
  const mockOnSubmit = vi.fn();

  beforeEach(() => {
    mockOnSubmit.mockClear();
  });

  // Unit tests
  it('should render form fields', () => {
    render(<LoginForm onSubmit={mockOnSubmit} />);

    expect(screen.getByPlaceholderText(/four-word/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
  });

  it('should call onSubmit with credentials', async () => {
    render(<LoginForm onSubmit={mockOnSubmit} />);

    await userEvent.type(
      screen.getByPlaceholderText(/four-word/i),
      'ocean-forest-moon-star'
    );
    await userEvent.type(
      screen.getByLabelText(/password/i),
      'password123'
    );
    await userEvent.click(screen.getByRole('button', { name: /login/i }));

    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalledWith({
        fourWords: 'ocean-forest-moon-star',
        password: 'password123',
      });
    });
  });
});
```

### Test Coverage

**Minimum Coverage**:
- Critical paths: 90%+
- General code: 80%+
- UI components: 70%+

**Generate Coverage Reports**:
```bash
# Rust
cargo tarpaulin --all-features --workspace --out Html

# TypeScript
npm run test:coverage
```

---

## Documentation Standards

### Rust Documentation

**Module Documentation**:
```rust
//! Authentication service for Communitas.
//!
//! This module provides secure authentication using encrypted vaults
//! and platform-specific credential storage.
//!
//! # Examples
//!
//! ```rust
//! use communitas_core::AuthService;
//!
//! let auth = AuthService::new(storage_manager);
//! let session = auth.login("ocean-forest-moon-star", "password").await?;
//! println!("Logged in as: {}", session.display_name);
//! ```

use std::collections::HashMap;
```

**Function Documentation**:
```rust
/// Creates a new encrypted vault for the given identity.
///
/// # Arguments
///
/// * `four_words` - Four-word identity (must be valid dictionary words)
/// * `password` - User password (minimum 8 characters)
/// * `display_name` - Human-readable name for the identity
///
/// # Returns
///
/// * `Ok(String)` - Vault ID on success
/// * `Err(Error)` - Error if vault creation fails
///
/// # Errors
///
/// Returns error if:
/// - Four-word address is invalid
/// - Password is too weak
/// - Vault already exists
/// - Storage operation fails
///
/// # Examples
///
/// ```rust
/// let vault_id = auth.create_vault(
///     "ocean-forest-moon-star",
///     "MySecurePassword123!",
///     "Alice Johnson"
/// ).await?;
/// ```
pub async fn create_vault(
    &mut self,
    four_words: &str,
    password: &str,
    display_name: &str,
) -> Result<String, Error> {
    // Implementation
}
```

### TypeScript Documentation

**JSDoc Comments**:
```typescript
/**
 * Authentication context providing user session management.
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { user, login, logout } = useAuth();
 *
 *   if (!user) {
 *     return <LoginForm onSubmit={login} />;
 *   }
 *
 *   return <Dashboard user={user} onLogout={logout} />;
 * }
 * ```
 */
export interface AuthContextType {
  /** Current authenticated user, null if not logged in */
  user: SessionInfo | null;

  /** Whether user is authenticated */
  isAuthenticated: boolean;

  /** Whether authentication operation is in progress */
  isLoading: boolean;

  /**
   * Authenticates user with four-word address and password.
   *
   * @param fourWords - Four-word identity (e.g., "ocean-forest-moon-star")
   * @param password - User password
   * @returns Session information on success
   * @throws {AuthError} If credentials are invalid
   */
  login: (fourWords: string, password: string) => Promise<SessionInfo>;

  /**
   * Logs out current user and clears session.
   */
  logout: () => Promise<void>;
}
```

---

## Git Standards

### Commit Messages

**Format**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code restructuring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples**:
```
feat(auth): add passkey authentication

- Implement WebAuthn registration
- Add Touch ID support for macOS
- Update authentication flow UI
- Add comprehensive tests

Closes #123
```

```
fix(network): resolve connection timeout issue

The connection timeout wasn't being properly enforced,
causing long hangs when peers were unreachable.

- Set proper timeout in QUIC configuration
- Add retry logic with exponential backoff
- Improve error messages

Fixes #456
```

### Branch Naming

**Feature Branches**:
```
feature/passkey-auth
feature/channel-threads
feature/member-management
```

**Bug Fix Branches**:
```
fix/login-timeout
fix/message-encoding
fix/network-reconnect
```

**Documentation Branches**:
```
docs/api-reference
docs/architecture-guide
docs/update-readme
```

---

## Security Standards

### Input Validation

**Validate All Inputs**:
```rust
pub fn validate_four_words(words: &str) -> Result<(), ValidationError> {
    // Check format
    if !words.contains('-') {
        return Err(ValidationError::InvalidFormat);
    }

    // Check word count
    let parts: Vec<&str> = words.split('-').collect();
    if parts.len() != 4 {
        return Err(ValidationError::InvalidWordCount);
    }

    // Check dictionary
    for word in parts {
        if !is_valid_word(word) {
            return Err(ValidationError::InvalidWord { word: word.to_string() });
        }
    }

    Ok(())
}
```

### Secure Storage

**Never Log Sensitive Data**:
```rust
// ❌ FORBIDDEN - Logging sensitive data
debug!("Login attempt: {} with password {}", four_words, password);

// ✅ CORRECT - Redact sensitive data
debug!("Login attempt for user: [REDACTED]");
info!("Authentication successful for user: {}", four_words);
```

**Use Platform Keyring**:
```rust
// ✅ CORRECT - Store credentials securely
storage_manager.store_password_in_keyring(four_words, password).await?;

// ❌ INCORRECT - Storing credentials in plain text
fs::write("passwords.txt", format!("{}: {}", four_words, password))?;
```

### Cryptography

**Use Audited Libraries Only**:
```rust
// ✅ CORRECT - Use saorsa-pqc (audited)
use saorsa_pqc::mldsa::MlDsa65;
use saorsa_pqc::mlkem::MlKem768;

let keypair = MlDsa65::generate()?;
let signature = keypair.sign(message)?;

// ❌ INCORRECT - Rolling your own crypto
fn my_custom_encryption(data: &[u8], key: &[u8]) -> Vec<u8> {
    // Custom implementation - DON'T DO THIS
}
```

---

## See Also

- [Development Guide](README.md) - Complete development guide
- [Contributing](contributing.md) - How to contribute
- [API Reference](../api/README.md) - API documentation
- [Architecture](../architecture/README.md) - System architecture

---

**Coding Standards**: Write code that makes the team proud. ✨🎯
