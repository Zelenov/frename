# Publishing frename in the Mac App Store: owner guide

Everything that needs a person, in order. The agent built the rest: the sandboxed Store variant
(`cargo build --features store`), its packaging (`packaging/macos-store/`), the listing texts and
screenshots (`packaging/macos-store/listing.md`), and the workflow that signs, validates and
uploads it (`.github/workflows/mac-app-store.yml`). Why it is built this way:
`docs/design/mac-app-store.md`.

Times are rough; Apple's own waits (enrollment, review) vary.

| # | Step | Where | Time |
|---|---|---|---|
| 1 | Enroll in the Apple Developer Program | developer.apple.com/programs/enroll | 20 min + Apple's check, 1–2 days |
| 2 | Register the bundle id | developer.apple.com → Identifiers | 5 min |
| 3 | Make the two certificates | a Mac: Keychain Access + developer.apple.com → Certificates | 20 min |
| 4 | Make the provisioning profile | developer.apple.com → Profiles | 5 min |
| 5 | Create the app in App Store Connect | appstoreconnect.apple.com → Apps | 10 min |
| 6 | Make an App Store Connect API key | appstoreconnect.apple.com → Users and Access → Integrations | 5 min |
| 7 | Add the GitHub secrets | github.com/Zelenov/frename → Settings → Secrets and variables → Actions | 10 min |
| 8 | First build: run the workflow, TestFlight | GitHub Actions; TestFlight app on a Mac | 30 min + processing ~30 min |
| 9 | Fill in the listing, privacy and export answers | App Store Connect → frename | 45 min |
| 10 | Submit for review | App Store Connect | 5 min + review, usually 1–3 days |
| 11 | Every later version | GitHub Actions → App Store Connect | 15 min per release |

## 1. Apple Developer Program

- Go to <https://developer.apple.com/programs/enroll/>, sign in with the Apple Account you want to
  publish under (two-factor authentication must be on), enroll as an **Individual** (your legal
  name is shown as the seller) or an **Organization** (needs a D-U-N-S number and a website).
- Cost: **99 USD a year** (or the local price), renewed yearly; the app leaves the Store if the
  membership lapses.
- Apple checks the enrollment; usually within a day or two. You get an e-mail.
- Afterwards, note your **Team ID**: developer.apple.com → Account → Membership details
  (10 characters, e.g. `A1B2C3D4E5`).

## 2. The bundle id

developer.apple.com → Certificates, IDs & Profiles → **Identifiers** → **+** → App IDs → App:

- Description: `frename`
- Bundle ID: **Explicit**, `io.github.zelenov.frename` (the id every macOS build of frename uses:
  `packaging/macos/build-app.sh`, `packaging/macos-store/build-store.sh`).
- Capabilities: none needed (the App Sandbox is not a capability here; it comes from the
  entitlements). Continue → Register.

## 3. Certificates (on a Mac)

Two certificates, both for Mac App Store distribution:

1. On a Mac, open **Keychain Access** → menu Keychain Access → Certificate Assistant → **Request a
   Certificate From a Certificate Authority…** → your e-mail, "Saved to disk" → save the
   `.certSigningRequest` file.
2. developer.apple.com → **Certificates** → **+** → **Apple Distribution** → upload the request
   → download `distribution.cer` → double-click it (it goes into your login keychain).
3. Again **+** → **Mac Installer Distribution** → the same request file → download → double-click.
4. In Keychain Access → login → My Certificates, select both ("Apple Distribution: …" and
   "3rd Party Mac Developer Installer: …" or "Mac Installer Distribution: …"), right-click →
   **Export 2 items…** → format `.p12` → choose a password. This file and its password become two
   secrets in step 7. Keep the `.p12` somewhere safe (a password manager), then delete it from
   the Desktop.

No Mac at hand? A certificate request can also be made with OpenSSL; ask the agent for the
commands. The Mac is needed anyway for TestFlight (step 8).

## 4. Provisioning profile

developer.apple.com → **Profiles** → **+** → Distribution: **Mac App Store Connect** → App ID
`io.github.zelenov.frename` → the Apple Distribution certificate from step 3 → name it
`frename Mac App Store` → Generate → **Download** (`frename_Mac_App_Store.provisionprofile`).
It is valid for as long as the certificate (a year); renew both together.

## 5. The app in App Store Connect

<https://appstoreconnect.apple.com> → **Apps** → **+** → New App:

- Platforms: **macOS**
- Name: `frename` (must be unique in the Store; if taken, e.g. `frename – tag video clips`)
- Primary language: English (U.S.)
- Bundle ID: `io.github.zelenov.frename`
- SKU: `frename-mac`
- User access: Full Access

Then open the app → **App Information** → note the **Apple ID** (a number, e.g. `6700000000`):
it is the `MAS_APPLE_APP_ID` secret.

## 6. App Store Connect API key (for uploads from GitHub)

App Store Connect → **Users and Access** → **Integrations** → **App Store Connect API** → Team
Keys → **+** → name `GitHub frename`, access **App Manager** → Generate.

