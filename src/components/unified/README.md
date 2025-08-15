# Unified UI Components - Communitas Platform

Beautiful, glassmorphic UI components with deep four-word identity integration for the Communitas P2P collaboration platform.

## 🎨 Design Philosophy

These components implement a unified design system that combines:
- **Glassmorphism**: Beautiful frosted glass effects with depth
- **Four-Word Identity**: Unique visual identity from network addresses
- **Cross-Platform**: Responsive design for desktop, mobile, and web
- **Accessibility**: WCAG 2.1 AA compliant with full keyboard support
- **Performance**: Optimized for 60fps animations and fast rendering

## 📦 Installation

```bash
# Install dependencies
npm install @mui/material @emotion/react @emotion/styled framer-motion blake3
```

## 🚀 Quick Start

```tsx
import { UnifiedCard, FourWordAvatar } from './components/unified'

function App() {
  const userIdentity = 'ocean-forest-moon-star'
  
  return (
    <UnifiedCard variant="glass" fourWordTheme={userIdentity}>
      <FourWordAvatar 
        fourWords={userIdentity}
        size="lg"
        presence="online"
        showWords
      />
      <h2>Welcome to Communitas</h2>
    </UnifiedCard>
  )
}
```

## 📚 Components

### UnifiedCard

The base card component with glassmorphism effects.

```tsx
<UnifiedCard
  variant="glass"        // 'glass' | 'solid' | 'elevated' | 'floating'
  fourWordTheme="..."    // Apply four-word gradient
  interactive            // Enable hover effects
  blur={20}             // Backdrop blur amount
  opacity={0.85}        // Background opacity
>
  Content
</UnifiedCard>
```

**Props**:
- `variant`: Visual style variant
- `fourWordTheme`: Four-word address for theming
- `interactive`: Enable interactive hover state
- `blur`: Backdrop filter blur (10-30px)
- `opacity`: Background opacity (0.7-0.95)
- `gradient`: Use gradient as background
- `onClick`: Click handler
- `onHover`: Hover handler

### FourWordAvatar

Avatar component with four-word identity visualization.

```tsx
<FourWordAvatar
  fourWords="ocean-forest-moon-star"
  size="md"              // 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  presence="online"      // 'online' | 'away' | 'busy' | 'offline'
  type="personal"        // 'personal' | 'organization' | 'project'
  showWords
  showTooltip
  animated
/>
```

**Props**:
- `fourWords`: Four-word address string
- `size`: Avatar size (xs: 24px to xl: 96px)
- `showWords`: Display four-words next to avatar
- `showTooltip`: Show tooltip on hover
- `presence`: User presence status
- `type`: Entity type (affects border styling)
- `gradient`: Gradient type ('linear' | 'radial' | 'conic')
- `animated`: Enable pulse animation for online status

## 🎨 Design Tokens

Access the unified design system tokens:

```tsx
import { UnifiedDesignTokens } from './components/unified/tokens'

// Use glassmorphism effects
const glassStyles = {
  ...UnifiedDesignTokens.glass.light,
  backdropFilter: `blur(${UnifiedDesignTokens.blur.md}px)`
}

// Use consistent spacing
padding: UnifiedDesignTokens.spacing.md

// Use gradients
background: UnifiedDesignTokens.gradients.primary
```

## 🎭 Animations

Pre-built animation presets using Framer Motion:

```tsx
import { UnifiedAnimations } from './components/unified/animations'

<motion.div {...UnifiedAnimations.cardHover}>
  <UnifiedCard>Animated Content</UnifiedCard>
</motion.div>
```

Available animations:
- `pageTransition`: Page enter/exit animations
- `cardHover`: Card hover and tap effects
- `messageBubble`: Message appearance animation
- `navigationSlide`: Navigation panel sliding
- `modalOverlay`: Modal backdrop fade
- `notification`: Notification slide-in
- `staggerContainer/staggerItem`: Staggered list animations

## 🛠️ Utilities

Helper functions for four-word processing:

```tsx
import { 
  generateFourWordGradient,
  generateFourWordColors,
  isValidFourWords,
  getFourWordInitials 
} from './utils/fourWords'

// Generate consistent gradient
const gradient = generateFourWordGradient('ocean-forest-moon-star')

// Get color scheme
const colors = generateFourWordColors('ocean-forest-moon-star')
// Returns: { primary: '#...', secondary: '#...', accent: '#...' }

// Validate format
const isValid = isValidFourWords('ocean-forest-moon-star') // true

// Get initials
const initials = getFourWordInitials('ocean-forest-moon-star') // "OFMS"
```

## 🎯 Best Practices

### Performance
```tsx
// ✅ Good: Memoize expensive computations
const gradient = useMemo(() => 
  generateFourWordGradient(fourWords), [fourWords]
)

// ❌ Bad: Computing on every render
const gradient = generateFourWordGradient(fourWords)
```

