pub(super) fn normalize_cron(expr: &str) -> Result<String, anyhow::Error> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => Ok(format!("0 {}", parts.join(" "))),
        6 => {
            if parts[0] != "0" {
                anyhow::bail!("cron seconds must be 0 for minute-based scheduling");
            }
            Ok(parts.join(" "))
        }
        _ => Err(anyhow::anyhow!("invalid cron expression")),
    }
}

fn parse_cron_cached<'a>(
    expr: &str,
    schedule_cache: &'a mut std::collections::HashMap<String, cron::Schedule>,
) -> Result<&'a cron::Schedule, anyhow::Error> {
    use std::str::FromStr as _;

    let expr = normalize_cron(expr)?;
    if !schedule_cache.contains_key(&expr) {
        let schedule = cron::Schedule::from_str(&expr)?;
        schedule_cache.insert(expr.clone(), schedule);
    }
    Ok(schedule_cache
        .get(&expr)
        .expect("schedule_cache contains key we just inserted"))
}

pub(super) fn cron_matches_minute_cached(
    expr: &str,
    minute_start: chrono::DateTime<impl chrono::TimeZone>,
    schedule_cache: &mut std::collections::HashMap<String, cron::Schedule>,
) -> Result<bool, anyhow::Error> {
    use chrono::Duration as ChronoDuration;

    let schedule = parse_cron_cached(expr, schedule_cache)?;
    let prev = minute_start.clone() - ChronoDuration::seconds(1);
    let mut iter = schedule.after(&prev);
    let Some(next) = iter.next() else {
        return Ok(false);
    };
    Ok(next == minute_start)
}

#[cfg(test)]
mod tests {
    use super::{cron_matches_minute_cached, normalize_cron};

    #[test]
    fn normalize_cron_accepts_5_or_6_fields() {
        assert_eq!(normalize_cron("*/5 * * * *").unwrap(), "0 */5 * * * *");
        assert_eq!(normalize_cron("0 */6 * * * *").unwrap(), "0 */6 * * * *");
    }

    #[test]
    fn normalize_cron_rejects_other_field_counts() {
        assert!(normalize_cron("").is_err());
        assert!(normalize_cron("* * * *").is_err());
        assert!(normalize_cron("* * * * * * *").is_err());
    }

    #[test]
    fn cron_matches_minute_cached_matches_expected_minutes() {
        use chrono::TimeZone as _;

        let minute_start = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 10, 0).unwrap();
        let mut cache = std::collections::HashMap::new();

        assert!(cron_matches_minute_cached("*/5 * * * *", minute_start, &mut cache).unwrap());
        assert!(cron_matches_minute_cached("0 */5 * * * *", minute_start, &mut cache).unwrap());
        assert!(!cron_matches_minute_cached("*/7 * * * *", minute_start, &mut cache).unwrap());
    }
}
