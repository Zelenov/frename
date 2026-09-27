# frename in the Microsoft Store

Design notes for issue #51. Builds on the Velopack installer (#10, `installer.md`) and the
code-signing work (#50, `docs/research/windows-signing.md`, `docs/signing-setup.md`).
The owner's steps are in [`docs/store-setup.md`](../store-setup.md).

Sources were read on 2026-09-27. learn.microsoft.com could not be reached from the agent's
session, so Microsoft Learn pages were read from their public source repositories, which are the
same text:

- `WDD` = <https://github.com/MicrosoftDocs/windows-dev-docs/blob/docs/> (branch `docs`, last
  commit 2026-09-17), published under <https://learn.microsoft.com/windows/apps/>;
- `MSIX` = <https://github.com/MicrosoftDocs/msix-docs/blob/main/msix-src/> (last commit
  2026-08-24), published under <https://learn.microsoft.com/windows/msix/>.

The Store Policies read are **version 7.19** (`WDD hub/apps/publish/store-policies.md`). A 7.20
was published on 2026-09-15, effective 2026-10-22 (search result only); the sections cited here
are checked against the live page before the first submission (`store-setup.md` step 6).

## Recommendation

**Publish an MSIX package, built by CI from the same code with a `store` cargo feature, and let
the Store sign, host and update it.** Keep the GitHub installer (Velopack) exactly as it is.

Why not list the existing `frename-win-Setup.exe` in the Store instead:

