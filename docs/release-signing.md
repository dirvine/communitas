# Communitas release signing

Communitas ships **two** desktop apps from the same release tag:

- **Dioxus** (Tauri 2 bundle) — verifies updates against a JSON feed signed with
  post-quantum ML-DSA-65.
- **Swift** (native macOS) — verifies updates against a Sparkle appcast signed
  with Ed25519.

Both formats are emitted by the same CI job from the same Git tag. This
document describes where the secrets live, how releases are signed, how the
apps verify, and how to rotate each key. See also: `scripts/parity-check.sh`
and the x0x sibling document at `x0x/src/upgrade/signature.rs`.

## Summary

| App | Feed | Algorithm | Verifier | Public key |
|---|---|---|---|---|
| Dioxus | `update.json` + `update.json.sig` | ML-DSA-65 (post-quantum) | `communitas-dioxus/src/release_signing_key.rs` | 1024-byte embedded constant |
| Swift | `appcast.xml` + embedded `sparkle:signature` | Ed25519 (Sparkle 2) | Sparkle SPUStandardUpdaterController | `SUPublicEDKey` in `Info.plist` |

## Why two formats?

- The Dioxus app is the post-quantum flagship; ML-DSA-65 matches the signing
  scheme we already use for x0x binaries, so one secret ceremony covers both.
- The Swift app uses Sparkle, which natively supports Ed25519 via its
  `generate_keys` / `sign_update` tools. Retrofitting ML-DSA into Sparkle is a
  larger undertaking and is tracked separately.

We accept the double key-management cost to keep both apps deterministically
verifiable today rather than blocking Swift releases on a Sparkle fork.

## CI release workflow

`.github/workflows/release.yml` runs on every `v*` tag. The flow is:

1. **Build** — compile and bundle Dioxus, Swift, and any platform archives.
2. **Sign — ML-DSA-65** — for Dioxus's `update.json`:
   - Secret: GitHub secret `COMMUNITAS_ML_DSA_SECRET` (raw bytes, base64).
   - Tool: `x0x-keygen sign --context communitas-release-v1 --input update.json --output update.json.sig`.
   - Output artifacts: `update.json`, `update.json.sig`.
3. **Sign — Ed25519 (Sparkle)** — for Swift's `appcast.xml`:
   - Secret: GitHub secret `COMMUNITAS_ED25519_SECRET` (Sparkle's exported
     base64 private key).
   - Tool: Sparkle's `sign_update` CLI (shipped in Sparkle-tools in the runner).
   - Output: `appcast.xml` with `<sparkle:edSignature>` embedded on each
     `<enclosure>`.
4. **Verify (dry-run)** — before publishing the release:
   - Run `x0x-keygen verify` against `update.json.sig` with the embedded
     public key constant copied out of `release_signing_key.rs`.
   - Run `sign_update -p <embedded-ed-pub>` over a downloaded enclosure to
     confirm the Ed25519 signature round-trips.
   Any verify failure fails the release job — we do **not** publish partially
   signed artifacts.
5. **Publish** — upload `update.json`, `update.json.sig`, `appcast.xml`, and
   all platform bundles to the GitHub Release.

## Update-feed shapes

### `update.json` (Dioxus)

Pretty-printed JSON with the following required fields (see
`communitas-dioxus/src/components/settings_view.rs` for the parser):

```json
{
  "version": "0.11.8",
  "published_at": "2026-04-17T10:30:00Z",
  "notes": "Release notes…",
  "platforms": {
    "darwin-aarch64": { "url": "https://…/Communitas-arm64.dmg", "signature": "" }
  }
}
```

The ML-DSA-65 signature signs **the raw bytes of `update.json`** under
context `"communitas-release-v1"`. The per-platform `"signature"` field is
intentionally empty — we sign the envelope, not individual binaries.

### `appcast.xml` (Swift)

Standard Sparkle 2 RSS feed:

```xml
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle">
  <channel>
    <item>
      <sparkle:shortVersionString>0.11.8</sparkle:shortVersionString>
      <sparkle:version>11800</sparkle:version>
      <enclosure
        url="https://…/Communitas.dmg"
        sparkle:edSignature="Base64EdSig"
        length="42000000"
        type="application/octet-stream"/>
    </item>
  </channel>
</rss>
```

Sparkle verifies each `enclosure` against `SUPublicEDKey` from `Info.plist`.

## Key rotation

Rotating either key is a one-way change — once the next release ships,
older builds will stop accepting updates until they are upgraded once by
hand. Coordinate with the on-call before doing this.

### Rotating `COMMUNITAS_ML_DSA_SECRET` (Dioxus)

1. Generate a new keypair locally:
   ```
   x0x-keygen generate --out ~/.saorsa-keys/communitas-v2.secret
   x0x-keygen export-public \
       --in ~/.saorsa-keys/communitas-v2.secret \
       --out ~/.saorsa-keys/communitas-v2.pub
   ```
2. Update the embedded public key constant:
   ```
   x0x-keygen embed-rust \
       --key ~/.saorsa-keys/communitas-v2.pub \
       --name COMMUNITAS_RELEASE_KEY \
       > communitas-dioxus/src/release_signing_key.rs
   ```
3. Ship a Dioxus release **signed with the old key** that bakes in the new
   public key — this is the handover point. Users must run this build once
   to accept updates signed with the new key.
4. Update the GitHub secret `COMMUNITAS_ML_DSA_SECRET` to the base64 of the
   new `*.secret` file.
5. Revoke the old `*.secret` from your local keystore.

### Rotating `COMMUNITAS_ED25519_SECRET` (Swift)

1. `generate_keys` from Sparkle's tool suite.
2. Copy the new public key into `communitas-apple/Resources/Info.plist`
   under `SUPublicEDKey`.
3. Ship a Swift release signed with the **old** private key that embeds the
   new public key. Users must run this release to accept updates signed
   with the rotated key.
4. Update GitHub secret `COMMUNITAS_ED25519_SECRET` to the new private key.
5. Keep the old Ed25519 public key archived — Sparkle can be configured to
   accept both during a transition window if needed.

## Local verification (release QA)

Before tagging, a maintainer should run:

```
just check          # lint + tests
just parity         # Dioxus ↔ Swift client-method parity
scripts/release-dryrun.sh vX.Y.Z   # see x0x/scripts/release-dryrun.sh
```

The dry-run script exercises the full signing + verify cycle against
ephemeral test keys so a misconfigured CI secret fails the check locally.

## Failure modes & recovery

- **Corrupt `update.json.sig`** — the Dioxus updater prints the verification
  error in Settings; users remain on the current version. Re-sign and
  re-upload.
- **Wrong `SUPublicEDKey` in bundle** — Sparkle silently ignores the update.
  Ship a patch release with the correct key.
- **Signature passes but bundle hash fails** — usually indicates a
  middlebox is rewriting the download. The DMG install will abort. Publish
  via the GitHub Releases CDN, not a cache in front.

## Contact

Release ceremony owners: `david@saorsalabs.com`, `@saorsa-labs/security`.
