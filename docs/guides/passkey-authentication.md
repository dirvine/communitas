# Passkey Authentication Guide

**Using Passkey to Log In**

Once you've registered a passkey, logging in is faster and more secure than passwords. This guide shows you how to authenticate with your passkey.

## Quick Login

### For Repeat Visits

The fastest way is usually auto-login:

1. Open Communitas
2. If your identity was last used on this device, it may auto-login
3. Place finger on Touch ID / Look at Face ID / Use Windows Hello
4. You're logged in!

### For Different Identities

1. Open Communitas
2. Click "Switch Identity"
3. Select the identity with a ✓ (passkey registered)
4. Use your biometric
5. Logged in instantly!

## Full Login Flow

### Step 1: Start Communitas

Open the Communitas application. You'll see either:
- **Login screen** (if not logged in)
- **Recent identities** (if you've used them before)

### Step 2: Select Identity

If you see multiple recent identities:
1. Look for the one with a **green checkmark** (✓) or **fingerprint icon**
2. That means a passkey is registered for that identity
3. Click or tap on that identity

If you don't see your identity:
1. Click "Add Identity" or "Browse More"
2. Enter your four-word identity (e.g., "ocean-forest-moon-star")
3. If a passkey exists for this identity, it will show a passkey indicator

### Step 3: Authenticate with Passkey

When you select an identity with a passkey, you'll see a prompt like:

**"Authenticate with Touch ID"**
- MacBook: Place finger on Touch ID reader
- iPhone: Let it scan your face or touch sensor
- Windows: Look at camera or place finger on reader
- Android: Use your enrolled biometric

**You'll see:**
- 🔒 Fingerprint or face icon (animated)
- "Waiting for biometric..."
- Cancel button (if you want to use password instead)

### Step 4: Complete Authentication

When your biometric is recognized:
- ✅ "Success!" message appears
- App unlocks automatically
- You're logged in to your identity

**The whole process takes 2-3 seconds!**

## What Happens Behind the Scenes

Here's what happens when you authenticate with a passkey:

```
1. You select identity
   ↓
2. Communitas retrieves passkey challenge from encrypted storage
   ↓
3. Challenge sent to device's biometric authenticator
   ↓
4. Biometric prompt shown (Touch ID, Face ID, etc.)
   ↓
5. Device authenticates your biometric locally
   ↓
6. Device signs challenge with your passkey
   ↓
7. Signed response sent back to Communitas
   ↓
8. Communitas verifies signature
   ↓
9. ✅ Session created, you're logged in!
```

**Important:** Your biometric NEVER leaves your device. The authenticator works locally on your device only.

## Authentication on Different Devices

### Same Identity, Different Device

You can register the same identity on multiple devices with different passkeys:

**Device A (MacBook):**
1. Register passkey with Touch ID
2. Log in with Touch ID

**Device B (iPhone):**
1. Register passkey with Face ID
2. Log in with Face ID

Each device has its own passkey for the same identity.

**To set up on a new device:**
1. On the new device, open Communitas
2. Click "Add Identity"
3. Enter your four-word identity
4. Log in with password (or another device if available)
5. Go to Security and register a new passkey for this device
6. Now you can use passkey on this device too!

### Switching Devices

The fastest way to move between your devices:

1. Open Communitas on Device A (already logged in)
2. Open Communitas on Device B
3. Recent identities show because they're synced
4. Device B has passkey for this identity
5. Tap identity → Authenticate with Device B's biometric
6. Logged in on Device B!

## Login Methods Comparison

| Method | Speed | Security | Convenience |
|--------|-------|----------|-------------|
| **Passkey** | 2-3 sec | ⭐⭐⭐⭐⭐ Highest | ⭐⭐⭐⭐⭐ Easiest |
| **Password** | 5-10 sec | ⭐⭐⭐ Good | ⭐⭐⭐ OK |
| **PIN** | 3-5 sec | ⭐⭐⭐⭐ Very Good | ⭐⭐⭐⭐ Good |

**Passkey wins on both speed and security!**

## Troubleshooting

### Authentication Failed

**"Biometric Not Recognized"**

This happens when:
- Your biometric didn't match (maybe lighting is different, face angle changed)
- Biometric data got corrupted
- Device lost biometric data (very rare)

**Fix:**
1. Try again - biometric recognition can vary
2. Better lighting helps
3. If repeated failures:
   - Restart device
   - In device settings, test biometric works in other apps
   - If broken, re-enroll biometric
   - Delete passkey and re-register

**"Passkey Not Found"**

This means:
- No passkey registered for this identity on this device
- Passkey was deleted
- Data was corrupted

**Fix:**
1. Use password to log in instead
2. Register a new passkey:
   - Go to Security
   - Click "Register Passkey"
   - Complete registration

**"Device Not Responding"**

Your biometric hardware isn't responding:

**Fix:**
1. Restart the app
2. Restart the device
3. In device Settings, test biometric works in other apps
4. If broken, may need hardware repair
5. Use password login until fixed

### Slow Authentication

**"Authentication takes longer than expected"**

Possible reasons:
- Network delay (offline apps shouldn't have this)
- Device is busy with other tasks
- Biometric is slow (older devices)
- Poor biometric match (bad lighting, angle)

**Fix:**
1. Close other apps
2. Try again in better conditions
3. Restart device
4. If still slow, try password method

### Password Instead of Passkey

**"I want to use password instead"**

During login, you should see an option like:
- "Use Password Instead"
- "Password" button
- "Login Options"

Click to switch to password entry.

**If you don't see the option:**
1. Close the login prompt
2. Try "Switch Identity" again
3. Or completely close and reopen Communitas

**Why might you prefer password?**
- Biometric not working
- Device biometric registry corrupted
- Testing purposes
- Remote login (some scenarios)

## Advanced Features

### Auto-Login

If your identity was last used on THIS device, Communitas may auto-login:

1. Open Communitas
2. Biometric prompt appears automatically
3. Use your biometric
4. Instantly logged in!

**To disable auto-login:**
- Settings → Security → Auto-Login: OFF

**Note:** Auto-login requires passkey to be registered.

### Session Persistence

Once logged in, you stay logged in until:
- You explicitly log out
- You restart the device (on some platforms)
- Security timeout occurs (usually after 30+ days)

**To log out:**
1. Click your identity name
2. Select "Logout"
3. Next time, need to authenticate again

### Time-Limited Sessions

For sensitive operations, Communitas may ask to re-authenticate:
- Accessing financial information
- Deleting important data
- Changing security settings

This re-authentication requirement happens even if you're logged in.

## Security During Authentication

### What's Protected

✅ **Your biometric is protected:**
- Never sent over network
- Never stored in cloud
- Only used locally on your device
- Can't be intercepted

✅ **Your passkey is protected:**
- Encrypted in platform keyring
- Hardware-protected when available
- Can't be copied to another device

✅ **The authentication is protected:**
- Each login uses a different challenge
- Can't replay old authentications
- Man-in-the-middle protected

### What You Should Protect

❌ **Protect your device:**
- Use device lock PIN/password
- Update device software regularly
- Be careful with app permissions
- Use device biometric security

❌ **Protect physical access:**
- Don't leave device unlocked
- Don't lend device to others
- Don't use public device biometrics
- Secure device if traveling

## Multi-Device Scenarios

### Scenario 1: Home Setup

**Devices:** MacBook, iPhone, iPad

**Setup:**
- MacBook: Register passkey with Touch ID
- iPhone: Register passkey with Face ID
- iPad: Register passkey with Touch ID

**Login:**
- From any device, instantly log in with that device's biometric
- Same identity, different passkeys per device

### Scenario 2: Work Setup

**Devices:** Work laptop, personal phone

**Setup:**
- Work laptop: Register with Windows Hello
- Personal phone: Register with fingerprint

**Usage:**
- At work: Use Windows Hello
- Mobile: Use fingerprint
- Can switch between devices seamlessly

### Scenario 3: Backup Setup

**Devices:** Primary phone + backup phone

**Setup:**
- Primary: iPhone with Face ID
- Backup: Older iPhone with Touch ID

**If primary fails:**
1. Grab backup phone
2. Recent identities already there
3. Use Touch ID to log in
4. No downtime!

## Performance

### Authentication Speed by Device

| Device Type | Speed | Notes |
|------------|-------|-------|
| iPhone 14+ | ~1 sec | Fastest Face ID |
| MacBook (Touch ID) | ~2 sec | Fast, reliable |
| Windows (Face) | ~2 sec | Requires good lighting |
| iPad | ~2 sec | Good for tablet use |
| Android | ~2-3 sec | Varies by device |

### Network Latency

Since passkeys are local-first, authentication **doesn't require network**. Even if offline:
- ✅ Authentication works
- ✅ Session created
- ✅ You can use app
- ✅ Sync happens when online

## FAQ

**Q: Can someone use my passkey if they steal my device?**
A: No, they'd need your biometric too. Even with your device and password, they can't use your passkey without your fingerprint/face.

**Q: What if I'm on a different device where I haven't registered a passkey?**
A: Use password to log in, then optionally register a passkey for that device.

**Q: Can I authenticate with someone else's biometric on my device?**
A: Their biometric won't match your registered passkey, so no.

**Q: How do I know if a passkey is actually being used?**
A: Go to Security → Manage Passkeys. You'll see each registered passkey and when it was last used.

**Q: Why does it sometimes ask for password again after just logging in?**
A: For security - sensitive operations may require re-authentication even if logged in.

**Q: What if my device biometric registry gets corrupted?**
A: Delete the passkey from device settings, re-register biometric, then re-register passkey in Communitas.

**Q: Can I use the same passkey on multiple devices?**
A: No, each device has its own separate passkey for the same identity.

**Q: What if I reset my device?**
A: Passkey is lost if not backed up. Use password login, then re-register passkey.

## Related Guides

- [Passkey Registration Guide](./passkey-registration.md)
- [Passkey Troubleshooting](./passkey-troubleshooting.md)
- [Security Best Practices](./passkey-security.md)
- [API Reference](../api/passkey-webauthn-api.md)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl + Shift + I | Open identity switcher |
| Escape | Close biometric prompt |
| Enter | Confirm biometric prompt |

## Performance Tips

1. **Keep device updated** - Latest software = faster biometric
2. **Clean camera/sensor** - Especially before Face ID
3. **Good lighting** - Helps facial recognition
4. **Consistent angle** - Train biometric at angle you'll use
5. **Remove obstacles** - Glasses/masks may slow down Face ID

## Next Steps

- Review [Passkey Security Best Practices](./passkey-security.md)
- Learn about [Multi-Device Setup](./passkey-multidevice.md)
- Check [Troubleshooting Guide](./passkey-troubleshooting.md) if issues arise

---

**Need Help?**
- Email: support@saorsalabs.com
- GitHub Issues: https://github.com/saorsa-labs/communitas/issues
- Discord Community: [Join us](https://discord.gg/communitas)

**Last Updated:** January 25, 2026
**Version:** 1.0
