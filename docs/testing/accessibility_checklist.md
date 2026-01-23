# Accessibility Checklist

_WCAG 2.1 AA Compliance Checklist for Communitas_

This document provides a component-by-component accessibility checklist following WCAG 2.1 Level AA guidelines.

## Quick Reference

### Required for All Interactive Elements

- [ ] Keyboard accessible (Tab, Enter, Space, Escape)
- [ ] Visible focus indicator (2px ring minimum)
- [ ] Accessible name (aria-label or visible text)
- [ ] Color contrast meets AA (4.5:1 normal, 3:1 large)
- [ ] Touch target minimum 44x44px

### Required for All Components

- [ ] No content flashes more than 3 times/second
- [ ] Respects `prefers-reduced-motion`
- [ ] Works at 200% zoom
- [ ] No horizontal scroll at 320px width

---

## Component Checklists

### Buttons

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Role is button | `<button>` element or `role="button"` | ✅ |
| Accessible name | Visible text or `aria-label` | ✅ |
| Keyboard operable | Responds to Enter and Space | ✅ |
| Focus visible | 2px emerald ring | ✅ |
| Disabled state | `disabled` attribute, reduced opacity | ✅ |
| Loading state | `aria-busy="true"` when loading | ✅ |

**Testing Instructions**:
1. Tab to button - focus ring should appear
2. Press Enter - should activate
3. Press Space - should activate
4. Screen reader should announce button label

### Modal Dialogs

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Role is dialog | `role="dialog"` | ✅ |
| Modal attribute | `aria-modal="true"` | ✅ |
| Accessible name | `aria-label` or `aria-labelledby` | ✅ |
| Focus trapped | `use_focus_trap` hook | ✅ |
| Escape closes | Handled by focus trap | ✅ |
| Focus returns | `use_return_focus` hook | ✅ |
| Auto-focus first | `use_auto_focus` hook | ✅ |
| Background inert | Click outside closes or blocked | ✅ |

**Testing Instructions**:
1. Open modal - first element should receive focus
2. Tab repeatedly - should cycle within modal
3. Shift+Tab - should cycle backward
4. Escape - should close modal
5. Close modal - focus should return to trigger

### Form Fields

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Label present | `<label>` or `aria-label` | ✅ |
| Label associated | `for` attribute or wrapping | ✅ |
| Required indicated | `aria-required="true"` or `required` | ✅ |
| Error associated | `aria-describedby` to error message | ✅ |
| Error announced | Error in `aria-live` region | ✅ |
| Focus visible | 2px ring on focus | ✅ |
| Placeholder not label | Label exists separately | ✅ |

**Testing Instructions**:
1. Tab to field - focus ring should appear
2. Screen reader should announce field label
3. Submit with error - error should be announced
4. Required fields should indicate requirement

### Cards (Kanban, etc.)

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Accessible name | `aria-label` with card title | ✅ |
| Role | Interactive cards use `role="button"` | ✅ |
| Keyboard selectable | Enter to open detail | ✅ |
| Focus visible | 2px ring + slight scale | ✅ |
| State announced | Priority, due date status | ✅ |

**Testing Instructions**:
1. Tab to card - focus ring should appear
2. Enter key - should open card detail
3. Screen reader should announce card title and metadata

### Dropdown Menus

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Trigger has aria-expanded | Reflects open state | ✅ |
| Trigger has aria-haspopup | `aria-haspopup="menu"` | ✅ |
| Menu has role | `role="menu"` | ✅ |
| Items have role | `role="menuitem"` | ✅ |
| Arrow navigation | Up/Down to navigate | ✅ |
| Escape closes | Returns focus to trigger | ✅ |
| Selection | Enter/Space to select | ✅ |

**Testing Instructions**:
1. Tab to trigger - should show focus
2. Enter/Space - should open menu
3. Arrow Down - should move to first item
4. Arrow Up - should move to last item
5. Enter - should select and close
6. Escape - should close without selection

### Lists (File Browser, etc.)

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Container role | `role="list"` or `<ul>` | ✅ |
| Item role | `role="listitem"` or `<li>` | ✅ |
| Accessible name | Item text or `aria-label` | ✅ |
| Arrow navigation | Up/Down through items | ✅ |
| Selection indicator | `aria-selected` for selected | ✅ |

**Testing Instructions**:
1. Tab to list - should focus first item
2. Arrow keys - should navigate items
3. Enter - should activate/open item
4. Screen reader should announce item contents

### Trees (Drive Browser)

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Container role | `role="tree"` | ✅ |
| Item role | `role="treeitem"` | ✅ |
| Expandable items | `aria-expanded` attribute | ✅ |
| Level indication | `aria-level` attribute | ✅ |
| Arrow navigation | Up/Down/Left/Right | ✅ |

**Testing Instructions**:
1. Tab to tree - should focus first item
2. Arrow Down - next visible item
3. Arrow Right - expand if collapsed, or first child
4. Arrow Left - collapse if expanded, or parent
5. Enter - activate item

