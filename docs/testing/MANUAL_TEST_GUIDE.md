# Manual Test Guide for User Creation Flow

## Current Issue
The "SIGN IN" text you see in the top-right corner when logged in is NOT a button - it's just a label. The actual interactive element is the user avatar (circular icon with the first letter of the username).

## Steps to Create a New User

### 1. Sign Out Current User (if logged in)
- Look for the circular avatar icon in the top-right (it will have a letter like "R" or "F")
- Click on the avatar circle (not the "SIGN IN" text)
- A dropdown menu should appear
- Click "Sign Out" from the menu
- The page should refresh and show you're logged out

### 2. Click the Actual Sign In Button
- After logging out, a real "Sign In" button should appear
- This button will be clickable and should open a dialog

### 3. Switch to Create Identity Mode
- In the dialog that opens, look for a "Create Identity" tab or button
- Click it to switch to creation mode

### 4. Fill in the Form
- **Four Words**: Enter a unique four-word identifier (e.g., "lake-forest-river-wind")
- **Display Name**: Enter your desired display name
- **Device Name**: Enter a name for this device

### 5. Submit the Form
- Click the "Create Identity" or "Create" button
- Wait for the identity to be created
- You should be automatically logged in with your new identity

## Why the Current Implementation Seems Broken

The issue is that the app is caching authentication in multiple places:
1. **localStorage** - Browser's local storage
2. **IndexedDB** - Offline storage service for persistence
3. **Tauri backend** - If running in Tauri, it may have its own session

When you refresh the page, even after clearing localStorage, the app restores the session from IndexedDB (see `AuthContext.tsx` line 135: `const cachedIdentity = await offlineStorage.get<UserIdentity>('current_identity')`).

## To Fully Clear Authentication

Run this in the browser console:
```javascript
// Clear all storage
localStorage.clear();
sessionStorage.clear();

// Clear IndexedDB
indexedDB.databases().then(databases => {
    databases.forEach(db => {
        indexedDB.deleteDatabase(db.name);
    });
});

// Reload the page
location.reload();
```

## The Real Problem

The AuthStatus component (lines 107-118 in `AuthStatus.tsx`) only shows a Sign In button when `!authState.isAuthenticated`. When authenticated, it shows the user avatar instead. The "SIGN IN" text visible when logged in is just a visual label, not an interactive button.

## Recommended Fix

To make the UI clearer, the AuthStatus component should:
1. Not show "SIGN IN" text when already logged in
2. Make the avatar more obviously clickable (add hover effects)
3. Or add a visible dropdown arrow next to the avatar

## Test the Fix

1. Open browser DevTools
2. Go to Application tab
3. Clear all storage (Local Storage, Session Storage, IndexedDB)
4. Refresh the page
5. You should now see the actual Sign In button
6. Click it to open the login dialog
7. Create a new identity

This should work correctly once the app is in a logged-out state.