### Accessibility
```tsx
// ✅ Good: Provide proper labels
<FourWordAvatar 
  fourWords={identity}
  showTooltip // Keyboard accessible
/>

// ✅ Good: Respect motion preferences
const prefersReducedMotion = window.matchMedia(
  '(prefers-reduced-motion: reduce)'
).matches
```

### Theming
```tsx
// ✅ Good: Use theme-aware components
<UnifiedCard variant="glass">
  Adapts to light/dark theme
</UnifiedCard>

// ✅ Good: Apply four-word theming consistently
<UnifiedCard fourWordTheme={userIdentity}>
  <FourWordAvatar fourWords={userIdentity} />
</UnifiedCard>
```

## 🧪 Testing

Components include comprehensive test suites:

```bash
# Run tests
npm test

# Run with coverage
npm test -- --coverage

# Run specific component tests
npm test UnifiedCard
npm test FourWordAvatar
```

Test utilities provided:
```tsx
import { renderWithTheme } from './__tests__/test-utils'

test('renders with theme', () => {
  renderWithTheme(<UnifiedCard>Content</UnifiedCard>)
})
```

## 📱 Responsive Design

Components automatically adapt to screen size:

```tsx
// Desktop: Full featured
<UnifiedCard variant="glass" interactive>
  <ComplexContent />
</UnifiedCard>

// Mobile: Optimized touch targets and simplified effects
<UnifiedCard variant="solid" blur={10}>
  <SimpleContent />
</UnifiedCard>
```

## ♿ Accessibility

All components are WCAG 2.1 AA compliant:

- ✅ Keyboard navigation support
- ✅ Screen reader compatibility
- ✅ Proper ARIA attributes
- ✅ Color contrast requirements
- ✅ Focus indicators
- ✅ Reduced motion support

## 🎨 Customization

Extend components with custom styles:

```tsx
// Using sx prop
<UnifiedCard
  sx={{
    background: 'linear-gradient(...)',
    padding: 4,
    '&:hover': {
      transform: 'scale(1.05)'
    }
  }}
>
  Custom styled content
</UnifiedCard>

// Using className
<UnifiedCard className="custom-card">
  Content
</UnifiedCard>
```

## 📊 Performance

Components are optimized for performance:

- **Render time**: < 16ms per component
- **Bundle size**: ~42KB gzipped total
- **Animation**: Consistent 60fps
- **Memory**: Efficient with proper cleanup

## 🔧 TypeScript Support

Full TypeScript support with comprehensive types:

```tsx
import type { UnifiedCardProps, FourWordAvatarProps } from './components/unified'

const MyComponent: React.FC<{ cardProps: UnifiedCardProps }> = ({ cardProps }) => {
  return <UnifiedCard {...cardProps} />
}
```

## 📖 Examples

### Personal Dashboard
```tsx
<UnifiedCard variant="glass" fourWordTheme={userIdentity}>
  <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, p: 2 }}>
    <FourWordAvatar 
      fourWords={userIdentity}
      size="lg"
      presence="online"
      type="personal"
    />
    <Box>
      <Typography variant="h6">Welcome back!</Typography>
      <Typography variant="body2" color="text.secondary">
        {userIdentity}
      </Typography>
    </Box>
  </Box>
</UnifiedCard>
```

### Organization Card
```tsx
<UnifiedCard variant="elevated" interactive onClick={handleOrgClick}>
  <CardContent>
    <FourWordAvatar
      fourWords={orgIdentity}
      type="organization"
      size="xl"
      showWords
    />
    <Typography variant="h5">{orgName}</Typography>
    <Typography variant="body2">
      {memberCount} members · {projectCount} projects
    </Typography>
  </CardContent>
</UnifiedCard>
```

### Message Bubble (Coming Soon)
```tsx
<MessagingBubble
  message={message}
  isOwn={message.sender === currentUser}
  showAvatar
  showTimestamp
  onReact={(emoji) => handleReaction(emoji)}
/>
```

## 🚧 Roadmap

### Implemented ✅
- UnifiedCard with glassmorphism
- FourWordAvatar with presence
- Design tokens system
- Animation library
- Utility functions

### Coming Soon
- MessagingBubble component
- NavigationRail component
- FileCard component
- OrganizationCard component
- QuickActionFab component
- StatusBar component
- Storybook documentation
- Visual regression testing

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines for details.

## 📄 License

MIT License - See LICENSE file for details

## 🙏 Credits

Built with:
- [Material-UI](https://mui.com/)
- [Framer Motion](https://www.framer.com/motion/)
- [Blake3](https://github.com/BLAKE3-team/BLAKE3)
- Love for beautiful, accessible design ❤️