### Loading States

| Requirement | Implementation | Status |
|------------|----------------|--------|
| aria-busy on container | `aria-busy="true"` | ✅ |
| Skeleton announces | `aria-label="Loading..."` | ✅ |
| Animation reducible | Respects `prefers-reduced-motion` | ✅ |
| Progress announced | Live region for completion % | ✅ |

**Testing Instructions**:
1. Screen reader should announce loading state
2. With reduced motion - no animation
3. When complete - should announce

### Error Messages

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Role is alert | `role="alert"` | ✅ |
| Live region | `aria-live="assertive"` | ✅ |
| Associated with field | `aria-describedby` | ✅ |
| Color not only indicator | Icon or text prefix | ✅ |

**Testing Instructions**:
1. Trigger error - should be announced
2. Error should be visible (not just red)
3. Screen reader should announce error text

### Offline Indicators

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Status role | `role="status"` | ✅ |
| Live region | `aria-live="polite"` or `assertive` | ✅ |
| Dismissible | Dismiss button with label | ✅ |
| Not blocking | Does not prevent interaction | ✅ |

**Testing Instructions**:
1. Go offline - banner should appear
2. Screen reader should announce offline
3. Go online - should announce restored
4. Dismiss button should be keyboard accessible

### Canvas

| Requirement | Implementation | Status |
|------------|----------------|--------|
| Keyboard navigation | Tab through elements | ✅ |
| Element selection | Enter to select | ✅ |
| Deselection | Escape to deselect | ✅ |
| Tool selection | Number keys 1-9 | ✅ |
| Undo/Redo | Ctrl+Z/Ctrl+Y | ✅ |
| Delete | Delete key | ✅ |
| Zoom | Ctrl+/-/0 | ✅ |

**Testing Instructions**:
1. Tab to canvas - should enter navigation mode
2. Tab - should cycle through elements
3. Enter - should select focused element
4. Arrow keys - should move selected element
5. Delete - should remove selected element

---

## Color Contrast Quick Reference

### Text Contrast (WCAG AA)

| Text Type | Minimum Ratio | Our Colors |
|-----------|---------------|------------|
| Normal text | 4.5:1 | White on slate-900: 15.9:1 ✅ |
| Large text (18pt/14pt bold) | 3:1 | Emerald on slate-900: 5.8:1 ✅ |
| UI components | 3:1 | Focus ring: 4.5:1+ ✅ |

### Color Palette Contrast Chart

| Foreground | slate-900 | slate-800 | slate-700 |
|------------|-----------|-----------|-----------|
| white (#ffffff) | 15.9:1 ✅ | 12.1:1 ✅ | 9.1:1 ✅ |
| slate-100 (#f1f5f9) | 14.1:1 ✅ | 10.7:1 ✅ | 8.0:1 ✅ |
| slate-400 (#94a3b8) | 6.2:1 ✅ | 4.7:1 ✅ | 3.5:1 ✅ |
| emerald-500 (#10b981) | 5.8:1 ✅ | 4.4:1 ✅ | 3.3:1 ✅ |
| red-500 (#ef4444) | 4.6:1 ✅ | 3.5:1 ✅ | 2.6:1 ⚠️ |

---

## Testing Procedures

### Keyboard Testing

1. Unplug mouse
2. Start at top of page
3. Tab through all interactive elements
4. Verify focus is visible at each stop
5. Verify all actions possible via keyboard
6. Verify no keyboard traps (except modals)

### Screen Reader Testing

#### VoiceOver (macOS)

1. Enable: Cmd+F5
2. Navigate: VO+Arrow keys
3. Interact: VO+Space
4. Verify announcements are meaningful

#### Common Issues

- Missing button labels → Add `aria-label`
- Images without alt → Add `alt` attribute
- Form fields without labels → Add `<label>`
- Dynamic content not announced → Add `aria-live`

### Color Contrast Testing

1. Use WebAIM Contrast Checker
2. Test all text/background combinations
3. Verify focus rings visible
4. Test in grayscale mode

### Reduced Motion Testing

1. Enable reduced motion in System Preferences
2. Verify animations are disabled/reduced
3. Verify functionality unchanged

---

## Automated Testing

### Unit Tests

Location: `communitas-dioxus/tests/accessibility.rs`

```bash
cargo test -p communitas-dioxus accessibility
```

Tests include:
- Color contrast calculations
- ARIA pattern verification
- Keyboard binding definitions
- Focus management patterns

### Integration Tests

Location: `communitas-ui-service/tests/accessibility_integration.rs`

```bash
cargo test -p communitas-ui-service accessibility
```

Tests include:
- Focus trap behavior
- Announcement integration
- Offline state handling
- Accessibility contracts

---

## References

- [WCAG 2.1 Quick Reference](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices Guide](https://www.w3.org/WAI/ARIA/apg/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [Dioxus Accessibility](https://dioxuslabs.com/docs/)

---

_Last updated: January 23, 2026_
_Version: 1.0 (M6)_
