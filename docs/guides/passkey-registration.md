# Passkey Registration Guide

> Status (2026-02-02): Passkey support is deferred and currently unavailable. This guide is retained for future reintroduction.

**What is a Passkey?**

A passkey is a biometric way to secure your Communitas identity. Instead of remembering a password, you use:
- **Touch ID** on Mac/iPhone
- **Face ID** on Mac/iPhone
- **Windows Hello** on Windows (face or fingerprint)
- **Android Biometric** on Android
- **Face/Fingerprint unlock** on other devices

Once registered, logging in is as simple as touching your device or looking at it.

## Quick Start

### Step 1: Open Identity Settings

1. Open Communitas
2. Click your identity name (four words) in the top bar
3. Select "Manage Identity..."
4. Go to "Security" tab

### Step 2: Register Passkey

1. Click "Register Passkey"
2. Choose a device name (e.g., "My MacBook" or "Work iPhone")
3. Click "Register"
4. Your device will ask for biometric confirmation
   - **Touch ID:** Place finger on reader
   - **Face ID:** Look at camera
   - **Windows Hello:** Face or fingerprint
5. Wait for confirmation

### Step 3: Verify Success

You should see:
- ✅ Green checkmark next to your identity
- "Last registered: Just now"
- Device name showing in the list

**That's it!** Your passkey is now registered.

## Step-by-Step Instructions by Device

### macOS (Touch ID or Face ID)

#### What You Need
- Mac with Touch ID (MacBook, Magic Keyboard) or Face ID (newer models)
- Communitas 0.8+
- Touch ID/Face ID enabled in System Settings

#### Registration Steps

1. **Open Communitas** and click your identity
2. **Select "Security"** tab
3. **Click "Register Passkey"**
4. **Enter device name** (e.g., "MacBook Pro 14-inch")
5. **Click "Register"**
6. **When prompted:**
   - For Touch ID: Place your registered finger on the Touch ID sensor
   - For Face ID: Face the camera and allow it to recognize your face
7. **Wait for confirmation** (usually 2-3 seconds)
8. **See success message** with passkey details

#### Troubleshooting on macOS

**"Touch ID/Face ID not available"**
- Go to System Settings > Biometric Settings
- Verify Touch ID/Face ID is enabled
- Add or re-enroll your biometric
- Restart Communitas and try again

**"Passkey registration failed"**
- Make sure your biometric is properly enrolled
- Try again in better lighting conditions
- Restart Communitas
- If persistent, report issue with error details

### Windows (Windows Hello)

#### What You Need
- Windows 10/11 with Windows Hello enabled
- Camera (for face recognition) or fingerprint reader
- Communitas 0.8+

#### Registration Steps

1. **Open Communitas**
2. **Click your identity** and select "Security"
3. **Click "Register Passkey"**
4. **Enter device name** (e.g., "Work Computer")
5. **Click "Register"**
6. **When Windows Hello prompt appears:**
   - For face: Look at your camera
   - For fingerprint: Place your finger on the reader
7. **Wait for verification** (2-3 seconds)
8. **See success confirmation**

#### Troubleshooting on Windows

**"Windows Hello not configured"**
- Go to Settings > Accounts > Sign-in options
- Under "Windows Hello," select your biometric type
- Enroll your face or fingerprint
- Restart Communitas and try again

**"Camera/Fingerprint reader not responding"**
- Check Device Manager for hardware issues
- Update camera/reader drivers
- In Settings, go to Privacy > Camera/Biometric
- Ensure Communitas has permission
- Restart and try again

### iPhone/iPad (Face ID or Touch ID)

#### What You Need
- iPhone/iPad with Face ID or Touch ID
- Communitas iOS app 0.8+
- Biometric enrolled in Settings

#### Registration Steps

1. **Open Communitas**
2. **Tap your identity** (four words at top)
3. **Select "Security"**
4. **Tap "Register Passkey"**
5. **Choose device name** (e.g., "My iPhone 15")
6. **Tap "Register"**
7. **When prompted:**
   - For Face ID: Look at camera and let it scan your face
   - For Touch ID: Place your registered finger on sensor
8. **See success screen**

#### Tips for iPhone/iPad
- Make sure your Face ID/Touch ID is properly set up
- If wearing glasses with Face ID, try registering with/without them
- Keep device at natural viewing angle for Face ID
- Use the same biometric registered on your device

### Android (Biometric)

#### What You Need
- Android 9+ with biometric (fingerprint, face unlock, etc.)
- Communitas Android app 0.8+
- Biometric enrolled in Settings

#### Registration Steps

1. **Open Communitas app**
2. **Tap identity** (four words)
3. **Select "Security"** tab
4. **Tap "Register Passkey"**
5. **Enter device name** (e.g., "My Android Phone")
6. **Tap "Register"**
7. **When prompted, use your biometric:**
   - Fingerprint: Tap sensor
   - Face: Look at camera
8. **Confirmation appears**

## Advanced Options

### Multiple Passkeys

You can register passkeys on multiple devices for your identity:

1. Different devices (iPhone and MacBook, etc.)
2. Same device with different biometrics if available
3. Backup passkeys on a family member's device (not recommended for security)

#### Registering on Another Device

1. On new device, open Communitas
2. Log in with your four-word identity and password
3. Go to Security
4. Click "Register Passkey"
5. Repeat registration steps

