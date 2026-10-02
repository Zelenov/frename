# Publishing frename in the Microsoft Store (owner guide)

Follow this once to put frename in the Microsoft Store. The why is in
[`docs/design/microsoft-store.md`](design/microsoft-store.md). CI already builds and tests the Store
package (`ci-windows-store` on every PR, `build-store` / `test-store` in every release); what is
left needs a person: an account, an identity check, questionnaires, and pressing **Submit**.

Nothing costs money: developer registration is free for individuals. Total hands-on time is about
2 hours (3 with the optional steps 5 and 8), plus waiting for the identity check and for certification (up to 3 business days).

The Store package is separate from the GitHub installer, which does not change. Code signing
(`docs/signing-setup.md`) is not needed for the Store: the Store signs the package itself.

## Handing work back to the agent

Issue #51 keeps the label `hold` while you work through these steps, so the nightly agent leaves it
alone. Whenever a step below says **hand it back**:

1. comment on #51 with what happened (a rejection report, a changed policy link, "the name is
   taken, I reserved X", "it's live"), then
2. remove the label `hold` from #51.

The next nightly run then picks #51 up, does what the comment asks (a fix, a new package, a
release, the README and `version.md` once it is live) in a new PR, and puts `hold` back when it
needs you again. To have it done sooner, start an agent session and point it at your comment.

## 1. Create the developer account (15 min, then wait for the check)

1. Open <https://storedeveloper.microsoft.com> and choose **Get started** / **Individual
   developer**. Start there, not from Partner Center: other entry points still show the old paid
   sign-up.
2. Sign in with your **personal** Microsoft account (outlook.com, hotmail, or any address
   registered as a Microsoft account). A work or school account is not accepted for individuals.
3. Verify your identity when asked: a government-issued ID and a selfie, taken with your phone
   (the page shows a QR code), in good light, with the original document. Your profile is filled
   in from the ID; the name on it is the publisher name the Store shows under frename.
4. When it is accepted you land in Partner Center (<https://partner.microsoft.com/dashboard>).
   The **Apps and games** tile can take about 5 minutes to appear.

Individual, not company: frename is a free, open-source tool under your own name (Store policy
10.14 asks for a company account only for someone acting in their trade or profession). An
individual account cannot be converted to a company account later.

## 2. Reserve the name "frename" (5 min)

1. Partner Center → **Apps and games** → **+ New product** → **MSIX or PWA app**.
2. Type `frename` → **Check availability** → **Reserve product name**.
   If the name is taken, reserve `frename video tagger` (or another) and **hand it back** with
   the name, so the manifest's display name is changed to match. Then do step 3 and wait: step 4
   needs a release made after the agent's change (otherwise Partner Center rejects the package in
   step 6.4 because its name is not the reserved one).
3. Open the new product → **Product management** → **Product identity**. Keep this page open for
   step 3; it shows:
   - **Package/Identity/Name**, e.g. `12345EugeneZelenov.frename`
   - **Package/Identity/Publisher**, e.g. `CN=1A2B3C4D-…`
   - **Package/Properties/PublisherDisplayName**, e.g. `Eugene Zelenov`
   - **Store ID**, e.g. `9NXXXXXXXXXX` (needed only for step 8)

## 3. Give CI the package identity (5 min)

GitHub → repository **Zelenov/frename** → **Settings** → **Secrets and variables** → **Actions** →
**Variables** tab → **New repository variable**. Add these three, copying each value exactly
(case, spaces and dots must match; they are public anyway, so they are variables, not secrets):

| Variable name | Value (from Product identity) |
|---|---|
| `STORE_IDENTITY_NAME` | Package/Identity/Name |
| `STORE_PUBLISHER` | Package/Identity/Publisher (the whole `CN=…` string) |
| `STORE_PUBLISHER_DISPLAY_NAME` | Package/Properties/PublisherDisplayName |

Until they are set, CI builds the package with a placeholder identity, which installs for testing
but which the Store rejects.

## 4. Get a package to upload (10 min)

The first upload should be a released version, so its version number is right (`0.74` becomes
package version `0.74.0.0`; the Store needs every new upload to have a higher version).

1. After step 3, wait for the next release (a `version.md` change merged to `main`), or run the
   release workflow by hand: **Actions** → **Release** → **Run workflow** on `main` (it rebuilds
   only if that version's GitHub release does not exist yet; if it exists, wait for the next one,
   or **hand it back** asking for a release: the agent bumps `version.md`).
