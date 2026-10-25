use chrono::{DateTime, Datelike, Duration, Local, LocalResult, NaiveDate, TimeZone};

pub fn local_datetime(ts: i64) -> DateTime<Local> {
    match Local.timestamp_opt(ts, 0) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(a, _) => a,
        LocalResult::None => Local
            .timestamp_opt(ts, 0)
            .earliest()
            .unwrap_or_else(Local::now),
    }
}

pub fn local_day(ts: i64) -> String {
    local_datetime(ts).format("%Y-%m-%d").to_string()
}

pub fn local_time(ts: i64) -> String {
    local_datetime(ts).format("%H:%M").to_string()
}

pub fn local_datetime_string(ts: i64) -> String {
    local_datetime(ts).format("%Y-%m-%d %H:%M").to_string()
}

pub fn parse_day(day: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(day, "%Y-%m-%d").ok()
}

/// First second of a local calendar day.
pub fn start_of_day(day: &str) -> Option<i64> {
    let date = parse_day(day)?;
    let naive = date.and_hms_opt(0, 0, 0)?;
    match naive.and_local_timezone(Local) {
        LocalResult::Single(dt) => Some(dt.timestamp()),
        LocalResult::Ambiguous(a, b) => Some(a.timestamp().min(b.timestamp())),
        LocalResult::None => {
            // Some zones skip midnight around DST. Walk forward to the first valid local time.
            let mut candidate = naive;
            for _ in 0..180 {
                candidate += Duration::minutes(1);
                if let LocalResult::Single(dt) = candidate.and_local_timezone(Local) {
                    return Some(dt.timestamp());
                }
            }
            None
        }
    }
}

pub fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn shift_day(day: &str, delta: i64) -> String {
    parse_day(day)
        .map(|d| (d + Duration::days(delta)).format("%Y-%m-%d").to_string())
        .unwrap_or_else(today_str)
}

pub fn next_day_start(day: &str) -> Option<i64> {
    start_of_day(&shift_day(day, 1))
}

/// Monday..Sunday boundaries for the week containing `day`.
pub fn week_bounds(day: &str) -> (String, String) {
    let date = parse_day(day).unwrap_or_else(|| Local::now().date_naive());
    let weekday_from_monday = date.weekday().num_days_from_monday() as i64;
    let monday = date - Duration::days(weekday_from_monday);
    let sunday = monday + Duration::days(6);
    (
        monday.format("%Y-%m-%d").to_string(),
        sunday.format("%Y-%m-%d").to_string(),
    )
}

pub fn duration_label(seconds: i64) -> String {
    let s = seconds.max(0);
    let h = s / 3600;
    let m = (s % 3600) / 60;
    if h > 0 {
        format!("{h}h {m:02}m")
    } else {
        format!("{m}m")
    }
}

/// Split an interval at every local midnight so reports can group by `day`.
/// Returns `(start, end, day)` fragments. Empty or invalid intervals yield nothing.
pub fn split_at_midnight(start: i64, end: i64) -> Vec<(i64, i64, String)> {
    if end <= start {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut cur = start;
    let mut guard = 0;
    while cur < end && guard < 4000 {
        let day = local_day(cur);
        let next = next_day_start(&day).unwrap_or(end);
        let seg_end = next.clamp(cur + 1, end);
        out.push((cur, seg_end, day));
        cur = seg_end;
        guard += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_ts(y: i32, m: u32, d: u32, hh: u32, mm: u32) -> i64 {
        Local
            .with_ymd_and_hms(y, m, d, hh, mm, 0)
            .single()
            .expect("local timestamp")
            .timestamp()
    }

    #[test]
    fn splits_session_at_midnight() {
        let start = local_ts(2026, 3, 10, 23, 40);
        let end = local_ts(2026, 3, 11, 0, 20);
        let parts = split_at_midnight(start, end);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].0, start);
        assert_eq!(parts[0].1 - parts[0].0, 20 * 60);
        assert_eq!(parts[0].2, "2026-03-10");
        assert_eq!(parts[1].1 - parts[1].0, 20 * 60);
        assert_eq!(parts[1].2, "2026-03-11");
        assert_eq!(parts[0].1, parts[1].0);
    }

    #[test]
    fn splits_interval_across_three_days() {
        let start = local_ts(2026, 3, 10, 22, 0);
        let end = local_ts(2026, 3, 12, 2, 0);
        let parts = split_at_midnight(start, end);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts.iter().map(|p| p.1 - p.0).sum::<i64>(), 28 * 3600);
        assert_eq!(parts[1].2, "2026-03-11");
    }

    #[test]
    fn ignores_empty_interval() {
        assert!(split_at_midnight(100, 100).is_empty());
        assert!(split_at_midnight(200, 100).is_empty());
    }

    #[test]
    fn week_is_monday_to_sunday() {
        assert_eq!(
            week_bounds("2026-10-24"),
            ("2026-10-19".into(), "2026-10-25".into())
        );
        assert_eq!(
            week_bounds("2026-10-19"),
            ("2026-10-19".into(), "2026-10-25".into())
        );
        assert_eq!(
            week_bounds("2026-10-25"),
            ("2026-10-19".into(), "2026-10-25".into())
        );
    }

    #[test]
    fn day_boundaries_are_monotonic() {
        let start = start_of_day("2026-10-24").unwrap();
        let next = start_of_day("2026-10-25").unwrap();
        assert!(next > start);
        assert_eq!(local_day(start), "2026-10-24");
        assert_eq!(local_day(next - 1), "2026-10-24");
    }
}