- Download the key file `AuthKey_XXXXXXXXXX.p8` (only once possible).
- Note the **Key ID** (in the table) and the **Issuer ID** (above the table).

## 7. GitHub secrets

github.com/Zelenov/frename → Settings → Secrets and variables → **Actions** → New repository
secret, one per row. On a Mac, `base64 -i <file> | pbcopy` copies a file as base64.

| Secret | Value |
|---|---|
| `MAS_CERTIFICATES_P12` | the `.p12` from step 3, as base64 |
| `MAS_CERTIFICATES_PASSWORD` | its password |
| `MAS_PROVISIONING_PROFILE` | the `.provisionprofile` from step 4, as base64 |
| `MAS_TEAM_ID` | the Team ID from step 1 |
| `MAS_APPLE_APP_ID` | the app's Apple ID from step 5 (the number) |
| `APP_STORE_CONNECT_API_KEY_ID` | the Key ID from step 6 |
| `APP_STORE_CONNECT_API_ISSUER_ID` | the Issuer ID from step 6 |
| `APP_STORE_CONNECT_API_KEY_P8` | the whole text of `AuthKey_….p8` (open it in TextEdit, copy all) |

Only the workflow `Mac App Store`, started by hand, reads them. Pull request CI never does.

Also add the privacy policy's Mac paragraph: `packaging/store/privacy-policy.md` (from #51) covers
Windows and Linux. Before submitting, add to its "What stays on your computer" list: "On a Mac
(App Store version) they are kept in the app's container,
`~/Library/Containers/io.github.zelenov.frename/Data/Library/Application Support/frename`; the Mac
download from GitHub keeps them in `~/Library/Application Support/frename`." And to "Updates":
"**Mac App Store app:** the App Store installs and updates frename. frename itself makes no update
requests." The agent can make this change once #51 is merged.

## 8. First build and TestFlight

1. github.com/Zelenov/frename → **Actions** → **Mac App Store** → **Run workflow** → branch
   `main`, tick **Upload the build to App Store Connect** → Run.
2. The run (about 20 minutes) builds `frename.pkg`, signs it, **validates** it with App Store
   Connect, uploads it, and keeps it as an artifact. Its summary says "signed for the App Store".
   A red "Validate" step prints Apple's reason; send the run link to the agent.
3. App Store Connect → frename → **TestFlight**: the build appears after processing (15–60 min).
   Answer the export question if asked: frename uses only standard encryption for HTTPS
   ("None of the algorithms mentioned above" / exempt); the build already declares
   `ITSAppUsesNonExemptEncryption = NO`.
4. TestFlight → Internal Testing → **+** → add yourself. On a Mac with Apple silicon install
   **TestFlight** from the App Store, sign in with the same Apple Account, install frename.
5. Check on that Mac (what CI cannot):
   - 📂 → choose a folder of clips → a clip plays, with sound;
   - tag a clip and press Page Down: the file is renamed; press F2: a marker is saved (open the
     clip in Premiere Pro or check that the file's date changed);
   - quit frename (⌘Q) and start it again: **the same folder reopens without asking**;
   - right-click 📂 → choose one clip: frename asks once for its folder with the picker already in
     it; click Open;
   - Settings → paste an Anthropic key, run Describe with AI on one clip; quit and restart:
     the key is still there (Keychain);
   - Settings → Updates says "Updates come from the App Store".

## 9. Listing, privacy, export compliance

App Store Connect → frename, using `packaging/macos-store/listing.md` field by field:

- **App Information**: subtitle, categories, content rights, age rating (all "None" → 4+).
- **Pricing and Availability**: price, countries.
- **App Privacy** → privacy policy URL → **Get Started** → "No, we do not collect data from this
  app" → Publish (reasoning in `listing.md`).
- **macOS App → 1.0 Prepare for Submission**: promotional text, description, keywords, support and
  marketing URLs, copyright, the three screenshots from `packaging/macos-store/screenshots/`,
  App Review Information (notes in `listing.md`, your contact), **Build** → select the TestFlight
  build from step 8.
- **Version release**: "Manually release this version" lets you pick the day.

## 10. Submit

On the version page → **Add for Review** → **Submit to App Review**. Review usually takes one to
three days. If Apple rejects it, paste the rejection text into a comment on issue #85: the agent
fixes it and you run step 8 again (a new build number is automatic).

## 11. Later versions

After a GitHub release (the version in `version.md` is published):

1. Actions → **Mac App Store** → Run workflow on `main` with upload ticked (15–20 min).
2. App Store Connect → frename → **+ Version** (the new number, e.g. `0.78`) → What's New (the
   newest `version.md` block) → select the new build → Add for Review → Submit.

Once a year: renew the membership, then the certificates and profile (steps 3–4), and update the
`MAS_CERTIFICATES_P12`, `MAS_CERTIFICATES_PASSWORD` and `MAS_PROVISIONING_PROFILE` secrets.
