# Identity Recovery Guide

This guide explains how to back up and recover your Communitas identity using a recovery phrase.

## What is a Recovery Phrase?

When you create a Communitas identity, the app generates a **24-word recovery phrase** (also called a mnemonic). This phrase is the master key to your identity - it can regenerate all your cryptographic keys and restore access to your account on any device.

**Example recovery phrase format:**
```
1. ocean      7. forest    13. moon      19. river
2. crystal   8. thunder   14. garden    20. valley
3. horizon   9. meadow    15. beacon    21. summit
4. whisper  10. cascade   16. anchor    22. glacier
5. ember    11. tempest   17. voyage    23. aurora
6. twilight 12. harbor    18. zenith    24. stellar
```

## Storing Your Recovery Phrase

### DO:

- **Write it down on paper** - Use the provided recovery card or plain paper
- **Store in multiple secure locations** - Safe deposit box, fireproof safe, trusted family member
- **Verify you wrote it correctly** - Double-check each word and its position
- **Consider a metal backup** - For protection against fire and water damage
- **Keep copies in geographically separate locations** - Protection against local disasters

### DO NOT:

- **Take photos or screenshots** - These can be hacked or synced to cloud services
- **Store in email, notes apps, or cloud storage** - These are not secure
- **Store on your computer** - Malware can steal stored phrases
- **Share with anyone** - Your phrase gives complete access to your identity
- **Enter on websites** - Communitas will NEVER ask for your phrase online

## Optional Passphrase (Advanced)

You can add an optional passphrase (sometimes called the "25th word") for extra security.

**CRITICAL: Different passphrases create completely different identities:**

- Same 24 words + NO passphrase = Identity A (e.g., "ocean-forest-moon-star")
- Same 24 words + Passphrase "xyz" = Identity B (e.g., "river-valley-summit-glacier")

These are **mathematically different identities** with different four-word names, different keys, and no connection to each other.

**What this means:**

- **Pros**: If your 24 words are stolen, the attacker still cannot access your identity without the passphrase
- **Cons**: If you forget the passphrase, your identity is permanently lost - there is NO recovery
- **Important**: The passphrase is case-sensitive and not stored anywhere

Only use a passphrase if you have a secure way to remember or store it separately from your 24 words.

## What Gets Recovered

When you recover your identity, the following are restored:

| Recovered | Not Recovered |
|-----------|---------------|
| Your identity (four-word name) | Local message history |
| Signing keys for authentication | Local files and documents |
| Encryption keys for secure communication | App settings and preferences |
| Membership in groups you joined | Cached data |
| Access to shared data | Device-specific settings |

**Important**: Local data stored only on your device is NOT backed up to the recovery phrase. Only your identity keys are derived from the phrase.

## How to Recover Your Identity

### Step 1: Start Recovery

1. Open Communitas on your new device
2. On the login screen, select **"Recover existing identity"**
3. You'll see a screen with 24 numbered word fields

### Step 2: Enter Your Recovery Phrase

1. Enter each word in the correct numbered position (1-24)
2. Words are case-insensitive (you can type lowercase)
3. The app will validate each word as you type
4. Invalid words will be highlighted in red

**Tips:**
- Enter words in order from 1 to 24
- Check for common typos (e.g., "abandon" not "abanden")
- Words come from a specific 2048-word dictionary

### Step 3: Enter Passphrase (If Used)

If you created your identity with an optional passphrase:
1. Toggle "I have a passphrase"
2. Enter your passphrase exactly as you created it
3. Remember: passphrases are case-sensitive

**⚠️ WARNING**: If you enter the wrong passphrase (or no passphrase when you used one), recovery will appear to succeed but will produce a **completely different identity**. Always verify your four-word identity matches your expected identity in Step 4.

### Step 4: Verify Your Identity

After entering your phrase, the app will:
1. Validate the phrase checksum
2. Derive your cryptographic keys
3. Display your four-word identity (e.g., "ocean-forest-moon-star")
4. Ask you to confirm this is your expected identity

