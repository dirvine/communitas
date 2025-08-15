# Unified Component Specification - Communitas Platform

## Core Components Specification

### 1. UnifiedCard Component

**Purpose**: Base glassmorphic card component used throughout the platform

**Props Interface**:
```typescript
interface UnifiedCardProps {
  variant?: 'glass' | 'solid' | 'elevated' | 'floating'
  fourWordTheme?: string
  interactive?: boolean
  blur?: number // 10-30px
  opacity?: number // 0.7-0.95
  gradient?: boolean
  children: React.ReactNode
  className?: string
  sx?: SxProps
  onClick?: () => void
  onHover?: () => void
}
```

**Visual Specs**:
- Background: `rgba(255, 255, 255, 0.85)` (light) / `rgba(26, 26, 26, 0.85)` (dark)
- Backdrop filter: `blur(20px)` default
- Border: `1px solid rgba(255, 255, 255, 0.2)`
- Box shadow: `0 8px 32px 0 rgba(31, 38, 135, 0.37)`
- Border radius: `16px`
- Transition: `all 0.3s cubic-bezier(0.4, 0, 0.2, 1)`

**Hover State**:
- Transform: `translateY(-4px)`
- Backdrop filter: `blur(25px)`
- Box shadow: `0 12px 40px rgba(31, 38, 135, 0.5)`

### 2. FourWordAvatar Component

**Purpose**: Visual representation of four-word identities

**Props Interface**:
```typescript
interface FourWordAvatarProps {
  fourWords: string // "ocean-forest-moon-star"
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' // 24, 32, 48, 64, 96px
  showWords?: boolean
  showTooltip?: boolean
  presence?: 'online' | 'away' | 'busy' | 'offline'
  type?: 'personal' | 'organization' | 'project'
  gradient?: 'radial' | 'linear' | 'conic'
  animated?: boolean
  onClick?: () => void
}
```

**Visual Specs**:
- Gradient generation: Based on hash of four-words
- Border: 
  - Personal: No border
  - Organization: `2px solid #FFD700` (gold)
  - Project: `2px solid #C0C0C0` (silver)
- Presence indicator: 8px dot in corner
- Animation: Subtle pulse for online status

### 3. MessagingBubble Component

**Purpose**: WhatsApp-style message display with glassmorphism

**Props Interface**:
```typescript
interface MessagingBubbleProps {
  message: Message
  isOwn: boolean
  isThreaded?: boolean
  showAvatar?: boolean
  showTimestamp?: boolean
  showReactions?: boolean
  onReact?: (emoji: string) => void
  onReply?: () => void
  onEdit?: () => void
  onDelete?: () => void
  onThread?: () => void
}
```

**Visual Specs**:
- Max width: 70% of container
- Padding: 12px 16px
- Own messages: Four-word gradient background
- Others: Glass effect with white tint
- Border radius: 18px (with tail on first message)
- Font size: 14px (mobile) / 15px (desktop)
- Line height: 1.5

### 4. NavigationRail Component

**Purpose**: Adaptive navigation that switches between rail/drawer/bottom based on screen size

**Props Interface**:
```typescript
interface NavigationRailProps {
  mode: 'personal' | 'organization' | 'project'
  fourWordIdentity: string
  items: NavigationItem[]
  collapsed?: boolean
  activeItem?: string
  onNavigate: (path: string) => void
  onCollapse?: () => void
  showLabels?: boolean
  position?: 'left' | 'right' | 'bottom'
}
```

**Visual Specs**:
- Width: 72px (collapsed) / 280px (expanded)
- Background: Glass effect
- Item height: 56px
- Active indicator: Four-word gradient
- Icon size: 24px
- Transition: Width 300ms ease

### 5. FileCard Component

**Purpose**: Dropbox-style file representation with preview

**Props Interface**:
```typescript
interface FileCardProps {
  file: FileInfo
  view?: 'grid' | 'list' | 'compact'
  selected?: boolean
  showPreview?: boolean
  showActions?: boolean
  showSharing?: boolean
  onSelect?: () => void
  onOpen?: () => void
  onShare?: () => void
  onDownload?: () => void
  onDelete?: () => void
}
```

**Visual Specs**:
- Grid view: 200x240px cards
- List view: Full width, 72px height
- Thumbnail: 160x120px (grid) / 48x48px (list)
- Glass effect with hover lift
- Selection: 2px gradient border
- Actions: Fade in on hover

### 6. OrganizationCard Component

**Purpose**: Slack-style organization/workspace representation

**Props Interface**:
```typescript
interface OrganizationCardProps {
  organization: Organization
  memberCount: number
  projectCount: number
  unreadCount?: number
  lastActivity?: Date
  role?: 'owner' | 'admin' | 'member' | 'guest'
  onSelect?: () => void
  onManage?: () => void
  onLeave?: () => void
}
```

