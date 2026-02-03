# Passkey Security Best Practices

> Status (2026-02-02): Passkey support is deferred and currently unavailable. This guide is retained for future reintroduction.

**Securing your Communitas identity with passkeys**

This guide covers security considerations for passkey-based authentication in Communitas.

## Understanding Passkey Security

### How Passkeys Work Securely

```
Traditional Passwords:
  You → Password → Server (Server stores password hash)
  Problem: Server can be hacked, password can be stolen

Passkeys (WebAuthn):
  You → Biometric → Local Device → Passkey (never leaves device)
           ↓
     Signed Challenge
           ↓
         Server (verifies signature only)

  Secure: Biometric never sent, passkey never leaves device
```

### The Security Chain

Your Communitas account security depends on:

1. **Device Security** (60% of security)
   - Device lock (PIN/password)
   - Device biometric security
   - Keep device software updated

2. **Passkey Security** (30% of security)
   - Stored in platform keyring (hardware protected)
   - Tied to your biometric
   - Unique per device

3. **Account Security** (10% of security)
   - Recovery methods (email, codes)
   - Login monitoring
   - Session management

## Device Security

### Protect Your Device

The strongest passkey is only as safe as the device it's on.

#### Level 1: Basic Protection ✅

**Minimum requirements:**
- [ ] Device lock enabled (PIN, password, or biometric)
- [ ] All biometrics enrolled and working
- [ ] Lock enabled immediately (not 5 minutes)
- [ ] Device software updated

**Setup:**
- **Mac:** System Preferences → Security & Privacy → Require password immediately
- **Windows:** Settings → Accounts → Sign-in options → Require sign-in
- **iPhone:** Settings → Face ID/Touch ID → Enable
- **Android:** Settings → Security → Screen lock

#### Level 2: Recommended Protection ✅✅

Everything from Level 1, plus:
- [ ] Strong device PIN/password (6+ characters, mixed case)
- [ ] Biometric timeout after 1-3 minutes
- [ ] Disable USB/USB-C debugging
- [ ] Use strong WiFi password
- [ ] Auto-lock after 5 minutes

**Setup:**
- **Mac:** Use strong password + Touch ID
- **Windows:** Use strong password + Windows Hello
- **iPhone:** Strong passcode + Face ID (both)
- **Android:** Strong PIN + fingerprint

#### Level 3: Maximum Protection ✅✅✅

Everything from Level 2, plus:
- [ ] Very strong device password (15+ characters, symbols)
- [ ] Biometric + password for sensitive access
- [ ] Physical security features (case, lock, etc.)
- [ ] Regular security audits
- [ ] Encrypted backups only

**Setup for highly sensitive identities:**
- Use full recovery options
- Monitor sessions actively
- Regular security checkups

### Software Updates

**Why updates matter:**
- Fix security vulnerabilities
- Improve biometric recognition
- Update platform keyring security

**Best practices:**
- [ ] Update OS as soon as available
- [ ] Update Communitas regularly
- [ ] Enable auto-updates if available
- [ ] Restart device after updates

**On different platforms:**
- **Mac:** System Settings → General → Software Update
- **Windows:** Settings → Update & Security
- **iPhone:** Settings → General → Software Update
- **Android:** Settings → About → System Update

## Passkey Specific Security

### Register Securely

When registering a passkey:

