# macOS Code Signing & Notarization Guide

This guide covers everything needed to sign and notarize Wandel's macOS `.app` bundle, sidecar binaries (`ffmpeg`, `ffprobe`), and `.dmg` installer both locally (via `.env`) and automatically in GitHub Actions (`.github/workflows/release.yml`).

---

## 1. Prerequisites

1. An active **Apple Developer Program** membership.
2. A **Developer ID Application** certificate (for distribution outside the Mac App Store) exported as a `.p12` file.
3. An **App-Specific Password** generated at [appleid.apple.com](https://appleid.apple.com/) (Sign-In and Security -> App-Specific Passwords).
4. Your 10-character **Apple Team ID** (visible in [developer.apple.com/account](https://developer.apple.com/account) under Membership Details, and in parentheses at the end of your certificate name).

---

## 2. Export a `.p12` Certificate (If You Only Have It in Keychain)

1. Open **Keychain Access** on macOS.
2. Select **login** (left sidebar) -> **My Certificates**.
3. Locate `Developer ID Application: Your Name (TEAMID)` and expand it to confirm its private key is attached.
4. Right-click the certificate -> **Export "Developer ID Application: ..."** -> save as `certificate.p12`.
5. Set a strong export password. You will use this as `APPLE_CERTIFICATE_PASSWORD`.

---

## 3. Generate the Required Environment Variables

### `APPLE_CERTIFICATE` (Single-Line Base64 of `.p12`)
Run this in Terminal to copy the single-line base64 string directly to your clipboard:
```bash
base64 -i /path/to/certificate.p12 | tr -d '\n' | pbcopy
```

### `APPLE_SIGNING_IDENTITY`
List all installed code signing identities in your Keychain:
```bash
security find-identity -v -p codesigning
```
Look for the entry starting with `Developer ID Application:` and copy the full name, for example:
```text
Developer ID Application: Your Name (XXXXXXXXXX)
```

If your `.p12` is not imported into Keychain yet, inspect its subject/identity directly with OpenSSL:
```bash
openssl pkcs12 -in /path/to/certificate.p12 -nokeys -legacy 2>/dev/null | openssl x509 -noout -subject -dates
```
(Remove `-legacy` if your `.p12` was exported with modern OpenSSL 3 / macOS Sequoia encryption.)

---

## 4. Local Signed & Notarized `.dmg` Build

1. Copy `.env.example` to `.env` (already gitignored in `.gitignore`):
   ```bash
   cp .env.example .env
   ```
2. Fill in all 6 variables in `.env`:
   ```dotenv
   APPLE_CERTIFICATE=MIIK...single_line_base64...==
   APPLE_CERTIFICATE_PASSWORD=your_p12_password
   APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (XXXXXXXXXX)"
   APPLE_ID=you@example.com
   APPLE_PASSWORD=xxxx-xxxx-xxxx-xxxx
   APPLE_TEAM_ID=XXXXXXXXXX
   ```
3. Run the signed build script:
   ```bash
   npm run build:mac:signed
   ```
   Tauri v2 (`src-tauri/tauri.conf.json` with `bundle.macOS.hardenedRuntime: true`) automatically creates a temporary keychain, imports your `.p12`, signs `Wandel.app` + `ffmpeg`/`ffprobe` sidecars + the `.dmg`, submits to Apple Notary Service, and staples the notarization ticket.

---

## 5. Automated CI Signing (GitHub Actions)

`.github/workflows/release.yml` is already wired to pass all 6 secrets into both tag releases (`v*`) and manual `workflow_dispatch` builds.

Add these 6 secrets in **GitHub Repository -> Settings -> Secrets and variables -> Actions -> New repository secret**:

| Secret Name | Value |
| :--- | :--- |
| `APPLE_CERTIFICATE` | Single-line base64 output of `base64 -i certificate.p12 \| tr -d '\n'` |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting `certificate.p12` |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (XXXXXXXXXX)` (no outer quotes) |
| `APPLE_ID` | Your Apple Developer account email |
| `APPLE_PASSWORD` | App-specific password from `appleid.apple.com` |
| `APPLE_TEAM_ID` | 10-character Apple Team ID |

Once configured, pushing a version tag (for example `git tag v3.5.0 && git push origin v3.5.0`) builds, signs, notarizes, and staples both `aarch64` (Apple Silicon) and `x86_64` (Intel) `.dmg` installers automatically.

---

## 6. Useful Verification & Troubleshooting Commands

### Verify Signature on `.app` and `.dmg`
```bash
# Deep-verify the signed .app bundle (including sidecars)
codesign --verify --deep --strict --verbose=2 target/release/bundle/macos/Wandel.app

# Inspect signing details and Hardened Runtime flags
codesign -dvv target/release/bundle/macos/Wandel.app

# Verify the .dmg signature
codesign --verify --verbose=2 target/release/bundle/dmg/Wandel_*.dmg
```

### Verify Gatekeeper Acceptance & Stapled Notarization Ticket
```bash
# Check Gatekeeper assessment (must report "accepted")
spctl --assess --type execute --verbose=4 target/release/bundle/macos/Wandel.app
spctl --assess --type open --context context:primary-signature --verbose=4 target/release/bundle/dmg/Wandel_*.dmg

# Validate that the notarization ticket is stapled offline
xcrun stapler validate target/release/bundle/macos/Wandel.app
xcrun stapler validate target/release/bundle/dmg/Wandel_*.dmg
```

### Inspect Apple Notarization History & Error Logs
```bash
# List recent notarization submissions
xcrun notarytool history \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID"

# Fetch detailed JSON failure log for a specific submission ID
xcrun notarytool log <SUBMISSION_ID> \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID"
```

### Manually Staple an Already-Notarized `.dmg`
```bash
xcrun stapler staple target/release/bundle/dmg/Wandel_*.dmg
```

### Verify `.p12` Validity & Expiration Date
```bash
# Test that the .p12 password is correct and inspect expiration
openssl pkcs12 -in /path/to/certificate.p12 -nokeys -passin pass:YOUR_PASSWORD | openssl x509 -noout -subject -issuer -dates
```

### Common Issues
- **`errSecInternalComponent` or Keychain locked in CI:** Tauri v2 handles temporary keychain creation automatically when `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` are both set. Ensure `APPLE_CERTIFICATE` has no line breaks (`tr -d '\n'`).
- **Notarization rejected for unsigned sidecars:** Ensure `ffmpeg` and `ffprobe` are listed in `bundle.externalBin` in `src-tauri/tauri.conf.json` (already configured) so Tauri signs them with Hardened Runtime before packaging.
- **Unsigned local test build blocked by Gatekeeper:** On a local unsigned dev build, clear quarantine with `xattr -cr /Applications/Wandel.app`.
