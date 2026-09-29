# Spending and credit left on the paid services (#121)

The owner wants to see how much can still be spent before a batch stops with "the account has no
credit left". The two paid services are Anthropic (Describe with AI) and Soniox (Generate
subtitles).

## What the providers allow (researched 2026-09-29)

| Provider | Remaining prepaid balance | Spend | Key needed |
|---|---|---|---|
| Anthropic | **No endpoint.** Only the Console billing page shows it (an SDK feature request for it is open: anthropics/anthropic-sdk-python#1942). | `GET /v1/organizations/cost_report` (USD per day, in **cents** as a decimal string), `GET /v1/organizations/usage_report/messages` (tokens). Both are Admin API. | An Admin key (`sk-ant-admin01-…`), and the Console account must be an *organization*: individual accounts cannot use the Admin API. Not a normal key. |
| Anthropic | – | `usage.input_tokens` / `output_tokens` in every Messages response (frename already reads them) | Normal key |
| Soniox | **No endpoint found** (`sonisub`'s `usage.rs` says the same and points to the console's billing page). | `GET /v1/usage-logs` (per request `cost_usd`, at most 31 days per call, 91 days kept), scoped to the key's project. frename already uses it for its price per hour. | Project key |

Sources: platform.claude.com/docs/en/manage-claude/usage-cost-api and `…/api/beta/organization/cost_report/retrieve`, `…/admin-api`; the sonisub crate (`src/usage.rs`, `src/soniox.rs`); Soniox's docs (usage logs, errors) could only be read through search snippets from the sandbox that researched this. Not verified with a real Admin key.

Both providers report **spend, never the balance**. The only real signal about the balance is the
error a request gets when it is empty (`credit balance is too low` on Anthropic,
`organization_balance_exhausted` on Soniox): frename already stops the job on it.

## What frename does

1. **A spend ledger** (`ai_spend` table, migration 17): one row per file a batch billed, with the
   service, the time and the dollars. Dollars are Anthropic's own token usage from each response at
   the model's price (`Model::cost_usd`), and for Soniox the audio actually sent (`Outcome::uploaded_s`)
   at the price the estimate used. A file rebuilt from a saved transcript sent nothing and costs
   nothing. Today and this month are the **local** calendar (SQLite works them out, so no time zone
   code); since-the-top-up is exact.
2. **A recorded top-up** (`ai_top_up` table, one row per service): the amount the user added and the
   day (empty: now). **Remaining = top-up − spend since it**, never below zero, marked "estimate"
   everywhere. A service that reports an empty balance sets the top-up to 0 (now), so the estimate
   reads 0 until a new top-up is recorded.
3. **Where it shows:** Settings → Describe with AI and → Subtitles ("Spending": today, this month,
   since the top-up, credit left, the top-up form); the batch page before Run (a "Credit left" row
   in the plan, and a warning with an "Add credit" button when the run costs more than what is
   left).

Only frename's own spending is counted: another program using the same key, or spend before frename
ran, is not, which is why the number is an estimate and the top-up form exists.

## Decisions made without the owner

- **No admin-key integration.** Part 3 of the issue (show the provider's own number, correct the
  estimate from it) is not built: no balance endpoint exists, the cost report needs an *organization*
  Admin key that most users of this app will not have, and it could not be tried without one. It is
  filed as an `idea` issue. Spend from the cost report would only make "spent" more complete, not
  give the balance.
- The ledger is per **service**, not per model or key: two Anthropic keys on one account share it.
- Soniox's billing page opened by "Add credit" is `console.soniox.com/org/billing/overview`
  (from sonisub).
- A run that costs more than the estimate still runs: the warning does not block, since the
  estimate can be wrong in either direction.
- Cancelled or failed requests that were billed anyway (a timeout) are not counted, as the job report
  already says the spend is then a lower bound.
- The top-up day is typed as `YYYY-MM-DD`: no date picker exists in the app's design system.

## Tests

`ai::ledger` (remaining, warning), `db::ledger` (periods, top-up dates, refused input, used-up
reset), the batch panel helpers, the settings form, the subtitle result (saved transcript costs
nothing, empty balance sets `out_of_credit`).
