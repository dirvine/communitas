# Passkey Troubleshooting Guide

> Status (2026-02-02): Passkey support is deferred and currently unavailable. This guide is retained for future reintroduction.

**Troubleshooting common passkey and biometric authentication issues**

## Quick Diagnosis

If you're having passkey issues, start here:

### Are You Trying to...

**Register a passkey?** → See [Registration Issues](#registration-issues)

**Log in with a passkey?** → See [Authentication Issues](#authentication-issues)

**Delete or manage passkeys?** → See [Management Issues](#management-issues)

**Use passkeys on another device?** → See [Multi-Device Issues](#multi-device-issues)

## Registration Issues

### "WebAuthn Not Available"

**Error message:**
```
"WebAuthn not available"
```

**What this means:**
- Communitas couldn't initialize WebAuthn support
- Usually happens on startup

**Solutions:**

1. **Restart Communitas**
   - Close the app completely
   - Reopen it
   - Try again

2. **Check device compatibility**
   - WebAuthn requires biometric hardware
   - Older devices (pre-2015) may not have it
   - Check if Touch ID/Face ID/Windows Hello works in Settings

3. **Update Communitas**
   - Go to App Store / Play Store / Software Updates
   - Install latest version
   - Restart app and try again

4. **Check system permissions**
   - **Mac:** System Settings → Privacy & Security
   - **Windows:** Settings → Privacy & Security
   - **iOS:** Settings → Biometric & Authentication
   - **Android:** Settings → Security & Privacy
   - Ensure Communitas has permission to use biometric

### "Biometric Not Available" or "Face ID/Touch ID Not Setup"

**What this means:**
- Your device has biometric hardware but it's not set up
- You haven't enrolled any fingerprints or face
- Biometric is disabled

**Solutions:**

1. **Enroll biometric (if not done)**
   - **Mac:** System Settings → Touch ID (or Face ID for newer models)
   - **Windows 10/11:** Settings → Accounts → Sign-in Options → Windows Hello
   - **iPhone/iPad:** Settings → Face ID/Touch ID & Passcode
   - **Android:** Settings → Biometric & Security
   - Add fingerprint or face

2. **Re-enroll if existing biometric fails**
   - Remove old biometric from device settings
   - Enroll again from scratch
   - Restart Communitas
   - Try registration again

3. **Check if biometric works elsewhere**
   - Test in device Settings
   - Try unlocking device with biometric
   - Try using another app that uses biometric
   - If nothing works, biometric hardware may be broken

**If biometric hardware is broken:**
- Use password authentication instead
- Schedule device repair/replacement
- Contact device manufacturer

### "Passkey Already Registered for This Device"

**What this means:**
- You've already registered a passkey on this device for this identity
- Can't register twice on same device

**Solutions:**

1. **If registering on same device:**
   - Go to Security → Manage Passkeys
   - Delete the existing passkey first
   - Wait a few seconds
   - Register a new one

2. **If registering on different device:**
   - This is normal - each device needs its own passkey
   - Open app on the OTHER device
   - Register a passkey there

3. **If you're not sure which device:**
   - Go to Security → Manage Passkeys
   - Look at "Device Name" and "Last Used"
   - Compare with device you're on

### "Passkey Registration Failed" (Generic Error)

**What this means:**
- Registration started but encountered an error
- Could be many causes

**Troubleshooting steps:**

1. **First, verify biometric works:**
   ```
   Try authenticating with device biometric in:
   - Device Settings
   - Other apps
   - Device unlock
   ```
   If biometric doesn't work anywhere, fix that first.

2. **Try again with fresh start:**
   - Close Communitas completely
   - Wait 10 seconds
   - Reopen Communitas
   - Go to Security
   - Click Register Passkey
   - This time, complete biometric quickly

3. **Check internet connection (if applicable):**
   - Some validation may require network
   - Ensure stable connection
   - Try again

4. **Device too busy:**
   - Close other apps
   - Disable background apps
   - Reduce device load
   - Try again

5. **Timeout during registration:**
   - Biometric prompt may have timed out
   - Start over and respond faster
   - Most systems wait 30-60 seconds

### Registration Stuck or Frozen

**What this means:**
- Registration started but isn't progressing
- Biometric prompt may be frozen
- App may be hanging

**Solutions:**

1. **Wait 60 seconds**
   - Sometimes slow to respond
   - Give it time

2. **Press Escape or Cancel**
   - Most prompts have a cancel button
   - Try to close the prompt

3. **Force close app**
   - **Mac/Windows:** Force quit in Task Manager / Activity Monitor
   - **Mobile:** Swipe up (iPhone) / swipe down (Android) to close
   - Restart app

4. **Restart device (nuclear option)**
   - Last resort if stuck
   - Full restart of device
   - Usually solves most issues

5. **Reinstall Communitas**
   - If persistent issues
   - Uninstall app completely
   - Reinstall fresh version
   - Try again

## Authentication Issues

### "No WebAuthn Credential Found"

**What this means:**
- You tried to log in with passkey
- But Communitas can't find the credential
- Usually means passkey wasn't properly saved

**Solutions:**

1. **Check if passkey actually registered:**
   - Log in with password instead
   - Go to Security → Manage Passkeys
   - Is your passkey listed?

2. **If passkey is NOT listed:**
   - It wasn't properly registered
   - You'll need to re-register:
     - Go to Security
     - Click Register Passkey
     - Complete registration again

3. **If passkey IS listed:**
   - Might be data corruption
   - Try:
     - Close and reopen Communitas
     - Delete the passkey
     - Re-register it

4. **On different device:**
   - Remember: different device = different passkey
   - First passkey was on a different device
   - Need to register on THIS device
   - Go to Security and register new passkey

### "Authentication Failed" or "Biometric Not Recognized"

**What this means:**
- You entered your biometric
- But it didn't match stored biometric
- Credential verification failed

**Solutions:**

1. **Try again**
   - Biometric recognition isn't perfect
   - May fail due to:
     - Different angle
     - Different lighting
     - Finger/face position
     - Smudged sensor (fingerprint)

2. **Improve conditions:**
   - **Face ID:**
     - Better lighting
     - Straight-on angle
     - Remove glasses/sunglasses if newly added
     - No mask covering face
   - **Touch ID:**
     - Clean sensor with cloth
     - Use same finger as enrolled
     - Proper finger position
   - **Windows Hello:**
     - Better lighting
     - Remove hat/glasses if newly added
     - Face not covered

3. **Check if biometric works in Settings:**
   - Device Settings → Biometric
   - Try authenticating there
   - If fails there too, issue is with device biometric

4. **Re-enroll biometric:**
   - Device settings → Biometric/Face ID/Touch ID
   - Delete enrolled data
   - Enroll again from scratch
   - Return to Communitas and try login

5. **Delete and re-register passkey:**
   - Log in with password
   - Security → Manage Passkeys
   - Delete the passkey
   - Re-register it
   - Log out and try passkey login again

### "Timeout" During Authentication

**What this means:**
- Biometric prompt was waiting too long
- You didn't respond in time (usually 30-60 seconds)
- Or connection timed out (rare)

**Solutions:**

1. **Respond faster:**
   - When biometric prompt appears, authenticate immediately
   - Don't wait too long
   - Most systems have 30-60 second timeout

2. **Try again:**
   - Close the prompt
   - Try authentication again
   - Be ready this time

3. **Check for frozen prompt:**
   - Sometimes prompt gets stuck
   - Try pressing Escape key
   - Or Cancel button
   - Restart and try again

### "Biometric Hardware Error" or "Device Not Responding"

**What this means:**
- Your biometric hardware isn't responding
- Touch ID sensor, Face ID camera, or Windows Hello not working
- May be hardware failure

**Troubleshooting:**

1. **Test hardware in device Settings:**
   - Go to device Biometric settings
   - Try to authenticate
   - Does it work?

2. **If it works in Settings but not Communitas:**
   - Close Communitas
   - Wait 30 seconds
   - Reopen
   - Try again
   - If still fails, may be Communitas permissions issue

3. **If it doesn't work in Settings either:**
   - Hardware is broken
   - Options:
     - Use password to log in (temporary)
     - Get device repaired
     - Use different device

4. **Try device restart:**
   - Sometimes helps with hardware responsiveness
   - Restart device completely
   - Try again

### "Permission Denied" or "Biometric Permission Error"

**What this means:**
- Communitas doesn't have permission to use biometric
- Operating system is blocking it

**Solutions:**

1. **Grant Communitas permission:**
   - **Mac:** System Settings → Privacy & Security → Biometric
   - **Windows:** Settings → Privacy & Security → App Permissions
   - **iOS:** Settings → Communitas → Biometric Access
   - **Android:** Settings → Apps → Communitas → Permissions
   - Enable Communitas for biometric use

2. **Restart Communitas:**
   - After changing permissions
   - Close app completely
   - Reopen
   - Try again

3. **Reset app permissions:**
   - If already granted but still failing
   - **Mobile:** Delete and reinstall app
   - **Desktop:** Settings → Apps → Communitas → Reset

## Management Issues

### Can't Delete Passkey

**What this means:**
- Delete button not working or greyed out
- Can't remove a passkey

**Solutions:**

1. **Check if logged in:**
   - You must be logged in to manage passkeys
   - Log in with password if can't use passkey
   - Then try delete again

2. **Try refreshing:**
   - Go away from passkey list
   - Return to Security
   - Try delete again

3. **Restart and retry:**
   - Close and reopen Communitas
   - Go to Security
   - Try delete

4. **Force delete:**
   - If delete doesn't work:
   - Log in with password
   - Delete all passkeys option
   - Choose "Delete All"

### Can't See Registered Passkeys

**What this means:**
- You registered a passkey but don't see it in the list
- List is empty or missing passkey

**Causes:**

1. **Passkey registered on different identity:**
   - Passkeys are per-identity
   - If you're looking at different identity, won't see it
   - Switch to correct identity first

2. **Passkey registered on different device:**
   - Passkeys are per-device
   - Each device has separate passkey
   - Can't see passkeys from other devices
   - This is normal

3. **Passkey not fully registered:**
   - Registration may have failed
   - But claimed success
   - Try registering again

4. **Data not synced yet:**
   - New registrations sometimes take a moment to appear
   - Wait 10 seconds
   - Refresh the screen

## Multi-Device Issues

### Same Identity, Different Devices

### "Can't Use Passkey on New Device"

**What this means:**
- You registered passkey on Device A
- But it doesn't work on Device B
- Or Device B doesn't recognize the identity

**This is normal!** Passkeys are per-device.

**Solutions:**

1. **Register separate passkey on Device B:**
   - Same identity, different passkey on each device
   - Log in to Device B with password
   - Go to Security
   - Register new passkey
   - Use that device's biometric

2. **Verify identity exists on new device:**
   - Device B should show identity in recent list
   - If not, add it:
     - Click "Add Identity"
     - Enter four-word identity
     - Log in with password
     - Then register passkey for this device

3. **Sync between devices:**
   - Identity info syncs automatically
   - But passkeys are device-specific
   - Each device needs its own registration

### Can't Switch Between Devices

**What this means:**
- You have passkey on Device A
- But Device B doesn't know about the identity
- Or can't switch from A to B

**Solutions:**

1. **Add identity on Device B:**
   - Open Communitas on Device B
   - Click "Add Identity"
   - Enter your four-word identity
   - Log in with password (if asked)

2. **Device B now has identity:**
   - But needs its own passkey
   - Go to Security
   - Register new passkey
   - Use Device B's biometric

3. **Now you can switch:**
   - On Device A: Log out
   - On Device B: Log in with passkey
   - Back to Device A: Log in with passkey
   - Seamless switching!

### Passkey Doesn't Work After Device Restore

**What this means:**
- You restored your device from backup
- Passkey isn't working
- Usually means biometric data changed

**Solutions:**

1. **Verify biometric still works:**
   - Test in device Settings
   - Does device biometric work?
   - If not, biometric data was lost in restore

2. **Re-enroll biometric:**
   - Device Settings → Biometric
   - Add your fingerprint/face again
   - Return to Communitas

3. **Delete and re-register passkey:**
   - Log in with password
   - Security → Manage Passkeys
   - Delete existing passkey
   - Register new one
   - Should work now

## Platform-Specific Issues

### macOS Issues

**Touch ID not working:**
- System Settings → Touch ID
- Verify enrolled
- Try authenticating in Settings first
- Restart Communitas
- Permissions: System Settings → Privacy & Security

**Face ID on newer Macs:**
- System Settings → Face ID
- Enroll face (if not done)
- May require good lighting
- Clean camera lens

### Windows Issues

**Windows Hello failing:**
- Settings → Accounts → Sign-in options
- Check Windows Hello configured
- Enroll face or fingerprint
- Update camera drivers
- Privacy settings: Settings → Privacy & Security

**TPM requirements:**
- Windows 11 may require TPM 2.0
- Check Device Manager for TPM
- May need BIOS update
- Contact IT if in work environment

### iOS Issues

**Face ID issues:**
- Settings → Face ID & Passcode
- Re-enroll face
- Good lighting helps
- Clean camera
- Remove glasses if newly added

**Touch ID issues:**
- Settings → Touch ID & Passcode
- Re-enroll fingerprint
- Clean sensor with cloth
- Try different finger

### Android Issues

**Fingerprint not working:**
- Settings → Biometric & Security
- Re-enroll fingerprint
- Clean sensor
- Try different finger

**Face unlock issues:**
- Settings → Face Unlock
- Re-enroll face
- Good lighting required
- Straight-on angle

## Data Recovery

### Lost All Passkeys

**If you deleted all passkeys:**

1. **You still have password:**
   - Log in with password
   - Re-register new passkeys if desired

2. **You forgot password:**
   - Use recovery email
   - Go to recovery process
   - Will need identity verification

3. **Urgent access needed:**
   - Contact support@saorsalabs.com
   - Provide identity verification
   - Can help with account recovery

### Device Lost

**If device with passkey is lost:**

1. **From another device with passkey:**
   - Go to Security → Manage Passkeys
   - Find lost device's passkey
   - Click Delete
   - Confirm
   - Lost device can no longer access account

2. **From any device with password:**
   - Log in with password
   - Delete the lost device's passkey
   - Or delete all passkeys if unsure

3. **Lost device AND all other devices:**
   - Use recovery email
   - Reset through account recovery
   - Re-register passkeys on available device

## Still Having Issues?

### Get More Help

1. **Check logs (desktop):**
   - Communitas → Settings → Advanced → Logs
   - Look for error messages
   - Share with support

2. **Gather information:**
   - Device type (iPhone, MacBook, etc.)
   - OS version (iOS 16.5, Windows 11, etc.)
   - Error message (exact text)
   - When did it start?
   - What were you doing?

3. **Contact Support:**
   - Email: support@saorsalabs.com
   - Subject: "Passkey Issue on [Device]"
   - Include gathered information
   - Attach any error messages/logs

4. **Community Help:**
   - Discord: https://discord.gg/communitas
   - GitHub Issues: https://github.com/saorsa-labs/communitas/issues
   - Search existing issues first

## Related Documentation

- [Passkey Registration Guide](./passkey-registration.md)
- [Passkey Authentication Guide](./passkey-authentication.md)
- [Security Best Practices](./passkey-security.md)
- [API Reference](../api/passkey-webauthn-api.md)

---

**Last Updated:** January 25, 2026
**Version:** 1.0
**Status:** Production Ready
