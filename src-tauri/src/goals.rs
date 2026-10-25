use std::collections::BTreeMap;

use crate::timeutil::shift_day;

/// Count consecutive days that meet `target_seconds`, ending today.
/// A day that is not finished yet does not break yesterday's streak.
pub fn current_streak(totals: &BTreeMap<String, i64>, today: &str, target_seconds: i64) -> i64 {
    if target_seconds <= 0 {
        return 0;
    }
    let mut day = today.to_string();
    if totals.get(&day).copied().unwrap_or(0) < target_seconds {
        day = shift_day(&day, -1);
    }
    let mut streak = 0;
    let mut guard = 0;
    while totals.get(&day).copied().unwrap_or(0) >= target_seconds && guard < 3650 {
        streak += 1;
        day = shift_day(&day, -1);
        guard += 1;
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;

    fn totals(values: &[(&str, i64)]) -> BTreeMap<String, i64> {
        values.iter().map(|(d, s)| ((*d).to_string(), *s)).collect()
    }

    #[test]
    fn streak_includes_today_when_reached() {
        let t = totals(&[
            ("2026-10-24", 4 * 3600),
            ("2026-10-23", 5 * 3600),
            ("2026-10-22", 4 * 3600),
            ("2026-10-21", 3 * 3600),
        ]);
        assert_eq!(current_streak(&t, "2026-10-24", 4 * 3600), 3);
    }

    #[test]
    fn unfinished_today_keeps_yesterdays_streak() {
        let t = totals(&[
            ("2026-10-24", 3600),
            ("2026-10-23", 4 * 3600),
            ("2026-10-22", 4 * 3600),
            ("2026-10-21", 1_000),
        ]);
        assert_eq!(current_streak(&t, "2026-10-24", 4 * 3600), 2);
    }

    #[test]
    fn empty_history_has_no_streak() {
        assert_eq!(current_streak(&BTreeMap::new(), "2026-10-24", 3600), 0);
        assert_eq!(current_streak(&BTreeMap::new(), "2026-10-24", 0), 0);
    }
}
