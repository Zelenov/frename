//! The spend ledger and top-ups in the app database (issue #121); see [`crate::ai::ledger`].
//!
//! "Today" and "this month" are the user's local calendar ones; SQLite works them out, so no
//! time zone code lives here.

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use super::app_database::{lock_connection, AppDatabase};
use crate::ai::key::ApiKey;
use crate::ai::ledger::{SpendSummary, TopUp};

fn service_id(service: ApiKey) -> &'static str {
    match service {
        ApiKey::Anthropic => "anthropic",
        ApiKey::Soniox => "soniox",
    }
}

/// Milliseconds since the Unix epoch, now.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// The start (UTC milliseconds) of the local day or month that holds `at_ms`.
fn local_start_ms(conn: &Connection, at_ms: i64, unit: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        &format!(
            "SELECT CAST(strftime('%s', datetime(?1 / 1000, 'unixepoch'), 'localtime', 'start of {unit}', 'utc') AS INTEGER) * 1000"
        ),
        [at_ms],
        |row| row.get(0),
    )
}

/// Whether `date` is `YYYY-MM-DD` with a real month and day.
fn is_date(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();
    let numeric = |s: &str, len: usize| s.len() == len && s.bytes().all(|b| b.is_ascii_digit());
    matches!(parts.as_slice(), [y, m, d]
        if numeric(y, 4) && numeric(m, 2) && numeric(d, 2)
            && (1..=12).contains(&m.parse::<u32>().unwrap_or(0))
            && (1..=31).contains(&d.parse::<u32>().unwrap_or(0)))
}

impl AppDatabase {
    /// Record what one file of a batch cost on `service`, now.
    pub fn record_ai_spend(&self, service: ApiKey, usd: f64) {
        self.record_ai_spend_at(service, usd, now_ms());
    }

    /// [`Self::record_ai_spend`] at a given time (milliseconds since the Unix epoch).
    pub fn record_ai_spend_at(&self, service: ApiKey, usd: f64, at_ms: i64) {
        if !(usd.is_finite() && usd > 0.0) {
            return;
        }
        if let Ok(conn) = self.conn() {
            let conn = lock_connection(&conn);
            if let Err(e) = conn.execute(
                "INSERT INTO ai_spend (service, at_ms, usd) VALUES (?1, ?2, ?3)",
                params![service_id(service), at_ms, usd],
            ) {
                log::warn!("ledger: could not record ${usd:.4} spent on {service:?}: {e}");
            }
        }
    }

    /// What `service` cost today and this month (the local calendar), since its recorded
    /// top-up, and that top-up. A database that cannot be read gives nothing recorded.
    pub fn ai_spend_summary(&self, service: ApiKey) -> SpendSummary {
        self.ai_spend_summary_at(service, now_ms())
    }

    /// [`Self::ai_spend_summary`] as of a given time.
    pub fn ai_spend_summary_at(&self, service: ApiKey, now_ms: i64) -> SpendSummary {
        let conn = match self.conn() {
            Ok(conn) => conn,
            Err(e) => {
                log::warn!("ledger: the database cannot be opened: {e}");
                return SpendSummary::default();
            }
        };
        let conn = lock_connection(&conn);
        let id = service_id(service);
        let spent_since = |from_ms: i64| -> f64 {
            conn.query_row(
                "SELECT COALESCE(SUM(usd), 0) FROM ai_spend WHERE service = ?1 AND at_ms >= ?2 AND at_ms <= ?3",
                params![id, from_ms, now_ms],
                |row| row.get(0),
            )
            .unwrap_or(0.0)
        };
        let day = local_start_ms(&conn, now_ms, "day").unwrap_or(now_ms);
        let month = local_start_ms(&conn, now_ms, "month").unwrap_or(day);
        let top_up = conn
            .query_row(
                "SELECT usd, at_ms FROM ai_top_up WHERE service = ?1",
                [id],
                |row| {
                    Ok(TopUp {
                        usd: row.get(0)?,
                        at_ms: row.get(1)?,
                    })
                },
            )
            .ok();
        SpendSummary {
            today: spent_since(day),
            month: spent_since(month),
            since_top_up: top_up.map(|t| spent_since(t.at_ms)),
            top_up,
        }
    }