2. Open that run → the `test-store` job must be green (it installed the package on a clean
   Windows and played the test clips). Its summary then says one of:
   - Windows App Certification Kit: **PASS** (or **WARNING**) → go on;
   - the kit was not available on the runner, skipped → go on: Partner Center validates the
     package when you upload it (step 6.4); if that validation fails, **hand it back** with its
     message.
3. At the bottom of the run's page, under **Artifacts**, download
   `frename-store-msix-vX.Y.0` and unzip it: `frename-X.Y.0.0-x64.msix`.
   Artifacts are kept for 90 days.

## 5. Try the package on Windows (30 min, optional)

CI already did this on a clean Windows; skipping this step is fine, step 9 checks the Store's own
install. To see it yourself, use **Windows Sandbox** (Start → "Turn Windows features on or off" →
tick **Windows Sandbox**, restart; Windows Pro or Enterprise): it has no GStreamer, no frename, and
everything in it is gone when you close it. Inside the sandbox, install the
[Windows SDK](https://developer.microsoft.com/windows/downloads/windows-sdk/) (for `signtool`),
copy in the repository folder and the `.msix`, and in an **administrator** Windows PowerShell
(Start → "Windows PowerShell" → Run as administrator; not PowerShell 7) in the
repository folder run (the first line allows scripts in this window only):

```powershell
Set-ExecutionPolicy -Scope Process Bypass
packaging/windows/test-msix.ps1 -Msix <path>\frename-X.Y.0.0-x64.msix -Report wack.xml
```

On your own machine the script refuses to run while GStreamer is installed (it proves a clean
install). It removes its test certificate and the package when it ends. Without Visual Studio in
the sandbox it skips its check of the package's DLLs (CI does that one).

## 6. Fill in the first submission (60 min)

Partner Center → frename → **Start your submission**. The texts are ready in
[`packaging/store/`](../packaging/store/):

1. **Pricing and availability**: Markets: all. Visibility: **Public audience**, discoverable.
   Pricing: **Free**. Leave the rest at its defaults.
2. **Properties**:
   - Category: **Photo & video**.
   - Privacy policy URL: `https://github.com/Zelenov/frename/blob/main/packaging/store/privacy-policy.md`
     (answer **Yes** to "Does this product access, collect, or transmit personal information?":
     frames and audio of clips the user picks go to Anthropic or Soniox).
   - Website: `https://github.com/Zelenov/frename`
   - Support contact: `https://github.com/Zelenov/frename/issues`
   - System requirements: as in `packaging/store/listing.md` → "System requirements".
   - Product declarations: tick that the product **uses generative AI / shows AI-generated
     content** if Partner Center offers that box (Store policy 11.16), leave the others unticked.
3. **Age ratings**: take the IARC questionnaire. Category **Utility / productivity** (not a game,
   not a social or communication app). Answer **No** to violence, sexual content, gambling,
   in-app purchases, sharing location, and to "users can interact or exchange content with other
   users"; answer **No** to unrestricted internet access for browsing. Expected result: 3+ / E /
   PEGI 3. (10 min)
4. **Packages**: drag in the `.msix` from step 4. Partner Center validates it; errors about the
   identity or the display name mean a variable in step 3, or the reserved name, does not
   match. Device families: **Windows 10/11 Desktop**
   only.
5. **Store listings** → **Add/remove languages**: keep **English (United States)** →
   open it and paste every field from `packaging/store/listing.md`. Screenshots: upload the four
   PNG files in `packaging/store/screenshots/` with the captions in
   `packaging/store/screenshots.md`.
6. **Submission options**:
   - Publishing hold: **Publish this submission as soon as it passes certification** (or
     "manually" if you want to press the button yourself).
   - **Restricted capabilities** → `runFullTrust` asks for a reason. Paste:

     > frename is a Win32 desktop app (Rust) packaged with the Desktop Bridge. It plays video files
     > from folders the user opens and renames them, writes comments and markers into them, and
     > saves frames and subtitle files next to them, using standard Win32 file APIs. It bundles the
     > GStreamer media framework, whose plugin scanner is a helper process inside the package.
     > This needs full trust; it declares no other restricted capability.

   - **Notes for certification**. Paste:

     > No account or sign-in is needed. Open any folder with video files (the folder button at the
     > bottom, or drag a folder onto the window); the first clip plays. Click a tag, then press
     > PageDown: the clip you left is renamed with the tag. Two optional batch actions (Describe
     > with AI, Generate subtitles) call Anthropic's and Soniox's APIs with the user's own API key,
     > entered in Settings; without a key they explain that a key is needed, and everything else
     > works. Updates come from the Store: the app has no updater of its own.

7. Before pressing **Submit**, open the current Store Policies
   (<https://learn.microsoft.com/windows/apps/publish/store-policies>) and look at the version at
   the top. The design notes quote **7.19**; a 7.20 exists (published 2026-09-15), which the agent
   could not read. If the page shows 7.20 or newer, **hand it back** with the link and wait for the
   agent to compare sections 10.2, 10.5, 10.14 and 11.16 (a few lines of reading, usually the same
   day); otherwise go on.
8. **Submit to the Store**.

## 7. Certification (wait up to 3 business days)

Partner Center shows the progress. The listing goes live about 15 minutes after it passes.

If it is rejected, the report lists the failed policies with notes. **Hand it back** with the
report pasted into the comment: the agent fixes the package or the texts, and CI builds a new
package; you upload it as a new submission (step 6.4 only: everything else is kept).

## 8. Optional: automatic submissions from each release (30 min, after the first one is live)

The Store's submission API works only for a product that is already live, so do this after step 7.
Once set up, every release uploads its tested package as a new submission; the listing texts stay
as they are in Partner Center.

1. **Link an Entra ID tenant**: Partner Center → ⚙ **Account settings** → **Tenants** →
   **Associate** (use your Microsoft account's tenant) or **Create new Microsoft Entra tenant**
   (free). If your individual account does not offer this page, stop here: releases keep working
   and you upload each new package by hand (step 6.4).
2. **Add an app for GitHub**: Account settings → **User management** → **Microsoft Entra
   applications** → **Create Microsoft Entra application** → name it `frename-github-release` →
   role **Manager** → **Save**.
3. Open that application → **Add new key**. Copy the **Client ID** and the **Key** (shown only
   once; this is the client secret; it expires, the page shows when). Note the **Tenant ID** on
   the same page.
4. Account settings → **Legal info** (or **Account settings** overview) → copy the **Seller ID**.
5. GitHub → Settings → Secrets and variables → Actions → **Secrets** tab → New repository
   secret, four times:

   | Secret name | Value |
   |---|---|
   | `MS_STORE_TENANT_ID` | Tenant ID (step 3) |
   | `MS_STORE_CLIENT_ID` | Client ID (step 3) |
   | `MS_STORE_CLIENT_SECRET` | the Key (step 3) |
   | `MS_STORE_SELLER_ID` | Seller ID (step 4) |

6. Last, **Variables** tab → New repository variable `MS_STORE_PRODUCT_ID` = the **Store ID**
   (step 2.3, e.g. `9NXXXXXXXXXX`). This switches the `submit-store` job on; without it, releases
   never touch the Store. With it set but a secret missing, `submit-store` fails with the missing
   secret's name (the GitHub release is already published by then).
7. When the key expires, repeat step 3 and update `MS_STORE_CLIENT_SECRET`.

## 9. Check the Store version by hand (20 min, once it is live)

On a Windows 10/11 machine, ideally one where frename was never installed:

1. Install frename from its Store page (the link is on Product identity: "View in the Store").
   It installs without any warning.
2. Start it. If the GitHub-installed frename was on this machine, its recent folders and Settings
   are there (copied once; the two versions are independent afterwards). Open a folder of
   videos: a clip plays (MP4 and MOV at least).
3. Tag a clip and press PageDown: the file is renamed in that folder.
4. Settings → Updates says **Updates come from the Microsoft Store**, with no Check button.
   Check a few clips, click the batch button, and if a batch action shows **Open log**, click it:
   the log opens.
5. Settings → enter an Anthropic API key, run **Describe with AI** on one short clip; restart
   frename: the key is still saved. Open Windows **Credential Manager** → **Windows
   Credentials**: an entry with `frename` in its name exists. (This is the one thing the agent
   could not verify: that the password store works from a Store app.)
6. Uninstall from Start → right-click frename → **Uninstall**. The key entry remains in
   Credential Manager (remove it there, or in Settings before uninstalling).

If anything fails, **hand it back** with what you saw (if it is the key in Credential Manager,
the agent also corrects the privacy policy). Once the Store version works, **hand it
back** with "it's live" and the Store link: the agent then updates README.md (install from the
Store) and `version.md`, as the issue asks, and closes #51 with that PR.
