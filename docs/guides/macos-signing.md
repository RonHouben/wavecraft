# macOS Code Signing and Notarization Guide

This guide explains how to set up code signing and notarization for VstKit plugins on macOS.

## Prerequisites

- macOS development machine
- [Apple Developer Program](https://developer.apple.com/programs/) membership ($99/year)
- Xcode Command Line Tools installed (`xcode-select --install`)

---

## 1. Code Signing Setup

### 1.1 Create Developer ID Certificate

1. Log in to [Apple Developer](https://developer.apple.com/account)
2. Navigate to **Certificates, Identifiers & Profiles**
3. Create a new **Developer ID Application** certificate
4. Download and install the certificate (double-click the `.cer` file)

### 1.2 Find Your Signing Identity

Run this command to list available signing identities:

```bash
security find-identity -v -p codesigning
```

Output example:
```
1) ABC123DEF456... "Developer ID Application: Your Name (TEAM_ID)"
```

Copy the full string in quotes (including the team ID).

### 1.3 Set Environment Variable

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
```

Add this to your shell profile (`~/.zshrc` or `~/.bash_profile`) for persistence.

---

## 2. Notarization Setup

### 2.1 Create App-Specific Password

1. Go to [appleid.apple.com](https://appleid.apple.com/)
2. Sign in with your Apple ID
3. Navigate to **Sign-In and Security** → **App-Specific Passwords**
4. Generate a new password for "VstKit Notarization"
5. Copy the generated password

### 2.2 Store Credentials in Keychain (Recommended)

```bash
xcrun notarytool store-credentials "AC_PASSWORD" \
  --apple-id "your-apple-id@example.com" \
  --team-id "YOUR_TEAM_ID" \
  --password "your-app-specific-password"
```

This stores credentials securely in your macOS keychain.

### 2.3 Set Environment Variables

If you prefer environment variables over keychain:

```bash
export APPLE_ID="your-apple-id@example.com"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
export APPLE_APP_PASSWORD="your-app-specific-password"
```

Or use the keychain reference:

```bash
export APPLE_ID="your-apple-id@example.com"
export APPLE_TEAM_ID="YOUR_TEAM_ID"
export APPLE_APP_PASSWORD="@keychain:AC_PASSWORD"
```

---

## 3. Local Development Workflow

### 3.1 Build Without Signing (Fast Iteration)

```bash
cd engine
cargo xtask bundle
```

### 3.2 Ad-Hoc Sign for Local Testing

No Apple Developer account needed:

```bash
cargo xtask sign --adhoc
```

This allows the plugin to load locally but won't pass Gatekeeper on other machines.

### 3.3 Full Signed Build

Requires Developer ID certificate:

```bash
cargo xtask sign
```

Verify the signature:

```bash
codesign --verify --deep --strict target/bundled/vstkit.vst3
```

### 3.4 Notarization (Two-Step Workflow)

**Step 1: Submit**

```bash
cargo xtask notarize --submit
```

This uploads your plugins to Apple's notarization service and saves the request ID.

**Step 2: Check Status**

```bash
cargo xtask notarize --status
```

Wait until status is "Accepted" (typically 5-30 minutes).

**Step 3: Staple Ticket**

```bash
cargo xtask notarize --staple
```

This attaches the notarization ticket to your bundles.

**Alternative: Blocking Workflow**

```bash
cargo xtask notarize --full
```

This submits, waits (up to 30 minutes), and staples automatically.

### 3.5 Complete Release Build

One command for everything:

```bash
cargo xtask release
```

This runs: bundle → sign → notarize (full) → staple

---

## 4. CI/CD Setup (GitHub Actions)

### 4.1 Required Secrets

Add these to your GitHub repository secrets (**Settings** → **Secrets and variables** → **Actions**):

| Secret | How to Get It |
|--------|---------------|
| `APPLE_CERTIFICATE_P12` | Export certificate as P12, encode with `base64 -i certificate.p12 -o -` |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting P12 |
| `APPLE_SIGNING_IDENTITY` | Full identity string from `security find-identity` |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_TEAM_ID` | 10-character team ID (found in Apple Developer portal) |
| `APPLE_APP_PASSWORD` | App-specific password created earlier |

### 4.2 Export Certificate as P12

```bash
# Find certificate name
security find-identity -v -p codesigning

# Export to P12 (you'll be prompted for a password)
security export -k ~/Library/Keychains/login.keychain-db \
  -t identities \
  -f pkcs12 \
  -o certificate.p12 \
  -P "your-p12-password"

# Encode for GitHub secret
base64 -i certificate.p12 -o certificate.p12.txt
```

Copy the contents of `certificate.p12.txt` to the `APPLE_CERTIFICATE_P12` secret.

### 4.3 Trigger Release Build

Push a tag to trigger the workflow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Or manually trigger via GitHub Actions UI.

---

## 5. Troubleshooting

### 5.1 Common Errors

#### "No identity found"

```bash
# List available identities
security find-identity -v -p codesigning

# If empty, install your certificate
# Download from Apple Developer and double-click to install
```

#### "The signature is invalid"

Cause: Bundle was modified after signing.

Solution: Sign again after all modifications.

#### "Notarization rejected"

Fetch detailed log:

```bash
cargo xtask notarize --status
```

Common causes:
- Unsigned nested code (sign with `--deep`)
- Missing hardened runtime (`--options runtime`)
- Missing entitlements for JIT

#### "Unable to verify timestamp"

Cause: Network issue or Apple service downtime.

Solution: Retry signing.

### 5.2 Verify Notarization

Check if a bundle is properly notarized:

```bash
spctl --assess --type install --verbose target/bundled/vstkit.vst3
```

Expected output:
```
target/bundled/vstkit.vst3: accepted
source=Notarized Developer ID
```

### 5.3 Reset macOS Plugin Cache

If changes don't appear in DAW:

```bash
# For AU plugins
killall -9 AudioComponentRegistrar
auval -v aufx XXXX YYYY  # Replace with your codes

# For VST3/CLAP
# Restart the DAW
```

---

## 6. Entitlements Reference

### Production (`engine/signing/entitlements.plist`)

```xml
<key>com.apple.security.cs.allow-jit</key>
<true/>
```
**Required for:** WKWebView JavaScript JIT compilation

```xml
<key>com.apple.security.cs.allow-unsigned-executable-memory</key>
<true/>
```
**Required for:** WebKit internal memory management

```xml
<key>com.apple.security.cs.disable-library-validation</key>
<true/>
```
**Required for:** AU wrapper loading CLAP binary

### Development (`engine/signing/entitlements-debug.plist`)

Includes all production entitlements plus:

```xml
<key>com.apple.security.cs.debugger</key>
<true/>
```
**Allows:** LLDB debugger attachment

```xml
<key>com.apple.security.get-task-allow</key>
<true/>
```
**Allows:** Instruments profiling

---

## 7. Cost Breakdown

| Item | Cost | Frequency |
|------|------|-----------|
| Apple Developer Program | $99 | Annual |
| Code Signing | Included | - |
| Notarization | Included | - |

---

## 8. Additional Resources

- [Apple Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/)
- [Notarization Documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Hardened Runtime Entitlements](https://developer.apple.com/documentation/security/hardened_runtime)
- [VST3 Plugin Format](https://steinbergmedia.github.io/vst3_dev_portal/pages/What+is+the+VST+3+SDK/Index.html)

---

## Need Help?

If you encounter issues not covered here, please:

1. Check the [troubleshooting section](#5-troubleshooting)
2. Review logs with `--verbose` flag
3. Consult Apple's [notarization logs](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution/resolving_common_notarization_issues)
