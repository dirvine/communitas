# Development Patterns & Practices

## Project Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (18+ required)
# Via package manager or https://nodejs.org/

# Install Tauri CLI
cargo install tauri-cli
```

### Initial Setup
```bash
git clone <repository>
cd communitas
npm install
npm run tauri dev
```

## Development Workflow

### Daily Development
```bash
# Start development environment
npm run tauri dev

# In separate terminals:
# Frontend only (for UI work)
npm run dev

# Backend tests
cd src-tauri && cargo test

# Linting
npm run lint
cd src-tauri && cargo clippy
```

### Code Organization

#### Frontend Structure
```
src/
├── components/           # Reusable React components
│   ├── common/          # Shared components
│   └── pages/           # Page-specific components
├── hooks/               # Custom React hooks
├── utils/               # Utility functions
├── types/               # TypeScript type definitions
├── styles/              # CSS/SCSS files
└── App.tsx              # Main application component
```

#### Backend Structure
```
src-tauri/src/
├── commands/            # Tauri command handlers
├── models/              # Data models
├── services/            # Business logic
├── utils/               # Utility functions
├── lib.rs               # Command exports
└── main.rs              # Application entry point
```

## Coding Standards

### TypeScript/React
```typescript
// Component naming: PascalCase
export const UserProfile: React.FC<UserProfileProps> = ({ user }) => {
  // Hook usage at component top
  const [state, setState] = useState<StateType>(initialState);
  
  // Event handlers: handle prefix
  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    // Implementation
  };
  
  return (
    <div className="user-profile">
      {/* Implementation */}
    </div>
  );
};

// Props interface
interface UserProfileProps {
  user: User;
  onUpdate?: (user: User) => void;
}
```

### Rust/Tauri
```rust
// Command handlers
#[tauri::command]
async fn process_data(data: String) -> Result<ProcessedData, String> {
    // Use proper error handling
    let processed = process_input(&data)
        .map_err(|e| format!("Processing failed: {}", e))?;
    
    Ok(processed)
}

// Error handling - avoid unwrap/expect
fn safe_operation() -> Result<String, AppError> {
    let result = risky_operation()?;
    Ok(result)
}

// Use descriptive names
fn calculate_user_score(activities: &[Activity]) -> u32 {
    activities.iter().map(|a| a.points).sum()
}
```

## State Management

### Frontend State
```typescript
// Local component state
const [localState, setLocalState] = useState<LocalState>(initial);

// Global state (Context API)
const UserContext = createContext<UserContextType | null>(null);

// Custom hooks for state logic
const useUserData = () => {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    loadUserData().then(setUser).finally(() => setLoading(false));
  }, []);
  
  return { user, loading, setUser };
};
```

### Backend State
```rust
// Use Rust ownership model
struct AppState {
    user_data: HashMap<String, User>,
    settings: AppSettings,
}

// Thread-safe state sharing
use std::sync::{Arc, Mutex};
type SharedState = Arc<Mutex<AppState>>;
```

## IPC Communication

### Frontend to Backend
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Type-safe command invocation
const saveUserData = async (userData: UserData): Promise<void> => {
  try {
    await invoke('save_user_data', { userData });
  } catch (error) {
    console.error('Failed to save user data:', error);
    throw new Error('Save operation failed');
  }
};
```

### Backend Command Handlers
```rust
#[tauri::command]
async fn save_user_data(user_data: UserData) -> Result<(), String> {
    // Validation
    if user_data.name.is_empty() {
        return Err("User name cannot be empty".to_string());
    }
    
    // Business logic
    save_to_storage(&user_data)
        .await
        .map_err(|e| format!("Storage error: {}", e))?;
    
    Ok(())
}
```

## Testing Strategies

### Frontend Testing
```typescript
// Component testing with React Testing Library
import { render, screen, fireEvent } from '@testing-library/react';

test('UserProfile displays user information', () => {
  const mockUser = { name: 'John Doe', email: 'john@example.com' };
  
  render(<UserProfile user={mockUser} />);
  
  expect(screen.getByText('John Doe')).toBeInTheDocument();
  expect(screen.getByText('john@example.com')).toBeInTheDocument();
});

// Hook testing
import { renderHook, act } from '@testing-library/react';

test('useUserData loads user data', async () => {
  const { result } = renderHook(() => useUserData());
  
  expect(result.current.loading).toBe(true);
  
  await act(async () => {
    // Wait for effect to complete
  });
  
  expect(result.current.loading).toBe(false);
});
```

### Backend Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_save_user_data() {
        let user_data = UserData {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let result = save_user_data(user_data).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validation() {
        let invalid_data = UserData {
            name: "".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Test validation logic
    }
}
```

## Error Handling

### Frontend Error Boundaries
```typescript
class ErrorBoundary extends React.Component {
  state = { hasError: false };
  
  static getDerivedStateFromError(error: Error) {
    return { hasError: true };
  }
  
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Application error:', error, errorInfo);
  }
  
  render() {
    if (this.state.hasError) {
      return <ErrorFallback />;
    }
    
    return this.props.children;
  }
}
```

### Backend Error Types
```rust
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}
```

## Performance Guidelines

### Frontend Optimization
- Use React.memo for expensive components
- Implement proper key props for lists
- Lazy load components and routes
- Debounce user inputs
- Optimize re-renders with useCallback/useMemo

### Backend Optimization
- Use async/await for I/O operations
- Implement proper error handling
- Cache frequently accessed data
- Use efficient data structures
- Profile performance-critical paths

## Build & Deployment

### Development Build
```bash
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
```

### Quality Checks
```bash
# Frontend
npm run lint
npm run typecheck
npm test

# Backend
cd src-tauri
cargo clippy
cargo check
cargo fmt --check
```

This development approach ensures maintainable, testable, and performant code across both frontend and backend components.