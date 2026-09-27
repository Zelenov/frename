# Windows code signing for frename

Research for #50 (signed Windows installer, no "unknown publisher" / SmartScreen warning).
Current as of 2026-09-27. Where a fact could not be confirmed against a primary source (the
research proxy blocked several vendor domains), it is marked "(search snippet)" and should be
re-checked once before being relied on for money or legal decisions.

## Two things about frename that change the recommendation

1. **frename has no open-source license** (no `LICENSE` file, no `license` in `Cargo.toml`). The
   two free/cheap options below (SignPath Foundation, Certum's Open Source certificate) require an
   OSI-approved license as a precondition. Adding one is a prerequisite for those paths, and it is
   the owner's decision, not this issue's.
2. **Eligibility for the individual tier of Azure Artifact Signing depends on the owner's
   country** (see below) — an org's country, not the individual's, decides eligibility. This isn't
   known here, so the setup guide asks the owner to check it against their own situation.

## Options compared

| Option | Cost/year | Solo individual eligible? | CI automation | SmartScreen today |
|---|---|---|---|---|
| Azure Artifact Signing | ~$120 ($9.99/mo Basic) + a **paid** Azure subscription | Individuals: **USA/Canada only**. Orgs: USA, Canada, EU, UK and more (lists differ slightly between Microsoft pages) | Official GitHub Action (`azure/artifact-signing-action`) and built into Velopack (`vpk pack --azureTrustedSignFile`); OIDC login, no long-lived secret | Builds over weeks/months of downloads; no instant trust. CA intermediate rotations have reportedly reset it for some users |
| OV/IV certificate w/ cloud signing (SSL.com eSigner, Sectigo, DigiCert KeyLocker) | ~$130–550 + cloud-signing fees | Yes ("IV", individual validation), in most countries | Works through the CA's own cloud tool, or Velopack's generic `--signTemplate` | Same as Artifact Signing — builds over time |
| Certum "Open Source Code Signing" (SimplySign, cloud) | ~$50–69 first year, ~€29/yr renewal | Yes, individuals only — **requires an OSI license and an active OSS project** | Possible via community tools that automate SimplySign's login, less polished than official actions | Builds over time; publisher shows as "Open Source Developer, `<name>`" |
| EV certificate | ~$270–560 + cloud/HSM | Needs a registered organization | Same mechanics as OV | **No longer instant** — Microsoft removed EV's SmartScreen head start in 2024; EV is treated the same as OV now |
| SignPath Foundation (free for OSS) | Free | Yes, but needs an OSI license, some project reputation, MFA for the whole team, defined roles, a signing-policy page, a privacy statement | GitHub Action exists, but **every release needs a person to click Approve** in SignPath's UI | Builds over time; publisher shows as "SignPath Foundation", not the developer's name |

### Azure Artifact Signing (renamed from "Trusted Signing"/"Azure Code Signing" in 2026)

- Basic: **$9.99/month**, 5,000 signatures, 1 certificate profile of each type. Premium:
  $99.99/month, 100,000 signatures, 10 profiles. Overage $0.005/signature. Requires a **paid**
  Azure subscription — free, trial or sponsored subscriptions are rejected.
  ([FAQ](https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/faq.yml),
  [pricing](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/))
- Eligibility: individuals must be in the **USA or Canada**. Organizations are accepted from a
  wider list (USA, Canada, EU, UK, Australia, New Zealand, Japan, South Korea, Singapore,
  Switzerland, Norway, Israel per the quickstart; a separate Windows docs page gives a shorter
  list — check the portal at signup time).
  ([quickstart](https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/quickstart.md))
  A Norwegian developer was refused as an individual in September 2026 and switched to Certum:
  https://github.com/FrodeHus/elevate/pull/165
- Identity is checked from the Azure billing account's legal name/address against a
  government-issued ID plus a supporting document (utility bill, bank statement), verified by
  AU10TIX. Processing takes 1–20 business days, and the validation must be renewed periodically
  (reminders start 60 days before expiry; certificate issuance stops if it lapses). The old
  preview-era "3 years of business history" rule no longer applies at GA; self-employed
  individuals can apply (search snippet).
- Certificates are short-lived (3 days, auto-rotated), so signatures **must be timestamped**.
- **SmartScreen is not instant.** Microsoft's own Windows docs: "Azure Artifact Signing does
  **not** provide instant SmartScreen trust. New files will show a SmartScreen warning until they
  accumulate sufficient download history."
  ([source](https://raw.githubusercontent.com/MicrosoftDocs/windows-dev-docs/docs/hub/apps/package-and-deploy/code-signing-options.md))
  A March 2026 CA rotation reportedly brought warnings back for some previously-trusted apps
  (Microsoft Q&A, search snippets):
  https://learn.microsoft.com/en-us/answers/questions/5954185/artifact-signing-users-see-smartscreen-on-each-rel
  and https://learn.microsoft.com/en-us/answers/questions/5861538/azure-trusted-signing-still-seeing-smartscreen-war
- **GitHub Actions / CI**: official action `azure/artifact-signing-action@v2`
  (https://github.com/Azure/artifact-signing-action), Windows runners only. Auth via
  `DefaultAzureCredential` — OIDC (`azure/login` + `id-token: write`) is the recommended method
  over a client-secret. Required role: **"Artifact Signing Certificate Profile Signer"**, scoped to
  the certificate profile
  ([role tutorial](https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/tutorial-assign-roles.md)).
- **Velopack has this built in**: `vpk pack ... --azureTrustedSignFile metadata.json`. Velopack's
  docs note signing must happen *inside* `vpk pack`, not as a separate step afterwards, because
  `Update.exe` and `Setup.exe` are produced and need signing at different points of the packaging
  process
  ([source](https://raw.githubusercontent.com/velopack/velopack.docs/master/docs/packaging/signing.mdx)).
  `metadata.json` is `{"Endpoint", "CodeSigningAccountName", "CertificateProfileName"}` and **must
  be UTF-8 without a BOM**, or signing fails; the signing dlib bundled with `vpk` also needs
  **.NET 8** on the runner, and fails *silently* without it (real-world report:
  https://github.com/wivuu/stellarallegiance/issues/101).
  Velopack's own docs are out of date here: they still claim EV/Artifact Signing give "instant
  SmartScreen reputation", which contradicts Microsoft's current docs above.

### OV / IV certificates

- Microsoft's own estimate: $150–300/year. SSL.com from $129/year (individual validation, "IV")
  plus a metered eSigner cloud-signing tier (~$20/month for 20 signings). Sectigo ~$220–226/year,
  DigiCert ~$400–550/year. A certificate now lasts at most 459 days (since 2026-02-27), so
  multi-year purchases need reissuing.
- **Since June 2023, CA/Browser Forum rules require the private key to live on a hardware token or
  a cloud HSM** — a `.pfx` in a GitHub secret is no longer allowed for a newly issued certificate.
  CI therefore has to go through the CA's cloud-signing service (SSL.com eSigner, DigiCert
  KeyLocker, Sectigo/GlobalSign cloud) or an Azure Key Vault HSM with `AzureSignTool`
  (untested here, extra cost).
- **Certum "Open Source Code Signing"** (cloud, SimplySign) is the cheapest paid option (~$50–69
  first year, ~€29/year renewal), but individuals only, and only with an OSI-approved license and
  an active OSS project — frename doesn't currently qualify (no license). CI is possible only via
  community tooling that automates SimplySign's normally-interactive TOTP login (e.g.
  https://github.com/jay0lee/certum-cloud-code-sign, or the HTTPS-only
  https://github.com/Le-Syl21/ssign, both third-party, not official).
- SmartScreen reputation for OV/IV builds up exactly the same way as for Artifact Signing — no
  head start.

### EV certificates

- ~$400+/year (Microsoft's figure); same hardware-token/cloud-HSM requirement as OV.
- **No longer gives instant SmartScreen trust.** Microsoft, May 2026: "EV certificates no longer
  bypass SmartScreen... this behavior no longer exists... Paying a premium for EV solely to avoid
  SmartScreen warnings is no longer justified."
  ([source](https://raw.githubusercontent.com/MicrosoftDocs/windows-dev-docs/docs/hub/apps/package-and-deploy/smartscreen-reputation.md)).
  Microsoft's Trusted Root Program removed EV code-signing OIDs from its roots in August 2024;
  EV and OV are now treated identically.

### SignPath Foundation (free, open source)

- Requires: an OSI-approved license with no proprietary dual-licensing, no proprietary
  components, no malware/PUP, an actively maintained and already-released project, a described
  download page, **some verifiable project reputation** (a similar small project was turned down
  for "lacking" reputation), MFA for every team member on both SignPath and GitHub, defined
  Author/Reviewer/Approver roles, a "Code signing policy" section on the project's homepage
  crediting SignPath, and a privacy statement describing any network calls the signed program
  makes (frename's calls to Anthropic and Soniox would need disclosing).
- CI uploads the unsigned build as a GitHub artifact via
  `SignPath/github-action-submit-signing-request`, but **every release needs a person to approve
  it by hand** in SignPath's UI — this does not fit an unattended nightly release pipeline.
  The published binary is signed as "SignPath Foundation", not as "frename" or the owner's name.
- **Poor fit for frename today**: no license yet, 1 star (reputation bar likely not met), and the
  manual per-release approval conflicts with how this project ships releases.

## How SmartScreen reputation actually works

Per Microsoft's own docs (2026-05-04): reputation grows from download/install volume and
behavior, with "no exact threshold" — expect "several weeks and hundreds of clean installs from a
wide audience."
- **Unsigned files**: reputation is per file hash. Every release starts at zero.
- **Signed files**: reputation accrues to the certificate/publisher identity, so later releases
  signed with the *same* identity inherit it.
- **Switching identities resets it.** Moving from unsigned to signed, or between signing
  providers later, starts reputation over. Whichever option is chosen, it should be kept
  long-term rather than switched.
- Even once signed, early installs still show "Unknown app", but with the real publisher name
  instead of "Unknown publisher" — this is the actual, achievable improvement, not the immediate
  removal of every warning.

## Recommendation

**Azure Artifact Signing (Basic, $9.99/month)**, conditional on the owner being eligible (USA/
Canada as an individual, or a wider list of countries as a registered organization):

- It's the only option that fits both an unattended release pipeline (no manual per-release
  approval) and Velopack's own packaging step (a single `vpk pack` flag), authenticated with OIDC
  and no long-lived secret to rotate.
- Microsoft's current docs recommend it as the default for apps distributed outside the Store.

**If the owner is an individual outside the USA/Canada**, and not registering a business: the
"Done when" bar in #50 (a verified publisher name, not an instant removal of every SmartScreen
warning) is still reachable, just not through the fully-automated Azure path:
1. Cheapest: **Certum Open Source** (cloud) — needs adding an OSI license to frename first, plus
   community (non-official) CI tooling.
2. More automation-friendly but pricier: **SSL.com IV certificate with eSigner** — an official
   cloud-signing tool, works from any country SSL.com serves.

**Not recommended:** EV (no longer buys anything extra over OV/Artifact Signing) and SignPath
(manual per-release approval breaks the nightly pipeline; frename doesn't meet the license/
reputation bar yet either way).

**What to tell users regardless of the option chosen:** no option removes the SmartScreen warning
on day one. Signing replaces "Unknown publisher" with frename's real name; the warning itself
fades only after real-world download history builds up under that one identity, and switching
identities later throws that history away.

## Sources consulted directly (not search snippets)

- https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/faq.yml
- https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/quickstart.md
- https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/tutorial-assign-roles.md
- https://raw.githubusercontent.com/MicrosoftDocs/azure-docs/main/articles/artifact-signing/how-to-signing-integrations.md
- https://raw.githubusercontent.com/MicrosoftDocs/windows-dev-docs/docs/hub/apps/package-and-deploy/code-signing-options.md
- https://raw.githubusercontent.com/MicrosoftDocs/windows-dev-docs/docs/hub/apps/package-and-deploy/smartscreen-reputation.md
- https://raw.githubusercontent.com/velopack/velopack.docs/master/docs/packaging/signing.mdx
- https://github.com/Azure/artifact-signing-action
- https://github.com/FrodeHus/elevate/pull/165
- https://github.com/wivuu/stellarallegiance/issues/101
- https://github.com/electron-userland/electron-builder/pull/10190 (confirms EV's SmartScreen
  change, corrected in electron-builder's own docs 2026-09-17)