    /// Record that the user added `usd` to `service`, on the local day `date` (`YYYY-MM-DD`) or
    /// now; the estimate of what is left starts from it. `false` when the amount or the date is
    /// not usable.
    pub fn set_ai_top_up(&self, service: ApiKey, usd: f64, date: Option<&str>) -> bool {
        if !(usd.is_finite() && usd >= 0.0) || date.is_some_and(|d| !is_date(d)) {
            return false;
        }
        let Ok(conn) = self.conn() else {
            return false;
        };
        let conn = lock_connection(&conn);
        let at_ms = match date {
            // Midnight of that local day, as UTC. SQLite would quietly move a day that does not
            // exist (2026-02-31) to the next month: read the date back and refuse a change.
            Some(date) => conn
                .query_row(
                    "SELECT CAST(strftime('%s', ?1, 'utc') AS INTEGER) * 1000, date(?1)",
                    [date],
                    |row| {
                        Ok((
                            row.get::<_, Option<i64>>(0)?,
                            row.get::<_, Option<String>>(1)?,
                        ))
                    },
                )
                .ok()
                .filter(|(_, read_back)| read_back.as_deref() == Some(date))
                .and_then(|(at_ms, _)| at_ms),
            None => Some(now_ms()),
        };
        let Some(at_ms) = at_ms else {
            return false;
        };
        conn.execute(
            "INSERT INTO ai_top_up (service, usd, at_ms) VALUES (?1, ?2, ?3)
             ON CONFLICT(service) DO UPDATE SET usd = excluded.usd, at_ms = excluded.at_ms",
            params![service_id(service), usd, at_ms],
        )
        .is_ok()
    }

