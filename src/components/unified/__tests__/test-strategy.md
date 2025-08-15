# Component Test Strategy - Unified UI Components

## Test Categories

### 1. Unit Tests
- **Props validation**: Verify all props work correctly
- **State management**: Test internal state changes
- **Event handlers**: Verify callbacks are triggered
- **Conditional rendering**: Test different render paths
- **Edge cases**: Handle null, undefined, empty data

### 2. Visual Regression Tests
- **Snapshot testing**: Capture component appearance
- **Responsive breakpoints**: Test at different screen sizes
- **Theme variations**: Light/dark mode rendering
- **Animation states**: Capture hover, active, focus states
- **Cross-browser screenshots**: Chrome, Firefox, Safari

### 3. Accessibility Tests
- **ARIA attributes**: Verify proper labels and roles
- **Keyboard navigation**: Tab order and focus management
- **Screen reader**: Test with NVDA/JAWS announcements
- **Color contrast**: Verify WCAG 2.1 AA compliance
- **Motion preferences**: Test reduced motion support

### 4. Performance Tests
- **Render performance**: Measure render time
- **Re-render optimization**: Verify memo/callback usage
- **Memory leaks**: Check for cleanup in effects
- **Bundle size**: Monitor component size
- **Animation performance**: 60fps verification

### 5. Integration Tests
- **Component composition**: Test nested components
- **Context providers**: Verify theme/auth integration
- **Router integration**: Test navigation components
- **State management**: Redux/Context integration
- **API interactions**: Mock service calls

## Test Coverage Requirements

### UnifiedCard Component
```typescript
describe('UnifiedCard', () => {
  // Props tests
  test('renders with default glass variant')
  test('applies solid variant styles')
  test('applies elevated variant styles')
  test('applies floating variant styles')
  test('applies fourWordTheme gradient')
  test('handles interactive hover state')
  test('applies custom blur amount')
  test('applies custom opacity')
  test('triggers onClick handler')
  test('triggers onHover handler')
  
  // Visual tests
  test('matches snapshot - light theme')
  test('matches snapshot - dark theme')
  test('matches snapshot - with gradient')
  test('matches snapshot - hover state')
  
  // Accessibility tests
  test('has proper ARIA attributes')
  test('is keyboard navigable when interactive')
  test('announces content to screen readers')
  test('meets color contrast requirements')
  
  // Performance tests
  test('renders within 16ms')
  test('memoizes expensive calculations')
  test('cleans up on unmount')
})
```

### FourWordAvatar Component
```typescript
describe('FourWordAvatar', () => {
  // Props tests
  test('generates gradient from fourWords')
  test('renders at different sizes')
  test('shows/hides word display')
  test('displays tooltip on hover')
  test('shows presence indicator')
  test('applies type-specific borders')
  test('animates when enabled')
  
  // Four-word processing
  test('handles valid four-word format')
  test('handles invalid word formats')
  test('generates consistent gradients')
  test('extracts correct initials')
  
  // Visual tests
  test('matches snapshot - all sizes')
  test('matches snapshot - with presence')
  test('matches snapshot - organization type')
  test('matches snapshot - project type')
  
  // Accessibility tests
  test('provides alt text for screen readers')
  test('tooltip is keyboard accessible')
  test('presence status is announced')
})
```

### MessagingBubble Component
```typescript
describe('MessagingBubble', () => {
  // Message rendering
  test('renders text messages')
  test('renders image attachments')
  test('renders file attachments')
  test('renders link previews')
  test('renders emoji reactions')
  
  // Interaction tests
  test('triggers onReact callback')
  test('triggers onReply callback')
  test('triggers onEdit for own messages')
  test('triggers onDelete for own messages')
  test('triggers onThread callback')
  
  // Visual tests
  test('matches snapshot - own message')
  test('matches snapshot - other message')
  test('matches snapshot - with reactions')
  test('matches snapshot - threaded')
  
  // Accessibility tests
  test('announces message sender')
  test('announces message time')
  test('reaction buttons are accessible')
  test('action menu is keyboard navigable')
})
```

### NavigationRail Component
```typescript
describe('NavigationRail', () => {
  // Navigation tests
  test('renders navigation items')
  test('highlights active item')
  test('triggers onNavigate callback')
  test('collapses and expands')
  test('shows/hides labels')
  test('adapts to position prop')
  
  // Context tests
  test('renders personal navigation')
  test('renders organization navigation')
  test('renders project navigation')
  test('applies fourWordIdentity theme')
  
  // Responsive tests
  test('switches to bottom on mobile')
  test('auto-collapses on tablet')
  test('stays expanded on desktop')
  
  // Accessibility tests
  test('navigation is keyboard accessible')
  test('announces active page')
  test('collapse button is accessible')
  test('items have proper ARIA labels')
})
```

