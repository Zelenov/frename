# Setting up Windows code signing (owner guide)

Follow this once to make `.github/workflows/release.yml` sign `frename.exe`, the Velopack
installer (`frename-win-Setup.exe`) and the update packages with **Azure Artifact Signing**. See
`docs/research/windows-signing.md` for why this option was chosen and what it does and doesn't
fix (it does not remove the SmartScreen warning on day one — it replaces "Unknown publisher" with
frename's name, and the warning itself fades as real installs build up history).

**Before you start: check you're eligible.** Individuals can use Azure Artifact Signing only if
they are based in the USA or Canada; a registered organization is accepted from a wider list of
countries (see the research doc). If that doesn't fit you, the research doc's "Recommendation"
section names the alternatives (Certum Open Source needs frename to have an OSI license first;
SSL.com IV is pricier but works from more countries) — those aren't wired into `release.yml` yet
and would need a follow-up issue.

Nothing here needs frename's source code to change or any secret to be committed. Everything
below is Azure Portal / CLI steps and GitHub repository secrets.

## 1. Azure setup

1. Get a **paid** Azure subscription (a free/trial/sponsored one is rejected for this service).
2. In the Azure Portal, create a resource group for this, e.g. `frename-signing`.
3. Create an **Artifact Signing account** (search "Artifact Signing" in the portal) in that
   resource group. Pick a region close to you — you'll need its short code later (e.g. `weu` for
   West Europe, `eus` for East US); it becomes part of the endpoint URL.
   - SKU: **Basic** ($9.99/month, 5,000 signatures/month) is enough for frename's release volume.
4. Inside the account, complete **identity validation** (you need the "Identity Verifier" role on
   the account, which the account creator has by default). This is a real ID check by AU10TIX
   (government ID plus a utility bill or bank statement) and can take 1–20 business days. Do this
   step first — everything else can wait on it.
5. Once verified, create a **certificate profile**: type **Public Trust**, and note its exact
   name.
6. Note down, exactly as shown in the portal:
   - the **endpoint URL** for your account's region, e.g. `https://weu.codesigning.azure.net`
   - the **signing account name**
   - the **certificate profile name**

## 2. Let GitHub Actions authenticate (OIDC, no long-lived secret)

Run these with the [Azure CLI](https://learn.microsoft.com/cli/azure/install-azure-cli), signed in
as yourself (`az login`):

```sh
# 1. Register an app for GitHub Actions to use.
az ad app create --display-name frename-release-signing
# note the appId (this is your AZURE_CLIENT_ID) and note your tenant: az account show --query tenantId

# 2. Create the matching service principal.
az ad sp create --id <appId>

# 3. Let GitHub's OIDC tokens authenticate as this app, scoped to this repo's main branch
#    (release.yml only runs on pushes to main).
az ad app federated-credential create --id <appId> --parameters '{
  "name": "frename-release-main",
  "issuer": "https://token.actions.githubusercontent.com",
  "subject": "repo:Zelenov/frename:ref:refs/heads/main",
  "audiences": ["api://AzureADTokenExchange"]
}'

# 4. Grant that app permission to sign with your certificate profile (nothing broader).
az role assignment create \
  --assignee <appId> \
  --role "Artifact Signing Certificate Profile Signer" \
  --scope "/subscriptions/<subscriptionId>/resourceGroups/<resourceGroup>/providers/Microsoft.CodeSigning/codeSigningAccounts/<accountName>/certificateProfiles/<profileName>"
```

Replace `<appId>`, `<subscriptionId>`, `<resourceGroup>`, `<accountName>`, `<profileName>` with
your own values from step 1.

## 3. Add these GitHub Actions secrets

Repo → Settings → Secrets and variables → Actions → New repository secret. Add all six (the
workflow checks whether they're set — with none set, releases build unsigned as before; once
all six are set, the workflow requires signing to succeed):

| Secret name | Value |
|---|---|
| `AZURE_CLIENT_ID` | the app's `appId` from step 2.1 |
| `AZURE_TENANT_ID` | your Azure tenant ID (`az account show --query tenantId`) |
| `AZURE_SUBSCRIPTION_ID` | your Azure subscription ID |
| `AZURE_SIGNING_ENDPOINT` | the endpoint URL from step 1.6, e.g. `https://weu.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT` | the signing account name from step 1.6 |
| `AZURE_SIGNING_PROFILE` | the certificate profile name from step 1.6 |

None of these are secret in the traditional sense (OIDC means there's no password or certificate
file to leak), but GitHub secrets are still the right place for them — they're masked in logs and
kept out of the repository.

## 4. Verify it worked

Trigger a release the normal way (merge a `version.md` bump to `main`). In the `build-windows`
job:
- the "Signing status" step's job summary should say the build was signed, not "unsigned";
- the "Verify code signatures" step runs `signtool verify /pa` on `frename-win-Setup.exe` and the
  packaged `frename.exe`, and fails the job if either isn't validly signed.

Then, **by hand** (the agent can't check this): download the published `frename-win-Setup.exe` on
a Windows machine that hasn't run it before and see what SmartScreen shows. Expect the publisher
name to appear (no longer "Unknown publisher"), but do not expect the warning banner itself to be
gone on the first few downloads — see "How SmartScreen reputation actually works" in
`docs/research/windows-signing.md`. Once you've confirmed the publisher name shows correctly,
update the "unknown publisher" sentence in README.md's Requirements section and add a `version.md`
entry, since that's the point where the change becomes real for users.

## If you ever need to rotate credentials

Federated credentials don't expire the way a client secret does, but if you ever need to replace
the app: repeat step 2, then update the `AZURE_CLIENT_ID` secret (and any others that changed).
Do **not** switch to a different signing account/profile or a different signing method later
without a good reason — SmartScreen reputation is tied to that specific signing identity, and
switching throws away everything built up so far (see the research doc).
