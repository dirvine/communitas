# Experimental Mode Testing Guide

## Overview
The Communitas app now has an experimental mode that allows users to preview the new unified platform UI while keeping the stable legacy UI as the default.

## How to Enable Experimental Mode

### Method 1: UI Toggle
1. Launch the Communitas app
2. Look for the **beaker/science icon** in the top navigation bar
3. Click the toggle switch to enable experimental mode
4. The app will reload with the new unified UI

### Method 2: Developer Console
1. Open the browser developer console (F12)
2. Run the following commands:
```javascript
// Enable all Phase 1 features
window.enablePhase1()

// Or enable all features at once
window.enableAllFeatures()

// Check feature status
window.featureFlags.getDebugInfo()
```

### Method 3: Direct Feature Control
```javascript
// Enable specific features
window.featureFlags.enable('unified-design-system')
window.featureFlags.enable('context-aware-navigation')
window.featureFlags.enable('four-word-identity')
window.featureFlags.enable('unified-storage-ui')
```

## What's Different in Experimental Mode

### New Features
1. **Context-Aware Navigation**: Sidebar adapts based on Personal/Organization/Project context
2. **Unified Design System**: Modern glassmorphism UI with smooth animations
3. **Four-Word Identity**: Human-readable identities (e.g., "ocean-forest-moon-star")
4. **Unified Home Dashboard**: Beautiful dashboard with quick actions and activity stats

### Visual Changes
- Modern, clean interface inspired by WhatsApp, Slack, and Dropbox
- Glassmorphic effects with backdrop blur
- Smooth animations and transitions
- Color-coded avatars based on four-word identities
- Responsive card-based layouts

## Testing Checklist

### Phase 1 Features (Currently Available)
- [ ] **Experimental Mode Toggle**
  - [ ] Toggle switch appears in header
  - [ ] Enabling toggle reloads app with new UI
  - [ ] Disabling toggle returns to legacy UI
  - [ ] Preference persists across sessions

- [ ] **Context-Aware Navigation**
  - [ ] Personal mode shows user profile and personal items
  - [ ] Clicking organizations switches to organization context
  - [ ] Back button returns to previous context
  - [ ] Navigation items update based on context

- [ ] **Unified Home Page**
  - [ ] Welcome message displays user name
  - [ ] Four-word identity chip shows correctly
  - [ ] Quick action cards are clickable
  - [ ] Recent messages list displays
  - [ ] Organizations list shows with member counts
  - [ ] Activity stats display today's data

- [ ] **Design System**
  - [ ] Glassmorphic effects render correctly
  - [ ] Dark/light theme switching works
  - [ ] Animations are smooth
  - [ ] No visual glitches or artifacts

## Navigation Flow

### Personal Context
```
Home → Messages → Direct Messages
     → Calls → Recent/Scheduled
     → My Files
     → My Website
     → Organizations → [Switch to Org Context]
```

### Organization Context
```
Dashboard → Teams → Team Members
         → Projects → [Switch to Project Context]
         → Channels → #general, #announcements
         → Shared Files
         → Org Website
```

### Project Context
```
Overview → Members
        → Discussion
        → Resources
        → Documents
        → Milestones
        → Project Site
```

## Known Issues & Limitations

### Current Limitations (Phase 1)
- Messaging is view-only (no send functionality yet)
- Organizations and projects are using mock data
- File upload not yet implemented
- Website builder not available
- Voice/video calls not functional

### Coming in Future Phases
- **Phase 2**: Rich messaging with threading and reactions
- **Phase 3**: Full organization hierarchy and team management
- **Phase 4**: Website builder and DHT publishing

## Reporting Issues

If you encounter any issues:
1. Note the exact steps to reproduce
2. Check browser console for errors
3. Take a screenshot if visual issue
4. Report with:
   - Browser/platform
   - Experimental mode status
   - Feature flags enabled
   - Error messages

## Development Tips

### Reset All Settings
```javascript
// Clear all feature flags and settings
localStorage.clear()
window.location.reload()
```

### Debug Feature Flags
```javascript
// See all flags and their status
console.table(window.featureFlags.getDebugInfo())

// Check specific flag
window.featureFlags.isEnabled('unified-design-system')
```

### Gradual Rollout Testing
```javascript
// Test percentage-based rollout
window.featureFlags.setRolloutPercentage('rich-messaging', 50)

// Enable for specific users only
window.featureFlags.enableForUsers('voice-video-calls', ['user123', 'user456'])
```

## Performance Monitoring

Watch for:
- Initial load time comparison (legacy vs unified)
- Animation smoothness (should be 60fps)
- Memory usage over time
- Network requests for mock data

## Next Steps

1. **Immediate**: Test all Phase 1 features listed above
2. **This Week**: Gather feedback on UI/UX improvements
3. **Next Week**: Begin Phase 2 messaging implementation
4. **Month 1**: Complete core communication features
5. **Month 2**: Full organization management
6. **Month 3**: Website builder and publishing

---

*Remember: This is experimental! The legacy UI remains stable and default for all users.*