### FileCard Component
```typescript
describe('FileCard', () => {
  // Display tests
  test('renders in grid view')
  test('renders in list view')
  test('renders in compact view')
  test('shows file preview')
  test('shows file metadata')
  test('indicates shared status')
  
  // Interaction tests
  test('handles selection')
  test('triggers onOpen')
  test('triggers onShare')
  test('triggers onDownload')
  test('triggers onDelete')
  test('shows actions on hover')
  
  // File type tests
  test('renders image files')
  test('renders document files')
  test('renders video files')
  test('renders folder type')
  test('handles unknown types')
  
  // Performance tests
  test('lazy loads thumbnails')
  test('virtualizes long lists')
  test('debounces hover events')
})
```

## Test Utilities

### Mock Data Generators
```typescript
// Generate four-word addresses
export const generateFourWords = (seed?: number): string => {
  const words = ['ocean', 'forest', 'moon', 'star', 'mountain', 'river']
  // Return deterministic or random four-word combination
}

// Generate mock messages
export const generateMessage = (overrides?: Partial<Message>): Message => {
  return {
    id: faker.datatype.uuid(),
    content: faker.lorem.sentence(),
    sender: generateFourWords(),
    timestamp: faker.date.recent(),
    ...overrides
  }
}

// Generate mock files
export const generateFile = (overrides?: Partial<FileInfo>): FileInfo => {
  return {
    id: faker.datatype.uuid(),
    name: faker.system.fileName(),
    size: faker.datatype.number({ min: 1024, max: 10485760 }),
    type: faker.system.mimeType(),
    ...overrides
  }
}
```

### Custom Test Matchers
```typescript
// Check glassmorphism styles
expect.extend({
  toHaveGlassEffect(received) {
    const styles = window.getComputedStyle(received)
    const hasBlur = styles.backdropFilter.includes('blur')
    const hasTransparency = parseFloat(styles.opacity) < 1
    return {
      pass: hasBlur && hasTransparency,
      message: () => `Expected element to have glass effect`
    }
  }
})

// Check four-word gradient
expect.extend({
  toHaveFourWordGradient(received, fourWords) {
    const styles = window.getComputedStyle(received)
    const expectedGradient = generateGradient(fourWords)
    return {
      pass: styles.background.includes(expectedGradient),
      message: () => `Expected element to have gradient for ${fourWords}`
    }
  }
})
```

### Test Helpers
```typescript
// Render with theme provider
export const renderWithTheme = (component: ReactElement, theme = 'light') => {
  return render(
    <ThemeProvider theme={theme === 'light' ? lightTheme : darkTheme}>
      {component}
    </ThemeProvider>
  )
}

// Render with all providers
export const renderWithProviders = (component: ReactElement) => {
  return render(
    <ThemeProvider>
      <AuthProvider>
        <RouterProvider>
          {component}
        </RouterProvider>
      </AuthProvider>
    </ThemeProvider>
  )
}

// Wait for animations
export const waitForAnimation = () => {
  return new Promise(resolve => setTimeout(resolve, 300))
}
```

## Test Execution Plan

### Phase 1: Component Tests (Week 1)
1. Set up testing infrastructure
2. Write unit tests for all components
3. Achieve 90% code coverage
4. Set up snapshot testing

### Phase 2: Visual Tests (Week 2)
1. Configure visual regression tools
2. Create baseline snapshots
3. Test responsive breakpoints
4. Cross-browser testing setup

### Phase 3: Integration Tests (Week 3)
1. Test component composition
2. Test with real data flows
3. Test error boundaries
4. Performance benchmarking

### Phase 4: Accessibility Tests (Week 4)
1. ARIA attribute validation
2. Keyboard navigation testing
3. Screen reader testing
4. Color contrast validation

## CI/CD Integration

```yaml
# .github/workflows/component-tests.yml
name: Component Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
      - run: npm ci
      - run: npm run test:unit
      - run: npm run test:visual
      - run: npm run test:a11y
      - run: npm run test:performance
      - uses: codecov/codecov-action@v2
```

## Success Metrics

- **Code Coverage**: > 90% for all components
- **Visual Regression**: 0 unexpected changes
- **Accessibility**: 100% WCAG 2.1 AA compliance
- **Performance**: All components render < 16ms
- **Bundle Size**: Total < 100KB gzipped
- **Test Execution**: < 5 minutes for full suite