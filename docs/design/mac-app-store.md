# Mac App Store (issue #85)

frename in the Mac App Store, next to the free ad-hoc-signed download from #15
(`docs/design/macos.md`). Owner steps: `docs/mac-app-store-setup.md`.

## What the Store requires, and what it means for frename

| Requirement | Source | In frename |
|---|---|---|
| App Sandbox | [Guideline 2.4.5(i)](https://developer.apple.com/app-store/review/guidelines/) | `packaging/macos-store/entitlements.plist` |
| No self-update, updates only through the Store | 2.4.5(vii) | `--features store`: Velopack hooks and update checks off |
| Self-contained, no downloaded code | 2.4.5(ii), (iv) | GStreamer inside the bundle, as in #15 |
| Apple Distribution signature, Mac App Store Connect profile, `.pkg` signed with Mac Installer Distribution | [Electron's MAS guide](https://github.com/electron/electron/blob/main/docs/tutorial/mac-app-store-submission-guide.md) | `build-store.sh`, `mac-app-store.yml` |
| Entitlements `com.apple.application-identifier` and `com.apple.developer.team-identifier` matching the profile | [QA1884](https://developer.apple.com/library/archive/qa/qa1884/_index.html) | added by `build-store.sh` from `MAS_TEAM_ID` |
| `LSApplicationCategoryType` | [Apple](https://developer.apple.com/documentation/bundleresources/information-property-list/lsapplicationcategorytype) | `public.app-category.video` (#15's Info.plist) |
| Icon with 512 pt @2x (1024 px) | [ITMS-90236](https://github.com/electron-userland/electron-builder/issues/7720) | #15's `.icns` has every size |
| arm64 only: allowed when the minimum is macOS 12 | [Apple forums](https://developer.apple.com/forums/thread/810409) | `LSMinimumSystemVersion` 12.0 |
| A new `CFBundleVersion` for every upload | [Apple](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion) | the workflow's run number |
| Privacy policy URL; the privacy ("nutrition label") answers | [Manage app privacy](https://developer.apple.com/help/app-store-connect/manage-app-information/manage-app-privacy), 5.1.1(i) | the policy of #51; "Data Not Collected" (`listing.md`) |
| Privacy manifest | required-reason declarations only on iOS-family platforms, not macOS ([Apple](https://developer.apple.com/documentation/bundleresources/privacy-manifest-files)) | included anyway, `PrivacyInfo.xcprivacy`: no tracking, file timestamps 3B52.1/C617.1 |
| Export compliance | [ITSAppUsesNonExemptEncryption](https://developer.apple.com/documentation/bundleresources/information-property-list/itsappusesnonexemptencryption) | `NO`: HTTPS only; the owner confirms once |
| Screenshots 16:10: 1280×800, 1440×900, 2560×1600 or 2880×1800 | [Screenshot specifications](https://developer.apple.com/help/app-store-connect/reference/app-information/screenshot-specifications) | 2560×1600 in `packaging/macos-store/screenshots/` |

## The sandbox, feature by feature

- **Opening folders.** A folder chosen in an open panel, or dropped, is opened to the app "and
  recursively in nested folders", until the app quits
  ([Accessing files from the macOS App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)).
  frename's work (renaming clips, XMP inside them, `.comment.txt`, `.srt`, frame JPEGs, the
  folder's `.frename`) all happens inside the opened folder: covered by
  `com.apple.security.files.user-selected.read-write`.
- **Reopening the last folder after a restart.** Needs a security-scoped bookmark
  (`com.apple.security.files.bookmarks.app-scope`): `bookmarkData(options: .withSecurityScope)`
  while the app has access, and at the next start `URLByResolvingBookmarkData` with
  `.withSecurityScope` + `startAccessingSecurityScopedResource()` (same Apple page).
  `src/folder_access.rs` (the Objective-C calls) and `frename_core::folder_bookmarks` (the
  files): every folder frename scans gets a fresh bookmark in
  `<data folder>/folder-access/<FNV-1a of the path>.bookmark` (the 50 newest are kept; a fresh
  one each time replaces a stale one); before a scan the saved bookmark is resolved and started.
  Access is never stopped: the user may come back to the folder, and it ends with the process.
  Not in the database: no migration, and the bookmarks are useless outside this app and this Mac.
- **A clip opened alone** (right-click 📂 → a file, or a file dropped on the window): the sandbox
  opens that file only, not its folder, and frename always opens the whole folder. The Store build
  then asks for the folder: the folder picker opens inside it with the line "frename needs access
  to this folder to open it: click Open" (rfd puts a dialog's title into `NSOpenPanel`'s message
  on macOS, which the panel shows: `rfd` `backend/macos/file_dialog/panel_ffi.rs`), so one click
  grants it; the clip stays selected if that folder is chosen (`chosen_pair`). Cancelling leaves
  the current folder open. Only a refusal (`PermissionDenied`) asks: a missing folder (an unplugged
  drive) fails quietly as in every build, and nothing is asked at start-up — the last folder is
  reopened only if its bookmark still works.
- **Data folder** (database, log, GStreamer registry, bookmarks): the app's container; `HOME`
  points into it, so #15's `~/Library/Application Support/frename` lands there
  ([App Sandbox in depth](https://developer.apple.com/library/archive/documentation/Security/Conceptual/AppSandboxDesignGuide/AppSandboxInDepth/AppSandboxInDepth.html)).
- **GStreamer.** Plugins load from the bundle; the registry is written in the container. #15
  already scans plugins in-process (`GST_REGISTRY_FORK=no`), so no `gst-plugin-scanner` helper
  has to be signed with `com.apple.security.inherit`
  ([Embedding a helper tool in a sandboxed app](https://developer.apple.com/documentation/xcode/embedding-a-helper-tool-in-a-sandboxed-app)).
  Sound out needs no entitlement (only input does).
- **API keys.** frename keeps them with the `keyring` crate, whose macOS backend uses the
  file-based keychain (`SecKeychain` generic passwords). Items an app creates are its own; no
  entitlement is needed for that. TN3137 calls the file-based keychain "on the road to
  deprecation" but supported ([TN3137](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains)).
  CI checks it in the sandbox: the Store build's `--self-test` saves, reads and deletes a
  throwaway entry (`frename_core::ai::key::check_credential_store`).
- **Network** (Anthropic, Soniox): `com.apple.security.network.client`.
- **Opening the log or a web page** (Settings, the Anthropic billing link): through
  `NSWorkspace.openURL` on macOS instead of starting `/usr/bin/open`.
- **Updates.** `--features store` sets `package::STORE_BUILD`: Velopack's hooks never run, the
  update check never runs, and Settings → Updates shows the version and "Updates come from the
  App Store". Velopack stays linked (it is inert); the guideline is about behaviour.
- **Opening from Finder, Services, "Open With":** not in frename on macOS (#15), so nothing to
  sandbox.

## Build and CI

- `packaging/macos-store/build-store.sh` builds the app with #15's `build-app.sh` (same bundle,
  same GStreamer), adds the privacy manifest and the export key, and:
  - with the `MAS_*` signing inputs: embeds the profile, adds the team entitlements, signs with
    Apple Distribution and makes a `.pkg` signed with Mac Installer Distribution
    (`productbuild --component frename.app /Applications`);
  - without them: signs ad hoc **with the sandbox entitlements** (the app really runs sandboxed)
    and makes an unsigned `.pkg`, and says so.
- `ci.yml` → `ci-macos-store`: clippy and tests with `--features store`, the release build, the
  unsigned Store `.pkg`. `ci-macos-store-test` on a fresh runner (`test-store.sh`): installs the
  `.pkg` into `/Applications`, checks the bundle and the sandbox entitlement, proves the sandbox
  is on (the container appears; a clip outside it cannot be read), runs the self-test on clips
  copied into the container (decoding + Keychain), starts the app and takes a screenshot.
  A Store-signed app cannot run on a runner (it runs only once installed by the App Store or
  TestFlight), so this is as far as CI can test the sandbox.
- `mac-app-store.yml`: with the secrets, builds the signed `.pkg`, `xcrun altool --validate-app`,
  and with **upload** `xcrun altool --upload-package` with the App Store Connect API key.
  It runs with upload after every successful `Release` run on `main` (`workflow_run`, on the
  released commit) as a run of its own, so a Store problem never turns a release red or holds
  back the downloads; it can also be started by hand. The version is `version.md`'s (`0.77`,
  which the App Store Connect version record must match); the build number is the workflow's run
  number, which grows however the run started. `altool` remains the supported command-line upload for the App
  Store (only its notarization use ended, [TN3147 via fastlane](https://github.com/fastlane/fastlane/discussions/21347));
  notarization is not used for Store builds.

## What CI cannot prove (TestFlight checklist in the owner guide, step 8)

- A bookmark resolving after a real restart, and the folder picker granting a folder: both need
  a person clicking in an open panel.
- Renaming and writing XMP in a user-chosen folder inside the sandbox (CI's clips are in the
  container, where access is automatic).
- Sound.
- Apple's review.

## Decisions made without the owner

- **Same feature name as the Microsoft Store build (#51, PR #86): `store`**, with #86's
  `package::STORE_BUILD`, `run_velopack_hooks` early return and `UpdatesState::new(installed,
  from_store, …)` taken as written there. Each platform has one store, so one flag means "the
  store installs and updates this build". Merging with #86 conflicts only where the stores differ:
  the text of the Updates section (`updates-from-app-store` here, `updates-from-store` there; the
  merge picks one per platform with `cfg!(target_os = "macos")`), the doc comments, the log label,
  and #86's `store_data_dir`, which on macOS would move the Store build's data to
  `…/Application Support/frename-store` inside the container (harmless, still the container).
- **The privacy policy lives at #86's path** (`packaging/store/privacy-policy.md`, #86's text plus
  the Mac builds), so both stores link one page; whichever PR merges second takes the other's
  lines.
- **Bundle id `io.github.zelenov.frename`**, the one #15's download uses: one app identity.
  The two builds do not share data (the Store build lives in its container).
- **Bookmarks in files, not in the database**: no migration, and they belong to this machine.
- **Ask for a folder instead of refusing** when only a clip was granted: one extra click, and
  every way of opening a clip keeps working.
- **"Data Not Collected"** in the privacy answers: the optional AI actions send data from the
  user's Mac to the service the user chose, under the user's key; the developer gets nothing.
- **Build number = this workflow's run number**: always increasing, whether a release or a
  person started it, no file to keep.
- **Screenshots**: the README ones at 2560×1600 (the issue asks for these); real Mac captures
  from TestFlight are better later.
- **`ffmpeg` for subtitles of mkv/m2ts/avi** stays a `PATH` lookup: a sandboxed app cannot run a
  Homebrew `ffmpeg` and a Finder-started app does not see Homebrew's `PATH` anyway, so those
  files report that ffmpeg is needed, as on a Mac without it. The built-in decoder covers
  mp4/mov.