| | MSIX (chosen) | Existing EXE installer |
|---|---|---|
| Signing | The Store re-signs the package with a Microsoft certificate; ours may be unsigned ¹ | Every PE file must be signed with a certificate from the Microsoft Trusted Root Program ² — the paid certificate of #50, which the owner may not be eligible for |
| Hosting | The Store hosts it | We host it on a versioned HTTPS URL whose file never changes ² |
| Updates | Windows checks the Store every 24 hours ³ | The app updates itself (Velopack) ³ |
| Trust | No SmartScreen warning: installed by the Store | SmartScreen reputation still builds up per certificate |
| Cost | Nothing | The certificate (about $120 a year, see #50) |
| Work | A package job and a build flag (this PR) | Signing first (#50), then a listing |

¹ `WDD hub/apps/publish/publish-your-app/msix/app-package-requirements.md`: "Your MSIX and AppX
packages don't have to be signed with a certificate rooted in a trusted certificate authority… The
Microsoft Store will automatically re-sign your MSIX/AppX packages".
² Policy 10.2.9 and `WDD hub/apps/publish/publish-your-app/msi/app-package-requirements.md`.
³ `WDD hub/apps/distribute-through-store/how-to-distribute-your-win32-app-through-microsoft-store.md`
(comparison table: "The OS will automatically check updates every 24 hours" for MSIX; "The
application is responsible for managing its own auto-updates" for MSI/EXE).

## Developer account

- **Individual account, free.** The $19 fee is waived for individuals since September 2025, in
  "nearly 200 markets", without a credit card (`WDD hub/apps/publish/whats-new-individual-developer.md`,
  <https://blogs.windows.com/windowsdeveloper/2025/09/10/free-developer-registration-for-individual-developers-on-microsoft-store/>).
  Company accounts are free too since May 2026 (`WDD hub/apps/publish/whats-new-company-developer.md`).
- **Identity:** sign up at <https://storedeveloper.microsoft.com> ("the only supported entry point
  for the new flow"; going through Partner Center directly shows the old paid flow) with a
  personal Microsoft account, then a government-issued ID and a selfie, taken on a phone. The
  profile is filled from the ID. Microsoft gives no processing time; access is "instant" once
  verified (`WDD hub/apps/publish/partner-center/open-a-developer-account.md`).
- **Individual, not company**: policy 10.14 requires a company account from "any person acting
  in relation to their trade or profession" or when the publisher name looks like a business.
  frename is a free open-source tool published under the owner's own name, so an individual
  account fits. An individual account cannot be turned into a company one later
  (`open-a-developer-account.md`).
- The publisher name shown in the Store is the name on the ID.

## Store rules that shape the package

**Full trust, and no extra file capability.** A packaged desktop app that runs at medium
integrity "needs to declare the runFullTrust restricted capability"; restricted capabilities are
justified in the submission form and "may add some additional time", once
(`WDD uwp/packaging/app-capability-declarations.md`,
`WDD hub/apps/publish/publish-your-app/msix/manage-submission-options.md`). A full-trust app
"is already running as the user" and may write outside its package wherever the user may
(`MSIX desktop/desktop-to-uwp-behind-the-scenes.md`), so opening, renaming and writing into any
folder works with the ordinary file APIs frename uses. `broadFileSystemAccess` only applies to
the UWP `Windows.Storage` APIs (`app-capability-declarations.md`): not declared.

**No self-update inside the package.** Policy 7.19 has no general rule against it outside games
(10.2.5 is for games), but 10.2.2 forbids dynamically changing the described functionality, and a
Velopack update could not work anyway: package files "are marked read-only, and are heavily locked
down… Windows prevents apps from launching if those files are tampered with"
(`desktop-to-uwp-behind-the-scenes.md`). The Store build therefore has no updater at all.

**Bundled GStreamer.** Nothing in the policies forbids LGPL or GPL components, and there is no
codec clause outside web browsers (10.2.1; the 10.8.7 open-source wording was removed in 7.18,
`WDD hub/apps/publish/store-policies-change-history.md`). The package carries the same bundle as
the installer (LGPL GStreamer and FFmpeg, no GPL parts, licenses and source links in
`licenses\`). Launching `gst-plugin-scanner.exe` from inside the package is allowed; the
certification kit's optional "Blocked executables" check may flag it, and its documentation says
"If the flagged file(s) is part of your application, you may ignore the warning"
(`WDD uwp/debug-test-perf/windows-desktop-bridge-app-tests.md`). Open point: the LGPL lets users
replace the libraries, and files in an installed MSIX cannot be replaced (discussed without an
answer from Microsoft in <https://github.com/Microsoft/FFmpegInterop/issues/95>); the portable zip
on GitHub, where every DLL can be swapped, and the source links cover that.

**Privacy policy: required.** 10.5.1: "Product types that inherently have access to Personal
Information must always have privacy policies. These include… Desktop Bridge and Win32
products." 10.5.2 asks for opt-in consent before personal information goes to a third party, and a
way to withdraw it: frename sends nothing until the user saves their own key and starts **Describe
with AI** or **Generate subtitles**, whose panel says what will be sent and what it costs;
deleting the key in Settings withdraws it. 10.5.4: sent securely (both services are HTTPS).
The policy is `packaging/store/privacy-policy.md`, served by GitHub at a stable URL.

**Generative AI (11.16).** A product that shows content generated by AI models on user input must
say so in its listing, declare it in Partner Center, and "provide a means for users to report
inappropriate content to the developer". The listing says so; reporting goes to the GitHub issues,
linked from the listing and the privacy policy. The descriptions are written into the user's own
files, not shown to anyone else.

**Age rating.** The IARC questionnaire in Partner Center at the first submission (11.11,
`WDD hub/apps/publish/publish-your-app/msix/age-ratings.md`). Expected result: the lowest rating
(no violence, no user interaction, no purchases; the AI text is about the user's own footage).

**Certification** takes "up to three business days" and the listing goes live about 15 minutes
after it passes (`WDD hub/apps/publish/publish-your-app/msix/app-certification-process.md`).

## Where data lives in the Store build

Packaged desktop apps get their writes virtualized (`desktop-to-uwp-behind-the-scenes.md`):

- **Files:** new files and folders under `AppData\Local` go to a private per-package location
  that the app still sees at the normal path; files that already existed in the real AppData are
  opened and changed in place. Uninstalling removes the package's private copy.
- **Registry:** writes under `HKCU` are copied on write into a private per-package hive. Explorer
  never sees them.
- Turning virtualization off needs the `unvirtualizedResources` capability, which "is designed for
  certain types of desktop PC games" (`MSIX desktop/flexible-virtualization.md`): not asked for.

So the Store build keeps its own data folder, **`%LocalAppData%\frename-store`**, not the installed
Velopack version's `%LocalAppData%\frename` (`installer.md`, "Data folder"). Sharing that folder was
the first plan; review round 1 showed why it fails:

- the database runs in WAL mode (`app_database.rs`). `frename.db` exists, so the Store build would
  change it in place, but its new `-wal` and `-shm` files would go to the package's private
  storage: two builds on one database with different WAL files, no locking between them, and a
  stale WAL replayed over changes the other build made;
- the Store version lags GitHub by the certification time, so the newer installed version may
  migrate the schema under the older Store build;
- uninstalling the installed version (the obvious last step after moving to the Store) deletes
  `%LocalAppData%\frename` with the database the Store build was using.

With its own folder, which is new and therefore lives in the package's private storage:

- **migration**: on its first start (no `frename.db` in `frename-store` yet) the Store build copies
  the installed version's `%LocalAppData%\frename\frename.db`, if there is one, with the same
  `VACUUM INTO` import the installer uses for zip versions (`old_settings::import_once_from`; it
  reads changes still in the WAL and leaves the old files as they were). Settings and folder
  history carry over; afterwards the two builds are independent. It runs under `--self-test`
  too, so CI proves it inside a real package (below); a demo has a folder of its own and never
  imports. A copy that fails (the installed database busy or unreadable) is tried once more at
  the very next start, over the database of that one session, and never after that or after a
  zip import succeeded, so settings built up in the Store build are never replaced;
- **"Open log"** (batch view): Explorer and the editor it starts run outside the package, where the
  log is at `%LocalAppData%\Packages\<family name>\LocalCache\Local\frename-store`, not at the
  path frename sees. The Store build opens that path (`package::log_path_for_other_apps`, the
  family name taken from the package's install folder), and writes it into its log at start-up;
- **older zip or portable versions**: the same first-start offer and **Import from an old frename
  folder…** in Settings as the installed version has (`installer.md` flow 6), since the Store
  build also keeps its data away from the exe (`package::keeps_data_away_from_exe`);
- uninstalling the Store app removes its folder (clean uninstall, policy 10.2.7) and leaves the
  installed version's data alone;
- the GStreamer registry cache goes to the same folder (`bundled_gstreamer.rs`), so the two builds
  no longer rescan each other's plugins.

**API keys** stay in Windows Credential Manager through `keyring`'s `CredWriteW`. Microsoft
documents nothing about the Win32 credential API from a package; the documented packaged-app
store, `PasswordVault`, holds 20 credentials per app (`WDD hub/apps/develop/security/credential-locker.md`).
The credential store is serviced outside the app process and is not part of file or registry
virtualization, so the keys are expected to be the user's normal entries, shared with the other
builds and kept after uninstall (the privacy policy tells users how to remove them). Not verified
on a real machine: step 9 of the owner's guide checks it.

## What the `store` build changes

`cargo build --release --features store` (`Cargo.toml` `[features]`, `package::STORE_BUILD`):

- **No Velopack**: `run_velopack_hooks` returns at once, `Package::locate` returns `None` (an MSIX
  is not a Velopack package), so no update source is ever created.
- **Data folder** `%LocalAppData%\frename-store` (`package::store_data_dir`, set before anything
  reads it), filled from the installed version's database on the first start (above). The
  start-up log says `(Microsoft Store)`; CI's package test fails without it.
- **Settings → Updates** shows only the version and "Updates come from the Microsoft Store": no
  **Check for updates**, no start-up check, no update dot. `UpdatesState` treats the Store build as
  not installed whatever else it is told, so the installed version's imported update state (start-up
  check on, a newer version saved) changes nothing (unit test).
- **Settings migration**: the installed version's database is copied on the first start, and a
  zip or portable version's can be imported as in the installed version (see "Where data lives").
- **No "Open in frename" in Explorer's menu.** The installer adds it through `HKCU`, which is
  virtualized in a package, and Velopack's hooks do not run. A packaged app registers menu
  entries in its manifest instead (`uap3:FileTypeAssociation` verbs, or
  `desktop4:FileExplorerContextMenus` with a COM `IExplorerCommand` server,
  `WDD hub/apps/desktop/modernize/desktop-to-uwp-extensions.md`). Not in this issue: filed as an
  idea if the owner wants it.
- Everything else (playback, batch actions, AI, subtitles) is the same code.

A compile-time feature, because the owner asked for "a Store build flag or variant" and because it
cannot be switched off by where the exe happens to run. What it guarantees, exactly: the Store exe
never locates a Velopack package, so no update source is ever created and every update path is
off; the `velopack` crate itself is still linked in (making it an optional dependency would put
`cfg` on the whole updates feature for no change in behaviour). The price is a second
`cargo build --release` of the frename crate and a second GStreamer bundle in CI; the dependencies
are shared from the cache. Detecting the package at run time (`GetCurrentPackageFullName`) would
let one exe serve both packages and is the alternative if that build time starts to matter.

## The package

`packaging/windows/msix/AppxManifest.xml`, filled in by `packaging/windows/build-msix.ps1`:

- `Identity Name`, `Publisher` and `PublisherDisplayName` must be exactly Partner Center's
  ("Values in the manifest are case-sensitive. Spaces and other punctuation must also match",
  `WDD hub/apps/publish/view-app-identity-details.md`). They are public, so they are GitHub
  repository **variables** (`STORE_IDENTITY_NAME`, `STORE_PUBLISHER`,
  `STORE_PUBLISHER_DISPLAY_NAME`), not secrets. Unset, a placeholder identity (`frename.dev`,
  `CN=frename-dev`) builds a package that installs for tests but that the Store rejects.
- Version `X.Y.Z.0` from `version.md` (`0.74` → `0.74.0.0`): "the last (fourth) section of the
  version number is reserved for Store use and must be left as 0"
  (`msix/app-package-requirements.md`).
- `EntryPoint="Windows.FullTrustApplication"`, `TargetDeviceFamily Windows.Desktop`
  `MinVersion 10.0.17763.0` (Windows 10 1809; the `uap10` attributes need 19041 —
  `MSIX desktop/desktop-to-uwp-manual-conversion.md`), capabilities `runFullTrust` and
  `internetClient`.
- An app execution alias `frename.exe`, so `frename.exe <folder>` works from a terminal and CI can
  run the self-test inside the package.
- Logos in `packaging/windows/msix/Assets/` made from `frename-icon.ico` at the sizes the
  manifest names (44, 50, 150, 310×150; each far under the kit's 200 KB limit), indexed in
  `resources.pri` by `makepri`.
- Packed with `makeappx pack /h SHA256` (`MSIX package/create-app-package-with-makeappx-tool.md`).
  Not signed: the Store signs it. A plain `.msix` is accepted for upload
  (`WDD hub/apps/publish/publish-your-app/msix/upload-app-packages.md`).
- One listing language, `en-us`; the app itself still follows Windows' language (English or
  Russian). A Russian listing can be added in Partner Center later without a new package.

Microsoft's newer `winapp` CLI can generate a manifest and pack for Rust projects
(`WDD hub/apps/dev-tools/winapp-cli/guides/rust.md`); `makeappx` from the Windows SDK already on
the runners does the same with one less tool.

## CI

- `ci.yml`, job `ci-windows`: after the Velopack packages, builds the `store` variant, bundles
  GStreamer into it (`bundle.ps1`), packs `frename.msix` (version `0.0.1.0`) and uploads it.
  `ci-linux` and `ci-windows` run Clippy and the unit tests on the `store` build too.
- `ci.yml`, new job `ci-windows-store` on a fresh runner (`packaging/windows/test-msix.ps1`):
  1. fails if the runner has a GStreamer;
  2. signs a copy with a throwaway self-signed certificate whose subject is the package's
     publisher, and trusts it in `LocalMachine\TrustedPeople` (Windows installs only signed
     packages; `MSIX package/create-certificate-package-signing.md`);
  3. installs it, checks every DLL import with `check-bundle.ps1`, and runs
     `frename.exe --self-test tests/media` through the alias, inside the package; the log must
     exist and say `Microsoft Store`. Before it, the script makes an installed version's database
     in `%LocalAppData%\frename` (WAL mode, with Python on the runner) unless one exists, and the
     log must then say `settings imported from`: the first-start import works from inside a
     package, through file virtualization, not just in a unit test. The log must also name
     itself, at the path found in the package's storage, as the one "Open log" opens. The script
     refuses to run where the package is installed already (it uninstalls it at the end);
  4. uninstalls it and runs the **Windows App Certification Kit**
     (`appcert.exe reset`, `appcert.exe test -appxpackagepath … -reportoutputpath …`,
     `WDD uwp/debug-test-perf/windows-app-certification-kit.md`); any overall result but PASS
     or WARNING (a missing one too) fails the job, and the report is uploaded as an artifact. If a runner image ever lacks the kit, the
     job says so in its summary and passes on steps 1–3.
- `release.yml`: `build-store` and `test-store` do the same with the release version; the GitHub
  release does not wait for them (a Store problem never blocks a GitHub release), and the MSIX is
  a workflow artifact, not a release asset (an unsigned MSIX cannot be installed by users).
- `release.yml`, `submit-store`: only when the repository variable `MS_STORE_PRODUCT_ID` is set,
  after `test-store` and the GitHub release, it submits the MSIX with Microsoft's `msstore` CLI
  (`microsoft/microsoft-store-apppublisher`, then `msstore reconfigure` and
  `msstore publish <msix> -id <product id>`,
  `WDD hub/apps/publish/msstore-dev-cli/github-actions.md`). The submission API needs a product
  that is already live (the first submission is by hand), an Entra ID app with the Manager role in
  Partner Center, and works for free products (`store-submission-api.md`).

## Decisions made without the owner

1. **MSIX, not the EXE installer**, for the reasons in the table above. The EXE route stays
   possible once #50's certificate exists.
2. **A `store` cargo feature**, not run-time detection (see "What the `store` build changes").
3. **Its own data folder** (`%LocalAppData%\frename-store`) with a one-time copy of the installed
   version's database, so moving from the installer to the Store keeps settings, and the two
   builds never share a live database.
4. **No Explorer context menu in the Store build** for now (see above).
5. **Identity in repository variables** with a placeholder default, so CI builds and tests the
   package before the owner has an account.
6. **The Store package is not attached to GitHub releases**, and a failing Store job does not
   block the GitHub release.
7. **Automated submission is opt-in** (`MS_STORE_PRODUCT_ID`), because the API only works after a
   manual first submission.
8. **Individual account**, publisher name = the owner's legal name.
9. **Listing in English only** at first; category Photo & video.
10. **Privacy policy as a Markdown page in the repository** (a GitHub URL), not GitHub Pages: no
    Pages setup needed, and its history is public.
11. **Screenshots**: the three README screenshots as PNG (the Store takes PNG, 1366×768 or larger,
    `WDD hub/apps/publish/publish-your-app/msix/screenshots-and-images.md`).

## Test plan

- Unit tests: the Store build's data folder and where it imports from (`src/package.rs`); the
  first-start import copies once, including changes still in the WAL, and never again
  (`old_settings.rs`); the Updates state never checks and offers nothing in the Store build, even
  when told it is installed and with a saved newer version (`src/features/updates/state.rs`).
- CI: `ci-windows-store` (install, DLL walk, packaged self-test, certification kit) on every PR.
- By the owner, on Windows, after certification (`store-setup.md` step 9): install from the Store;
  settings of the GitHub-installed version, if any, are there; play a clip, rename a file in a
  normal folder, save an Anthropic key and run Describe with AI on one clip, check the key in
  Credential Manager, uninstall. Optionally before that, the CI package in Windows Sandbox
  (step 5).

## Sources not re-checked here

- Store Policies 7.20 (published 2026-09-15): check 10.2, 10.5, 10.14 and 11.16 on
  <https://learn.microsoft.com/windows/apps/publish/store-policies> before submitting.
- Whether an individual account can associate an Entra ID tenant for the submission API: the docs
  do not restrict it; `store-setup.md` says what to do if Partner Center does not offer it.

## Review notes not taken

- *Render plain window screenshots for the Store instead of the annotated README ones* (product,
  round 1): the issue asks for "screenshots (the README ones)", and they meet the Store's size
  rule. Plain demo-mode captures at 1920×1080 can replace them later without a new package.
- *Make `velopack` an optional dependency* (design, round 1): see "What the `store` build
  changes"; the rationale now says what the flag does and does not guarantee.
- *Bundle GStreamer once and copy it for the Store exe* (design, round 2): the two exes import the
  same DLLs today, but `bundle.ps1` walks the imports of the exe it is given, so a Store-only
  import could never be missed; about a minute of CI per run.
- *Do not commit the Store PNGs, convert at submission time* (design, round 2): the owner should not
  need ImageMagick; `screenshots.md` says how to refresh them when the README ones change.