### Step 5: Complete Setup

1. Verify the displayed four-word identity matches your expected identity
2. Set up any device-specific settings
3. Your identity is now recovered and ready to use

## Test Your Recovery (Recommended)

After creating your identity and securely storing your recovery phrase, **test the recovery process** before you need it in an emergency:

1. **Write down your four-word identity** (e.g., "ocean-forest-moon-star")
2. **On a second device** (or after reinstalling), select "Recover existing identity"
3. **Enter your 24 words** exactly as written
4. **Enter your passphrase** if you used one
5. **Verify the recovered identity matches** your written four-word identity

If the identities match, your backup is verified. If they don't match:
- Check for typos in your written phrase
- Verify passphrase (case-sensitive) or whether you used one at all
- Ensure word order is exactly correct

**Testing now prevents disaster later.** Don't wait until you've lost access to discover a backup problem.

## Troubleshooting

### "Invalid word" error

- Check spelling carefully
- Only words from the BIP39 English wordlist are valid
- Common confusion: "there" vs "their", "your" vs "you're"
- Try typing the first few letters and selecting from suggestions

### "Invalid checksum" error

This means one or more words are incorrect or in the wrong position:

1. Verify word order matches your written backup exactly
2. Check for transposed words (e.g., words 5 and 6 swapped)
3. Look for similar-looking words (e.g., "abandon" vs "about")
4. Ensure you have exactly 24 words

### Wrong identity recovered

If the four-word identity doesn't match what you expected:

1. **Did you use a passphrase?** Try recovery with your passphrase
2. **Wrong passphrase?** Passphrases are case-sensitive - check capitalization
3. **Different identity?** You may have multiple identities - verify you're using the correct phrase

### Recovery phrase lost

If you've lost your recovery phrase:

- There is no way to recover a lost phrase
- Communitas cannot retrieve your phrase - we never have access to it
- Your identity and any data encrypted to it will be permanently inaccessible
- You must create a new identity

## Security Considerations

### Why 24 Words?

24 words provide 256-bit security, which means:
- There are more possible combinations than atoms in the observable universe
- Even quantum computers cannot brute-force this level of security
- The same security standard used by cryptocurrency wallets worldwide

### Checksum Protection

The last word includes a checksum that verifies the other 23 words. This means:
- Typos are usually detected
- Random word combinations won't work
- You cannot guess or generate a valid phrase

### Post-Quantum Security

Communitas uses post-quantum cryptography (ML-DSA-87 and ML-KEM-768), which means:
- Your identity is protected against both classical and quantum computers
- Future advancements in quantum computing won't compromise your identity
- Level 5 security (192-bit quantum security level)

## Quick Reference Card

Cut out and store securely:

```
┌────────────────────────────────────────────────────────────┐
│                    COMMUNITAS RECOVERY                      │
│                                                             │
│  Identity: _______-_______-_______-_______                 │
│                                                             │
│  1. ________  7. ________ 13. ________ 19. ________        │
│  2. ________  8. ________ 14. ________ 20. ________        │
│  3. ________  9. ________ 15. ________ 21. ________        │
│  4. ________ 10. ________ 16. ________ 22. ________        │
│  5. ________ 11. ________ 17. ________ 23. ________        │
│  6. ________ 12. ________ 18. ________ 24. ________        │
│                                                             │
│  Passphrase used? [ ] Yes  [ ] No                          │
│                                                             │
│  KEEP THIS CARD SECURE - ANYONE WITH THESE WORDS           │
│  CAN ACCESS YOUR IDENTITY                                  │
└────────────────────────────────────────────────────────────┘
```

## Getting Help

If you're having trouble with recovery:

1. **Check this guide** - Most issues are covered in Troubleshooting
2. **Community forum** - Ask for help (never share your recovery phrase)
3. **GitHub issues** - Report bugs in the recovery process

**Remember**: Communitas support will NEVER ask for your recovery phrase. Anyone asking for your phrase is attempting to steal your identity.

---

*Last updated: January 2026*