**Do:**
- ✅ Register only on devices you own
- ✅ Use strong device lock
- ✅ Complete registration immediately (don't wait)
- ✅ Verify success message appears
- ✅ Test passkey works before forgetting password

**Don't:**
- ❌ Register on borrowed/public devices
- ❌ Register on devices you're about to sell
- ❌ Share your biometric with others
- ❌ Skip device lock setup
- ❌ Register multiple identities with same biometric on shared device

### Use Securely

When using a passkey to log in:

**Do:**
- ✅ Use on your own device
- ✅ Keep device physically secure
- ✅ Lock device after use
- ✅ Monitor biometric access
- ✅ Log out after sensitive work

**Don't:**
- ❌ Let others use your device with passkey
- ❌ Register passkey on shared device
- ❌ Leave device unattended while logged in
- ❌ Use in public where others can see
- ❌ Lend device to others with passkey registered

### Delete Securely

When removing a passkey:

**Before selling/giving away device:**
1. Log in to Communitas
2. Go to Security → Manage Passkeys
3. Delete the passkey
4. Verify it's gone
5. Only then transfer/sell device

**Before disposing of device:**
1. If possible, delete passkey first
2. Otherwise, factory reset device
3. Let OS erase all data
4. Or use device destruction service

## Multi-Device Security

### Managing Multiple Passkeys

**Same identity, different devices:**
- Each device has its own separate passkey
- Losing one device doesn't affect others
- Each biometric is independent

**Passkey Matrix Example:**
```
Identity: ocean-forest-moon-star

Device A (MacBook) → Passkey A (Touch ID)
Device B (iPhone)  → Passkey B (Face ID)
Device C (iPad)    → Passkey C (Touch ID)

If Device A is lost:
  - Delete Passkey A
  - Devices B & C still work
  - Device A can't access account anymore
```

### Which Devices to Register On

**Recommended:**
- ✅ Personal laptop (MacBook, Windows laptop)
- ✅ Personal phone (iPhone, Android)
- ✅ Tablet you own
- ✅ Desktop at home

**Not Recommended:**
- ❌ Work computer (employer controls)
- ❌ Shared family device
- ❌ Public computers
- ❌ Borrowed devices
- ❌ Kiosk/tablet in public

### Passkey Isolation

**Best practice:** Keep passkeys on isolated devices

```
High Security Identity:
  - Only on personal iPhone
  - Never on work computer
  - Never on cloud-synced devices

Work Identity:
  - On work computer
  - On personal phone (separate from high security)
  - Not on shared devices

Public Identity:
  - Can be on more devices
  - Acceptable on borrowed device
  - Less sensitive operations
```

## Lost Device Scenarios

### If You Lose a Device

**Immediate action (within hours):**

1. Log in to Communitas from ANOTHER device
2. Go to Security → Manage Passkeys
3. Find the lost device's passkey
4. Click "Delete"
5. Confirm deletion

**What happens:**
- ✅ Lost device can't access your account anymore
- ✅ Even with your biometric, it won't work
- ✅ Your data is safe
- ✅ Other devices still work

**Follow-up:**
1. Contact device carrier to report loss (for theft protection)
2. Request device wipe via "Find My" service
3. If device recovered, can re-register passkey

### If You Lose Multiple Devices

**Worst case scenario (all devices with passkeys lost):**

1. Use password from any OTHER device
2. Log in with password
3. Delete all lost device passkeys
4. Re-register on new devices as needed

**If no other device available:**
1. Use recovery email
2. Go through account recovery process
3. May need identity verification
4. Then re-register passkeys

## Password Fallback Security

### When to Use Password

**Keep password secure even with passkeys:**
- Passkey unavailable (device broken)
- New device without passkey
- Account recovery scenario
- Testing purposes

**Protect your password:**
- ✅ Use strong password (15+ characters)
- ✅ Use password manager
- ✅ Change regularly (every 90 days)
- ✅ Don't reuse password elsewhere
- ✅ Don't share password with anyone

**When to change password:**
- [ ] Every 90 days (minimum)
- [ ] After device compromise
- [ ] After suspected breach
- [ ] Before/after traveling
- [ ] When setting up new device

## Biometric Security

### Protecting Your Biometric Data

Your biometric is your "master key" - protect it carefully.

**Biometric never leaves your device:**
- ✅ Stored locally on device only
- ✅ Not sent to Communitas servers
- ✅ Not stored in cloud
- ✅ Not accessible to apps
- ✅ Hardware-protected when possible

**You should protect:**
- ✅ Device with biometric enrolled
- ✅ Physical access to device
- ✅ Device lock PIN/password
- ✅ Who you let enroll biometric

### Multi-Biometric Strategy

**Best practice for sensitive accounts:**

Use multiple biometrics on same device:
- Register fingerprint AND face (if available)
- Provides redundancy if one fails
- Backup authentication method
- Still requires your biometric (not someone else's)

**Example (iPhone):**
1. Face ID with all faces
2. Touch ID with all fingers
3. Either works for passkey auth
4. If Face ID fails, Touch ID works

## Session Security

### Sessions

Communitas maintains sessions for logged-in users.

**Session security best practices:**

- [ ] Log out when done
- [ ] Auto-logout enabled (if available)
- [ ] Regular session reviews
- [ ] Monitor active sessions
- [ ] Log out from other devices when suspicious

**Session timeouts:**
- Default: 30 days
- Sensitive: 24 hours
- High security: 1 hour

**To configure:**
- Settings → Security → Session Timeout
- Choose your risk level

### Multi-Session Monitoring

**View active sessions:**
1. Go to Security → Sessions
2. See all logged-in devices
3. View IP address and location (if available)
4. See last activity time

**If you see suspicious session:**
1. Click "Logout"
2. That device is logged out
3. If yours, log back in
4. If not yours, investigate

## Backup & Recovery

### Recovery Methods

Having backup methods is critical:

**Recommended setup:**
- [ ] Password saved in password manager
- [ ] Recovery email set up
- [ ] Recovery phone number (SMS)
- [ ] Recovery codes saved securely
- [ ] Passkeys on 2+ devices

**Recovery priority:**
1. Try passkey on any available device
2. Use password
3. Use recovery email
4. Use recovery phone
5. Contact support

### Recovery Code Management

**If Communitas provides recovery codes:**

1. Save immediately after account creation
2. Store in secure location:
   - ✅ Password manager
   - ✅ Physical safe
   - ✅ Encrypted file
3. Don't:
   - ❌ Store in email
   - ❌ Take screenshot (screenshot accessible to malware)
   - ❌ Share with anyone
   - ❌ Store on cloud without encryption

## Travel Security

### Traveling with Passkeys

**Before traveling:**
- [ ] Update all software
- [ ] Ensure passkeys registered on device
- [ ] Test passkey works locally
- [ ] Have password written down (secure way)
- [ ] Backup recovery methods

**While traveling:**
- [ ] Keep device physically secure
- [ ] Don't use public WiFi for auth
- [ ] Lock device immediately after use
- [ ] Monitor for unusual activity
- [ ] Keep device backed up

**High-risk travel:**
- Extra precaution in high-theft areas
- Consider lighter device load
- Backup identity recovery methods
- More frequent security checks

### Using VPN While Traveling

**Best practice:**
- Use reputable VPN service
- VPN helps protect network traffic
- Doesn't protect device itself
- Still needs local device lock

**Timing:**
- VPN → Device secure → Log in
- Not required for local passkey auth
- Recommended for password auth

## Compromised Device

### If Device Is Compromised

**Signs of compromise:**
- Unusual battery drain
- Device hot without reason
- Unexpected data usage
- Apps crashing frequently
- Slowdown in performance

**Immediate actions:**

1. **If passkey accessed:**
   - Log in from another device
   - Delete compromised device's passkey
   - Log out all sessions

2. **If password leaked:**
   - Change password immediately
   - From clean device
   - Update all recovery methods

3. **Scan for malware:**
   - Run full antivirus scan
   - Install security app
   - Update OS
   - Consider factory reset

4. **Monitor account:**
   - Check Sessions → Active Sessions
   - Look for unknown logins
   - Audit security settings

### Factory Reset

**For severely compromised device:**

1. Back up important data (to uncompromised location)
2. Factory reset device
   - Mac: Recovery mode erase
   - Windows: Reset this PC
   - iPhone: Settings → General → Reset → Erase All Content
   - Android: Settings → System → Reset Options → Erase All Data
3. Don't restore from backup (if compromised backup)
4. After reset, re-register passkey

## Compliance & Standards

### Security Standards

Communitas passkeys follow:
- ✅ W3C WebAuthn specification
- ✅ NIST authentication guidelines
- ✅ OWASP best practices
- ✅ Industry security standards

### Audit & Monitoring

**Communitas audits:**
- [ ] Security of webauthn-rs library
- [ ] Platform keyring integration
- [ ] Biometric handling
- [ ] Error conditions

**You should audit:**
- [ ] Your registered passkeys (monthly)
- [ ] Active sessions (weekly if sensitive)
- [ ] Device security (monthly)
- [ ] Recovery methods (quarterly)

## Security Checklist

### Initial Setup

- [ ] Device has strong lock (PIN/password)
- [ ] Biometric enrolled and working
- [ ] Device software updated
- [ ] Passkey registered on device
- [ ] Passkey tested and works
- [ ] Password saved securely (password manager)
- [ ] Recovery methods set up

### Monthly Review

- [ ] Device lock still strong
- [ ] Software up to date
- [ ] Review registered passkeys
- [ ] Check active sessions
- [ ] No suspicious activity

### Quarterly Security Audit

- [ ] Password strength review
- [ ] Recovery methods still valid
- [ ] Device physical condition
- [ ] Unnecessary passkeys deleted
- [ ] Multi-device passkey review

### Annually

- [ ] Full security assessment
- [ ] Password update
- [ ] Remove lost/old devices
- [ ] Biometric re-enrollment if needed
- [ ] Update recovery contacts

## Support Resources

### Reporting Security Issues

**Found a security vulnerability?**
1. Don't post publicly
2. Email: security@saorsalabs.com
3. Include details (not public yet)
4. Allow 48 hours for response

### Security Documentation

- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/)
- [NIST Authentication Guidelines](https://pages.nist.gov/800-63-3/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

## Related Guides

- [Passkey Registration Guide](./passkey-registration.md)
- [Passkey Authentication Guide](./passkey-authentication.md)
- [Troubleshooting Guide](./passkey-troubleshooting.md)
- [API Reference](../api/passkey-webauthn-api.md)

---

**Remember:** Security is everyone's responsibility. Passkeys are secure, but only if you protect your device.

**Last Updated:** January 25, 2026
**Version:** 1.0
**Status:** Production Ready