Now you can log in from either device using their biometric!

### Managing Registered Passkeys

To see all your registered passkeys:
1. Open Communitas
2. Tap your identity
3. Select "Security" → "Manage Passkeys"
4. You'll see:
   - Device name
   - Date registered
   - Last used date
   - Option to delete

### Deleting a Passkey

If you lose access to a device or want to remove a passkey:

1. **From another device with passkey:**
   - Go to Security → Manage Passkeys
   - Find the passkey to delete
   - Click "Delete"
   - Confirm deletion

2. **From any device with password:**
   - Log in with password instead
   - Go to Security
   - Click "Delete All Passkeys"
   - Confirm

## Important Security Notes

### Biometric Privacy
- Biometric data (fingerprint, face scan) never leaves your device
- Not stored in Communitas cloud
- Only used locally by your device for authentication

### Device Binding
- Each passkey is bound to a specific device
- Losing a device means losing that passkey
- Stealing a device doesn't let thieves use your passkey without your biometric

### Lost Device
If you lose a device with a registered passkey:
1. On another device, go to Security → Manage Passkeys
2. Delete the passkey from the lost device
3. From that point on, the lost device can't access your account

### What If You Forget?
Passkeys are tied to your device's biometric. You need:
- Your device
- Your biometric enrolled on that device

**Alternate login methods:**
- Use password to log in from any device
- Or log in from another device where you've registered a passkey

## Common Issues

### "Passkey Registration Failed"

**Possible causes:**
1. **Biometric not enrolled** - Set up Touch ID, Face ID, etc. first
2. **Biometric timed out** - Start over and authenticate faster
3. **Device not supported** - Some older devices lack biometric hardware
4. **Permissions** - Communitas might lack permission to use biometric

**Solutions:**
1. Verify your biometric is set up and working (test in device Settings)
2. Restart Communitas and try again
3. Update Communitas to latest version
4. Check app permissions in device Settings

### "Biometric Not Available"

**This means:**
- Your device doesn't have compatible biometric hardware, OR
- Biometric is disabled in settings, OR
- You haven't enrolled any biometric yet

**Fix:**
1. Go to device Settings
2. Find Biometric/Fingerprint/Face/Security
3. Enroll your fingerprint or face
4. Return to Communitas and try again

### "Can't Register Passkey on My Device"

**Possible reasons:**
- Device is too old (pre-2018 devices may not support)
- Device is in guest or restricted mode
- Biometric hardware is broken
- Device is in airplane mode (unlikely to be the issue, but check)

**Solutions:**
1. Try on a different device
2. Use password authentication instead
3. Contact support with device model and error message

## Best Practices

### ✅ Do

- ✅ Register passkeys on multiple devices you own
- ✅ Use strong device lock (PIN, password, or biometric)
- ✅ Keep device software updated
- ✅ Review your registered passkeys occasionally
- ✅ Delete passkeys from devices you no longer use
- ✅ Add a recovery method (backup codes or email)

### ❌ Don't

- ❌ Register passkey on a device you don't own
- ❌ Share your biometric (of course!)
- ❌ Leave device unlocked unattended
- ❌ Ignore notifications about new passkey registrations
- ❌ Use outdated devices with unpatched security holes
- ❌ Register more than 5 passkeys (management gets complex)

## Recovery

### If You Lose Access to All Passkeys

1. **You still have your password:**
   - Log in with password on any device
   - Re-register new passkeys as needed

2. **You forgot your password:**
   - Use email recovery (if set up)
   - Contact support with identity proof

3. **You lost all devices:**
   - Recovery codes (if saved)
   - Contact account recovery process

**Tip:** Set up recovery codes during passkey registration!

## FAQ

**Q: Can someone steal my passkey?**
A: Only if they have your device AND your biometric. Passkey is encrypted and tied to your specific biometric, so it can't be used on another device or without your fingerprint/face.

**Q: What if my biometric changes (fingerprint injury, beard growth)?**
A: Your device's biometric system is independent. Just re-authenticate in device settings. Communitas will ask for your current registered biometric.

**Q: Can I use the same passkey on multiple Communitas accounts?**
A: No, each account needs its own passkey. They're tied to your four-word identity, not your device.

**Q: Is it safe to register a passkey on a borrowed device?**
A: No, don't do it. The device owner could impersonate you. Only register on devices you own.

**Q: What happens when I upgrade my device?**
A: You'll need to re-register a passkey on the new device. It's a separate device, so a separate passkey.

**Q: Can I back up my passkey?**
A: Passkeys are automatically backed up by your device's backup system (iCloud, Google Drive, Microsoft OneDrive). When you restore the device, the passkey comes back.

**Q: Why does my passkey sometimes ask for password instead of biometric?**
A: Security feature - after a certain time period (usually 5-10 minutes) without using the device, it requires the password for additional security.

## Next Steps

- [Using Passkey to Log In](./passkey-authentication.md)
- [Passkey Troubleshooting](./passkey-troubleshooting.md)
- [Security Best Practices](./passkey-security.md)
- [Passkey API Reference](../api/passkey-webauthn-api.md)

---

**Need Help?**
- Email: support@saorsalabs.com
- GitHub Issues: https://github.com/saorsa-labs/communitas/issues
- Discord: [Communitas Community](https://discord.gg/communitas)

**Last Updated:** January 25, 2026
**Version:** 1.0