    /// The service reported no credit left: what is left is zero until a new top-up is recorded.
    pub fn mark_ai_credit_used_up(&self, service: ApiKey) {
        self.set_ai_top_up(service, 0.0, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Initializable;

    const HOUR: i64 = 3_600_000;
    const DAY: i64 = 24 * HOUR;

    fn database(name: &str) -> AppDatabase {
        let path =
            std::env::temp_dir().join(format!("frename-ledger-{name}-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let db = AppDatabase::with_path(&path);
        db.initialize().expect("migrate");
        db
    }

    #[test]
    fn spend_adds_up_by_service_and_period() {
        let db = database("periods");
        let now = 1_790_000_000_000;
        db.record_ai_spend_at(ApiKey::Anthropic, 0.25, now);
        db.record_ai_spend_at(ApiKey::Anthropic, 0.10, now - HOUR / 2);
        // Long ago: in no period the summary shows.
        db.record_ai_spend_at(ApiKey::Anthropic, 9.0, now - 70 * DAY);
        db.record_ai_spend_at(ApiKey::Soniox, 3.0, now);

        let anthropic = db.ai_spend_summary_at(ApiKey::Anthropic, now);
        assert!((anthropic.today - 0.35).abs() < 1e-9, "{anthropic:?}");
        assert!(anthropic.month >= anthropic.today);
        assert!(anthropic.month < 9.0, "{anthropic:?}");
        assert_eq!(anthropic.since_top_up, None);
        assert_eq!(anthropic.remaining(), None);
        let soniox = db.ai_spend_summary_at(ApiKey::Soniox, now);
        assert!((soniox.today - 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_day_ago_is_not_today_and_a_year_ago_is_not_this_month() {
        let db = database("edges");
        let now = 1_790_000_000_000;
        db.record_ai_spend_at(ApiKey::Anthropic, 1.0, now - 2 * DAY);
        db.record_ai_spend_at(ApiKey::Anthropic, 2.0, now - 400 * DAY);
        let s = db.ai_spend_summary_at(ApiKey::Anthropic, now);
        assert_eq!(s.today, 0.0, "two days ago is not today: {s:?}");
        assert!(s.month <= 1.0, "{s:?}");
    }

    #[test]
    fn this_month_starts_on_the_first_and_today_at_midnight() {
        let db = database("month");
        // Mid-month, mid-day in UTC: the local day and month hold it in every time zone.
        let now = 1_790_000_000_000 + 10 * DAY; // some day in the middle of a month
        let conn_bounds = |unit: &str| {
            let conn = db.conn().expect("conn");
            let conn = lock_connection(&conn);
            local_start_ms(&conn, now, unit).expect("start")
        };
        let (day, month) = (conn_bounds("day"), conn_bounds("month"));
        assert!(
            day <= now && now - day < 25 * HOUR,
            "today starts within a day"
        );
        assert!(
            month <= day && day - month < 31 * DAY,
            "the month starts within 31 days"
        );

        // Spend just before each start is outside its period, just after is inside.
        db.record_ai_spend_at(ApiKey::Anthropic, 1.0, day - 1);
        db.record_ai_spend_at(ApiKey::Anthropic, 2.0, day);
        db.record_ai_spend_at(ApiKey::Anthropic, 4.0, month - 1);
        let s = db.ai_spend_summary_at(ApiKey::Anthropic, now);
        assert!((s.today - 2.0).abs() < 1e-9, "{s:?}");
        let expected_month = if month == day { 2.0 } else { 3.0 };
        assert!((s.month - expected_month).abs() < 1e-9, "{s:?}");
    }

    #[test]
    fn the_estimate_is_the_top_up_minus_what_was_spent_since() {
        let db = database("topup");
        let now = now_ms();
        db.record_ai_spend_at(ApiKey::Anthropic, 5.0, now - 3 * DAY);
        assert!(db.set_ai_top_up(ApiKey::Anthropic, 20.0, None));
        db.record_ai_spend_at(ApiKey::Anthropic, 1.5, now + HOUR);

        let s = db.ai_spend_summary_at(ApiKey::Anthropic, now + 2 * HOUR);
        assert_eq!(s.top_up.map(|t| t.usd), Some(20.0));
        assert!(
            (s.since_top_up.expect("a top-up") - 1.5).abs() < 1e-9,
            "{s:?}"
        );
        assert!((s.remaining().expect("known") - 18.5).abs() < 1e-9);
        assert_eq!(
            db.ai_spend_summary_at(ApiKey::Soniox, now).remaining(),
            None,
            "per service"
        );
    }

    #[test]
    fn a_top_up_on_an_earlier_date_counts_the_spend_since_that_day() {
        let db = database("topup-date");
        let now = now_ms();
        db.record_ai_spend_at(ApiKey::Anthropic, 4.0, now - 5 * HOUR);
        // Two days ago, the start of that local day.
        let two_days_ago_ms = now - 2 * DAY;
        let date: String = {
            let conn = db.conn().expect("conn");
            let conn = lock_connection(&conn);
            conn.query_row(
                "SELECT date(?1 / 1000, 'unixepoch', 'localtime')",
                [two_days_ago_ms],
                |row| row.get(0),
            )
            .expect("date")
        };
        assert!(db.set_ai_top_up(ApiKey::Anthropic, 10.0, Some(&date)));
        let s = db.ai_spend_summary_at(ApiKey::Anthropic, now);
        assert!(
            (s.since_top_up.expect("a top-up") - 4.0).abs() < 1e-9,
            "{s:?}"
        );
        assert!((s.remaining().expect("known") - 6.0).abs() < 1e-9);
    }

    #[test]
    fn a_top_up_that_is_not_usable_is_refused() {
        let db = database("refused");
        assert!(!db.set_ai_top_up(ApiKey::Anthropic, -1.0, None));
        assert!(!db.set_ai_top_up(ApiKey::Anthropic, f64::NAN, None));
        assert!(!db.set_ai_top_up(ApiKey::Anthropic, 5.0, Some("28.09.2026")));
        assert!(!db.set_ai_top_up(ApiKey::Anthropic, 5.0, Some("2026-13-01")));
        assert!(
            !db.set_ai_top_up(ApiKey::Anthropic, 5.0, Some("2026-02-31")),
            "a day that does not exist"
        );
        assert_eq!(db.ai_spend_summary(ApiKey::Anthropic).top_up, None);
        assert!(db.set_ai_top_up(ApiKey::Anthropic, 5.0, Some("2026-02-28")));
    }

    #[test]
    fn no_credit_left_sets_what_is_left_to_zero_until_a_new_top_up() {
        let db = database("used-up");
        assert!(db.set_ai_top_up(ApiKey::Soniox, 20.0, None));
        db.mark_ai_credit_used_up(ApiKey::Soniox);
        assert_eq!(db.ai_spend_summary(ApiKey::Soniox).remaining(), Some(0.0));
        assert!(db.set_ai_top_up(ApiKey::Soniox, 10.0, None));
        assert_eq!(db.ai_spend_summary(ApiKey::Soniox).remaining(), Some(10.0));
    }

    #[test]
    fn a_spend_that_is_not_a_positive_number_is_not_recorded() {
        let db = database("junk");
        let now = now_ms();
        for usd in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            db.record_ai_spend_at(ApiKey::Anthropic, usd, now);
        }
        assert_eq!(db.ai_spend_summary_at(ApiKey::Anthropic, now).month, 0.0);
    }
}
