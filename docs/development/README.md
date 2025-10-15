# Development Guide

Comprehensive guide for developers contributing to Communitas.

## Table of Contents

- [Quick Start](#quick-start)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing Strategy](#testing-strategy)
- [Code Quality](#code-quality)
- [Common Tasks](#common-tasks)
- [Debugging](#debugging)
- [Performance](#performance)
- [Security](#security)

---

## Quick Start

### Prerequisites

**Required**:
- **Rust**: 1.85+ (stable channel)
- **Node.js**: 20+ with npm
- **Git**: Latest stable version

**Platform-Specific**:
- **macOS**: Xcode Command Line Tools
- **Windows**: Visual Studio 2022 Build Tools
- **Linux**: build-essential, libwebkit2gtk-4.1-dev, libssl-dev

### Installation

```bash
# Clone repository
git clone https://github.com/saorsalabs/communitas.git
cd communitas

# Install dependencies
npm install

# Build frontend
npm run build

# Start development
npm run tauri dev
```

### First Build

```bash
# Format and check code
cargo fmt --all
cargo clippy --all-features -- -D warnings

# Run tests
cargo test --all
npm test

# Build release
npm run tauri build
```

---

## Development Environment

### Recommended IDE Setup

**Visual Studio Code** (Recommended):
- Extensions:
  - rust-analyzer (Rust language server)
  - Tauri (Tauri development)
  - ESLint (JavaScript/TypeScript linting)
  - Prettier (Code formatting)
  - Error Lens (Inline error display)

**RustRover** (Alternative):
- Built-in Rust support
- Excellent debugging
- Tauri integration

### Configuration

**settings.json** (VS Code):
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-features"],
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

**Cargo.toml** (workspace):
```toml
[workspace]
members = [
  "communitas-core",
  "communitas-desktop",
  "communitas-tui",
  "communitas-bridge",
  "communitas-headless",
  "bootstrap-node",
]
```

---

## Project Structure

```
communitas/
├── communitas-core/          # Core Rust library
│   ├── src/
│   │   ├── auth_service.rs   # Authentication
│   │   ├── encrypted_storage/ # Vault management
│   │   ├── gossip/           # P2P networking
│   │   └── member_manager.rs # CRDT member management
│   └── tests/                # Integration tests
│
├── communitas-desktop/       # Tauri desktop app
│   ├── src/
│   │   ├── main.rs           # App entry point
│   │   ├── commands/         # Tauri commands
│   │   ├── member_manager.rs # Member management bridge
│   │   └── state.rs          # App state
│   └── tests/                # Integration tests
│
├── communitas-tui/           # Terminal UI
│   └── src/
│       └── main.rs           # TUI entry point
│
├── communitas-bridge/        # HTTP/REST bridge
│   └── src/
│       └── main.rs           # Bridge server
│
├── communitas-headless/      # Headless node
│   └── src/
│       └── main.rs           # Node server
│
├── src/                      # Frontend (React + TypeScript)
│   ├── components/           # React components
│   │   ├── auth/             # Authentication UI
│   │   ├── chat/             # Chat interface
│   │   └── prototype/        # Experimental UI
│   ├── contexts/             # React contexts
│   ├── services/             # Service modules
│   │   ├── api/              # Backend communication
│   │   ├── network/          # P2P networking
│   │   └── storage/          # Offline storage
│   └── types/                # TypeScript types
│
├── docs/                     # Documentation
│   ├── guides/               # User guides
│   ├── architecture/         # Architecture docs
│   ├── api/                  # API reference
│   ├── development/          # Development docs
│   └── operations/           # Operations docs
│
└── tests/                    # E2E tests
```

---

## Development Workflow

### Git Workflow

**Branching Strategy**:
```bash
main                          # Stable release branch
  └── develop                 # Integration branch
      ├── feature/xyz         # Feature branches
      ├── fix/abc             # Bug fix branches
      └── docs/update         # Documentation branches
```

**Creating Features**:
```bash
# Create feature branch from develop
git checkout develop
git pull origin develop
git checkout -b feature/new-feature

# Make changes
# ... edit files ...

# Format and check
cargo fmt --all
cargo clippy --all-features -- -D warnings
npm run typecheck

# Test
cargo test --all
npm test

# Commit
git add -A
git commit -m "feat: add new feature

- Implement core functionality
- Add comprehensive tests
- Update documentation"

# Push and create PR
git push origin feature/new-feature
gh pr create --base develop --title "feat: Add new feature"
```

**Commit Message Format**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Tests
- `chore`: Maintenance

**Example**:
```
feat(auth): add passkey authentication

- Implement WebAuthn registration
- Add Touch ID support for macOS
- Update authentication flow UI
- Add comprehensive tests

Closes #123
```

---

## Testing Strategy

### Test Organization

**Unit Tests** (Fast, isolated):
```rust
// In-module tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        assert_eq!(function(input), expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await.unwrap();
        assert!(result.is_valid());
    }
}
```

**Integration Tests** (Full system):
```rust
// tests/integration_auth.rs
#[tokio::test]
async fn test_complete_auth_flow() {
    let service = AuthService::new(test_storage()).await.unwrap();

    // Create vault
    let vault_id = service.create_vault(
        "test-identity",
        "password",
        "Test User"
    ).await.unwrap();

    // Login
    let session = service.login(
        "test-identity",
        "password",
        Some("Test Device")
    ).await.unwrap();

    assert_eq!(session.four_words, "test-identity");
}
```

**Frontend Tests** (React components):
```typescript
// src/components/auth/LoginForm.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LoginForm } from './LoginForm';

describe('LoginForm', () => {
  it('should handle successful login', async () => {
    render(<LoginForm />);

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
      expect(screen.getByText(/welcome/i)).toBeInTheDocument();
    });
  });
});
```

### Running Tests

```bash
# Backend tests
cargo test                           # All tests
cargo test --lib                     # Library tests only
cargo test integration_              # Integration tests
cargo test -- --nocapture           # Show output
RUST_LOG=debug cargo test          # With logging

# Frontend tests
npm test                             # All tests
npm run test:ui                     # Interactive UI
npm run test:coverage               # Coverage report

# E2E tests
npm run test:e2e                    # Full system tests
```

### Test Coverage

```bash
# Rust coverage (using tarpaulin)
cargo tarpaulin --all-features --workspace --timeout 300 --out Html

# TypeScript coverage
npm run test:coverage
```

**Coverage Requirements**:
- Critical paths: 90%+ coverage
- General code: 80%+ coverage
- UI components: 70%+ coverage

---

## Code Quality

### Rust Standards

**Forbidden Patterns** (Production Code):
```rust
// ❌ NEVER in production
value.unwrap()
value.expect("message")
panic!("error")
todo!()
unimplemented!()

// ✅ ALWAYS use proper error handling
let value = option.ok_or(Error::Missing)?;
let result = operation().map_err(Error::from)?;
```

**Allowed in Tests**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let value = option.unwrap(); // OK in tests
        assert_eq!(result.expect("test failed"), expected);
    }
}
```

**Clippy Configuration**:
```bash
# Enforced lints
cargo clippy --all-features -- \
  -D clippy::panic \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D warnings

# Not required
# -D clippy::pedantic (too strict for practical development)
```

**Code Formatting**:
```bash
# Format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

### TypeScript Standards

**Type Safety**:
```typescript
// ❌ NEVER use any
function process(data: any) { }

// ✅ ALWAYS use proper types
interface Data {
  id: string;
  name: string;
}
function process(data: Data) { }

// ❌ NEVER suppress TypeScript
// @ts-ignore
const value = dangerousOperation();

// ✅ ALWAYS fix the root cause
const value = safeOperation() as ExpectedType;
```

**Linting**:
```bash
# Run ESLint
npm run lint

# Fix auto-fixable issues
npm run lint:fix
```

### Documentation Standards

**Rust Documentation**:
```rust
/// Creates a new authentication service.
///
/// # Arguments
///
/// * `storage_manager` - Encrypted storage manager for vault operations
///
/// # Examples
///
/// ```rust
/// let storage = EncryptedStorageManager::new(config).await?;
/// let auth_service = AuthService::new(storage);
/// ```
///
/// # Errors
///
/// Returns error if storage initialization fails.
pub fn new(storage_manager: EncryptedStorageManager) -> Self {
    // Implementation
}
```

**TypeScript Documentation**:
```typescript
/**
 * Authenticates user with four-word address and password.
 *
 * @param fourWords - Four-word identity (e.g., "ocean-forest-moon-star")
 * @param password - User password
 * @returns Session information with user details
 * @throws {AuthError} If credentials are invalid
 *
 * @example
 * ```typescript
 * const session = await login('ocean-forest-moon-star', 'password123');
 * console.log(`Logged in as ${session.displayName}`);
 * ```
 */
async function login(fourWords: string, password: string): Promise<SessionInfo> {
  // Implementation
}
```

---

## Common Tasks

### Adding a Tauri Command

**1. Define command** (communitas-desktop/src/commands/):
```rust
use tauri::State;
use crate::AppState;

#[tauri::command]
pub async fn my_command(
    state: State<'_, AppState>,
    param: String,
) -> Result<String, String> {
    // Implementation
    Ok(result)
}
```

**2. Register command** (communitas-desktop/src/main.rs):
```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            my_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**3. Call from frontend**:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<string>('my_command', {
  param: 'value'
});
```

### Adding a React Component

**1. Create component**:
```typescript
// src/components/MyComponent.tsx
import { FC } from 'react';

interface MyComponentProps {
  title: string;
  onAction: () => void;
}

export const MyComponent: FC<MyComponentProps> = ({ title, onAction }) => {
  return (
    <div>
      <h2>{title}</h2>
      <button onClick={onAction}>Action</button>
    </div>
  );
};
```

**2. Add tests**:
```typescript
// src/components/MyComponent.test.tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MyComponent } from './MyComponent';

describe('MyComponent', () => {
  it('should call onAction when button clicked', async () => {
    const onAction = vi.fn();
    render(<MyComponent title="Test" onAction={onAction} />);

    await userEvent.click(screen.getByRole('button'));

    expect(onAction).toHaveBeenCalled();
  });
});
```

### Adding a Service Module

**1. Define service**:
```typescript
// src/services/MyService.ts
class MyService {
  private state: Map<string, any> = new Map();

  async performOperation(key: string, value: any): Promise<void> {
    // Implementation
    this.state.set(key, value);
  }

  async getState(key: string): Promise<any> {
    return this.state.get(key);
  }
}

export const myService = new MyService();
```

**2. Use in components**:
```typescript
import { myService } from '@/services/MyService';

function MyComponent() {
  useEffect(() => {
    myService.performOperation('key', 'value');
  }, []);
}
```

---

## Debugging

### Rust Debugging

**Logging**:
```rust
use tracing::{debug, info, warn, error};

#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    info!("Command called with param: {}", param);

    match operation() {
        Ok(result) => {
            debug!("Operation succeeded: {:?}", result);
            Ok(result)
        }
        Err(e) => {
            error!("Operation failed: {:?}", e);
            Err(e.to_string())
        }
    }
}
```

**Running with Logs**:
```bash
# Set log level
RUST_LOG=debug npm run tauri dev
RUST_LOG=communitas_core=trace npm run tauri dev

# Write logs to file
RUST_LOG=debug npm run tauri dev 2>&1 | tee debug.log
```

**VS Code Debugging** (launch.json):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tauri",
      "cargo": {
        "args": ["build", "--manifest-path=communitas-desktop/Cargo.toml"]
      },
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

### Frontend Debugging

**Browser DevTools**:
```typescript
// Console debugging
console.log('Value:', value);
console.table(arrayOfObjects);
console.time('operation');
// ... operation ...
console.timeEnd('operation');

// React DevTools
// Install extension and inspect component tree
```

**Chrome DevTools MCP**:
```bash
# Start bridge server
cargo run -p communitas-bridge

# Launch Chrome DevTools MCP
npx chrome-devtools-mcp@latest
```

**Network Debugging**:
```typescript
// Monitor network state
import { networkService } from '@/services/network/NetworkConnectionService';

networkService.subscribe((state) => {
  console.log('Network status:', state.status);
  console.log('Peer count:', state.peerCount);
  console.log('Last error:', state.lastError);
});
```

### Common Issues

**Issue: Tauri command not found**
```
Error: Command not found: my_command
```
**Solution**: Ensure command is registered in `generate_handler![]` in main.rs

---

**Issue: CORS errors in bridge mode**
```
Access to fetch blocked by CORS policy
```
**Solution**: Bridge server enables CORS by default. Check configuration in bridge-config.toml

---

**Issue: TypeScript errors**
```
Property 'x' does not exist on type 'Y'
```
**Solution**: Run `npm run typecheck` to see all errors. Fix type definitions in `src/types/`

---

## Performance

### Profiling

**Rust Profiling**:
```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin communitas-desktop

# Criterion benchmarks
cargo bench
```

**Frontend Profiling**:
```typescript
// React Profiler
import { Profiler } from 'react';

function onRenderCallback(
  id: string,
  phase: "mount" | "update",
  actualDuration: number
) {
  console.log(`${id} ${phase} took ${actualDuration}ms`);
}

<Profiler id="MyComponent" onRender={onRenderCallback}>
  <MyComponent />
</Profiler>
```

### Optimization Guidelines

**Backend**:
- Use async/await for I/O operations
- Minimize allocations in hot paths
- Use `Arc` for shared state
- Profile before optimizing
- Cache expensive computations

**Frontend**:
- Use `React.memo()` for expensive components
- Implement virtual scrolling for long lists
- Lazy load components with `React.lazy()`
- Optimize bundle size with code splitting
- Use Web Workers for heavy computations

---

## Security

### Secure Coding Practices

**Input Validation**:
```rust
// Validate all inputs
fn validate_four_words(words: &str) -> Result<(), Error> {
    if !is_valid_format(words) {
        return Err(Error::InvalidFormat);
    }
    if !is_in_dictionary(words) {
        return Err(Error::InvalidWords);
    }
    Ok(())
}
```

**Secure Storage**:
```rust
// Never log sensitive data
debug!("Login attempt for user: [REDACTED]");

// Use secure storage for credentials
storage_manager.store_password_in_keyring(four_words, password).await?;
```

**Cryptography**:
```rust
// Use post-quantum cryptography
use saorsa_pqc::mldsa::MlDsa65;
use saorsa_pqc::mlkem::MlKem768;

// Never roll your own crypto
// Always use audited libraries
```

### Security Checklist

- [ ] All inputs validated
- [ ] All outputs sanitized
- [ ] No sensitive data in logs
- [ ] Credentials stored securely
- [ ] Dependencies scanned (cargo audit)
- [ ] HTTPS for all external connections
- [ ] Rate limiting on public endpoints
- [ ] Authentication required for sensitive operations

---

## See Also

- [Coding Standards](coding-standards.md) - Detailed code quality guidelines
- [Contributing](contributing.md) - How to contribute to Communitas
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
- [API Reference](../api/README.md) - Complete API documentation
- [Architecture](../architecture/README.md) - System architecture overview

---

**Development Guide**: Build Communitas with confidence. 🛠️💻