**Visual Specs**:
- Size: 320x180px
- Glass card with organization gradient
- Logo/Avatar: 64x64px
- Stats bar at bottom
- Role badge in corner
- Activity indicator

### 7. QuickActionFab Component

**Purpose**: Floating action button with expandable quick actions

**Props Interface**:
```typescript
interface QuickActionFabProps {
  actions: QuickAction[]
  fourWordTheme?: string
  position?: { bottom: number; right: number }
  expandDirection?: 'up' | 'left' | 'radial'
  showLabels?: boolean
  autoHide?: boolean
}

interface QuickAction {
  id: string
  icon: React.ComponentType
  label: string
  onClick: () => void
  color?: string
  disabled?: boolean
}
```

**Visual Specs**:
- Main button: 56px diameter
- Mini buttons: 40px diameter
- Gradient background from four-words
- Expand animation: 300ms spring
- Backdrop blur when expanded
- Labels appear after 200ms

### 8. StatusBar Component

**Purpose**: Global status bar showing connection, sync, and activity

**Props Interface**:
```typescript
interface StatusBarProps {
  connectionStatus: 'connected' | 'connecting' | 'offline'
  syncStatus: 'synced' | 'syncing' | 'error'
  peerCount: number
  fourWordIdentity: string
  notifications?: number
  position?: 'top' | 'bottom'
  compact?: boolean
}
```

**Visual Specs**:
- Height: 32px (compact) / 48px (normal)
- Glass effect with subtle gradient
- Status dots: 8px with pulse animation
- Peer count badge
- Four-word identity chip
- Auto-hide after 5s (optional)

## Shared Design Tokens

```typescript
export const UnifiedDesignTokens = {
  // Glassmorphism
  glass: {
    light: {
      background: 'rgba(255, 255, 255, 0.85)',
      border: 'rgba(255, 255, 255, 0.2)',
      shadow: 'rgba(31, 38, 135, 0.37)'
    },
    dark: {
      background: 'rgba(26, 26, 26, 0.85)',
      border: 'rgba(255, 255, 255, 0.1)',
      shadow: 'rgba(0, 0, 0, 0.5)'
    }
  },
  
  // Blur levels
  blur: {
    none: 0,
    sm: 10,
    md: 20,
    lg: 30,
    xl: 40
  },
  
  // Border radius
  radius: {
    xs: 4,
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    round: 9999
  },
  
  // Transitions
  transition: {
    fast: '150ms ease',
    normal: '300ms ease',
    slow: '500ms ease',
    spring: 'cubic-bezier(0.4, 0, 0.2, 1)'
  },
  
  // Z-index layers
  zIndex: {
    base: 0,
    card: 10,
    navigation: 100,
    modal: 1000,
    tooltip: 1100,
    notification: 1200
  }
}
```

## Animation Specifications

```typescript
export const UnifiedAnimations = {
  // Card interactions
  cardHover: {
    transform: 'translateY(-4px)',
    backdropFilter: 'blur(25px)',
    transition: '300ms cubic-bezier(0.4, 0, 0.2, 1)'
  },
  
  // Button press
  buttonPress: {
    transform: 'scale(0.98)',
    transition: '100ms ease'
  },
  
  // Fade in
  fadeIn: {
    from: { opacity: 0, transform: 'translateY(10px)' },
    to: { opacity: 1, transform: 'translateY(0)' },
    duration: '300ms'
  },
  
  // Slide in
  slideIn: {
    from: { transform: 'translateX(-100%)' },
    to: { transform: 'translateX(0)' },
    duration: '300ms'
  },
  
  // Pulse
  pulse: {
    '0%, 100%': { opacity: 1 },
    '50%': { opacity: 0.5 },
    duration: '2s',
    iteration: 'infinite'
  }
}
```

## Accessibility Requirements

1. **WCAG 2.1 AA Compliance**
   - Color contrast: 4.5:1 minimum
   - Focus indicators: 2px solid outline
   - Keyboard navigation: Full support
   - Screen reader: Proper ARIA labels

2. **Motion Preferences**
   - Respect `prefers-reduced-motion`
   - Provide motion toggle in settings
   - Fallback to instant transitions

3. **Touch Targets**
   - Minimum 44x44px on mobile
   - 8px spacing between targets
   - Hover states for desktop only

## Performance Targets

- First paint: < 100ms
- Component render: < 16ms (60fps)
- Animation jank: < 1%
- Memory usage: < 50MB for 1000 components
- Bundle size: < 100KB for all components

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Mobile Safari 14+
- Chrome Android 90+

## Testing Requirements

Each component must have:
1. Unit tests: > 90% coverage
2. Visual regression tests
3. Accessibility tests
4. Performance benchmarks
5. Cross-browser tests
6. Mobile responsiveness tests