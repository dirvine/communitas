# Smoke Test Checklist

Manual testing checklist for release validation. Complete all sections before approving a release.

**Version**: __________ **Date**: __________ **Tester**: __________

## 1. Installation Tests (~10 min)

### Fresh Install
- [ ] Download DMG from GitHub Releases
- [ ] Open DMG (mounts correctly)
- [ ] Drag to Applications (no errors)
- [ ] First launch shows Gatekeeper approval (if needed)
- [ ] App launches without crash
- [ ] No error dialogs on startup

### Upgrade Install
- [ ] Install over existing version
- [ ] Previous settings preserved
- [ ] Previous data accessible
- [ ] No migration errors in logs

### Uninstall
- [ ] Delete from Applications
- [ ] Application Support data can be removed
- [ ] No orphaned processes

**Installation Result**: [ ] PASS [ ] FAIL

---

## 2. Core Functionality (~15 min)

### Identity & Authentication
- [ ] Identity creation works
- [ ] Four-word identity displayed
- [ ] Login successful
- [ ] Logout clears session
- [ ] Re-login works

### Settings Persistence
- [ ] Change a setting
- [ ] Quit and relaunch app
- [ ] Setting preserved

### Error States
- [ ] Disconnect network
- [ ] App shows offline indicator
- [ ] Reconnect network
- [ ] App recovers gracefully

**Core Result**: [ ] PASS [ ] FAIL

---

## 3. Feature Validation (~20 min)

### Messaging
- [ ] Navigate to Messages
- [ ] View existing threads (if any)
- [ ] Open a thread
- [ ] Send a message
- [ ] Message appears in thread
- [ ] Add a reaction
- [ ] Reaction displays

### Drive
- [ ] Navigate to Drive
- [ ] View folder structure
- [ ] Upload a file
- [ ] Progress indicator shows
- [ ] File appears in list
- [ ] Download a file
- [ ] File opens correctly

### Canvas
- [ ] Navigate to Canvas
- [ ] Create new canvas (or open existing)
- [ ] Use drawing tool
- [ ] Add text element
- [ ] Changes save automatically
- [ ] Undo/redo works

### Kanban
- [ ] Navigate to Kanban
- [ ] View a board
- [ ] Create a new card
- [ ] Drag card to different column
- [ ] Card position persists
- [ ] Add label/due date
- [ ] Delete card

### Calls
- [ ] Navigate to Calls
- [ ] Device selection available
- [ ] Microphone detected
- [ ] Camera detected (if present)
- [ ] Start call UI works

**Features Result**: [ ] PASS [ ] FAIL

---

## 4. Update Flow (~10 min)

### Update Check
- [ ] Go to Settings > Updates
- [ ] Current version displays correctly
- [ ] "Check for Updates" button works
- [ ] Shows "Up to date" or available update

### If Update Available
- [ ] Release notes display
- [ ] Download button works
- [ ] Progress bar shows
- [ ] Install option appears

**Update Result**: [ ] PASS [ ] FAIL

---

## 5. Onboarding Tour (~5 min)

- [ ] Go to Settings > Start Tour
- [ ] Tour overlay appears
- [ ] Welcome step shows
- [ ] Next button advances
- [ ] Previous button works
- [ ] Step counter accurate
- [ ] Escape key closes tour
- [ ] Finish button completes tour
- [ ] Tour doesn't reappear on next launch

**Onboarding Result**: [ ] PASS [ ] FAIL

---

## 6. Accessibility (~10 min)

### Keyboard Navigation
- [ ] Tab moves focus between elements
- [ ] Enter activates buttons
- [ ] Escape closes modals
- [ ] Arrow keys work in lists

### Visual
- [ ] Text readable at default size
- [ ] Contrast sufficient
- [ ] Focus indicators visible

### Screen Reader (if available)
- [ ] Main regions announced
- [ ] Buttons have labels
- [ ] Status changes announced

**Accessibility Result**: [ ] PASS [ ] FAIL

---

## 7. Platform Specific

### macOS Intel (x86_64)
- [ ] App launches
- [ ] Core features work
- [ ] No architecture errors

### macOS Apple Silicon (aarch64)
- [ ] App launches natively (not Rosetta)
- [ ] Core features work
- [ ] Performance acceptable

**Platform Result**: [ ] PASS [ ] FAIL

---

## 8. Edge Cases (~5 min)

- [ ] Minimize app, restore
- [ ] Resize window
- [ ] Full screen mode
- [ ] Close and reopen window
- [ ] Sleep/wake with app running
- [ ] Multiple instances prevented (or handled)

**Edge Cases Result**: [ ] PASS [ ] FAIL

---

## Summary

| Section | Result |
|---------|--------|
| Installation | [ ] PASS [ ] FAIL |
| Core Functionality | [ ] PASS [ ] FAIL |
| Feature Validation | [ ] PASS [ ] FAIL |
| Update Flow | [ ] PASS [ ] FAIL |
| Onboarding Tour | [ ] PASS [ ] FAIL |
| Accessibility | [ ] PASS [ ] FAIL |
| Platform Specific | [ ] PASS [ ] FAIL |
| Edge Cases | [ ] PASS [ ] FAIL |

**Overall Result**: [ ] APPROVED [ ] BLOCKED

**Notes**:
_________________________________________________________________________
_________________________________________________________________________
_________________________________________________________________________

**Signed**: __________________ **Date**: __________
