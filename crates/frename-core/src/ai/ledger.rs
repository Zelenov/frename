//! What frename has spent on the paid services, and what is probably left (issue #121).
//!
//! The spend is frename's own record: every AI batch reports what each file cost (from the
//! provider's own usage in each response), and it is kept per service in the app database (see
//! [`crate::AppDatabase::record_ai_spend`]). Neither provider tells a normal API key what is left
//! of its prepaid credit, so the remaining amount is an **estimate**: the top-up the user
//! recorded, minus the spend recorded since then.

use super::key::ApiKey;

/// A top-up the user recorded: "I added $20 on 2026-09-28".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopUp {
    pub usd: f64,
    /// When it was added: milliseconds since the Unix epoch.
    pub at_ms: i64,
}

/// What one service cost, by period, and its top-up if one was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpendSummary {
    pub today: f64,
    pub month: f64,
    /// Spent since the recorded top-up; `None` without one.
    pub since_top_up: Option<f64>,
    pub top_up: Option<TopUp>,
}

impl SpendSummary {
    /// The estimated credit left: the top-up minus what was spent since, never below zero.
    /// `None` without a recorded top-up.
    pub fn remaining(&self) -> Option<f64> {
        let top_up = self.top_up?;
        Some((top_up.usd - self.since_top_up.unwrap_or(0.0)).max(0.0))
    }

    /// Whether a run costing about `cost` would probably use more than what is left. Without a
    /// recorded top-up nothing is known, so nothing is warned about.
    pub fn would_exceed(&self, cost: f64) -> bool {
        self.remaining().is_some_and(|left| cost > left)
    }
}

/// The page where credit is added for `service`.
pub fn billing_url(service: ApiKey) -> &'static str {
    match service {
        ApiKey::Anthropic => "https://console.anthropic.com/settings/billing",
        ApiKey::Soniox => "https://console.soniox.com/org/billing/overview",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(top_up: Option<f64>, since: Option<f64>) -> SpendSummary {
        SpendSummary {
            top_up: top_up.map(|usd| TopUp { usd, at_ms: 0 }),
            since_top_up: since,
            ..SpendSummary::default()
        }
    }

    #[test]
    fn what_is_left_is_the_top_up_minus_the_spend_since() {
        assert_eq!(summary(Some(20.0), Some(4.5)).remaining(), Some(15.5));
        assert_eq!(summary(Some(20.0), None).remaining(), Some(20.0));
        assert_eq!(summary(None, None).remaining(), None, "nothing recorded");
    }

    #[test]
    fn what_is_left_is_never_negative() {
        assert_eq!(summary(Some(1.0), Some(3.0)).remaining(), Some(0.0));
    }

    #[test]
    fn a_run_is_warned_about_only_when_a_top_up_is_known_and_too_small() {
        let s = summary(Some(5.0), Some(4.0));
        assert!(s.would_exceed(1.5));
        assert!(!s.would_exceed(1.0), "exactly what is left is fine");
        assert!(!summary(None, None).would_exceed(100.0));
    }
}
