pub(super) fn is_daily_suffix(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let bytes = s.as_bytes();
    for (idx, ch) in bytes.iter().enumerate() {
        match idx {
            4 | 7 => {
                if *ch != b'-' {
                    return false;
                }
            }
            _ => {
                if !ch.is_ascii_digit() {
                    return false;
                }
            }
        }
    }
    true
}

pub(super) fn is_hourly_suffix(s: &str) -> bool {
    if s.len() != 13 {
        return false;
    }
    let bytes = s.as_bytes();
    for (idx, ch) in bytes.iter().enumerate() {
        match idx {
            4 | 7 | 10 => {
                if *ch != b'-' {
                    return false;
                }
            }
            _ => {
                if !ch.is_ascii_digit() {
                    return false;
                }
            }
        }
    }
    true
}
