use std::time::Duration;

use url::Url;

pub(super) fn normalize_base_url(raw: &str) -> Result<Url, anyhow::Error> {
    let mut url = Url::parse(raw)?;
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

pub(super) fn agent_ws_url(base_url: &Url) -> Result<Url, anyhow::Error> {
    let mut url = base_url.clone();
    match url.scheme() {
        "http" => url.set_scheme("ws").ok(),
        "https" => url.set_scheme("wss").ok(),
        other => anyhow::bail!("unsupported hub_url scheme: {other}"),
    };
    Ok(url.join("agent/ws")?)
}

pub(super) fn jittered_backoff(base: Duration, agent_id: &str, attempt: u32) -> Duration {
    if base.is_zero() {
        return base;
    }

    // Equal-jitter backoff: [base/2, base], deterministic per agent+attempt.
    let half = base / 2;
    let half_ms = half.as_millis().min(u128::from(u64::MAX)) as u64;
    if half_ms == 0 {
        return base;
    }

    let seed = fnv1a64(agent_id.as_bytes())
        .wrapping_add(u64::from(attempt).wrapping_mul(0x9e3779b97f4a7c15));
    let jitter_ms = seed % (half_ms + 1);
    half + Duration::from_millis(jitter_ms)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub(super) fn is_ws_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|e| e.is::<tokio_tungstenite::tungstenite::Error>())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn normalize_base_url_appends_slash() {
        let url = super::normalize_base_url("http://localhost:9876").unwrap();
        assert_eq!(url.as_str(), "http://localhost:9876/");
    }

    #[test]
    fn agent_ws_url_converts_scheme() {
        let base = super::normalize_base_url("https://hub.example.com/bastion").unwrap();
        let ws = super::agent_ws_url(&base).unwrap();
        assert_eq!(ws.as_str(), "wss://hub.example.com/bastion/agent/ws");
    }

    #[test]
    fn jittered_backoff_is_deterministic_and_in_range() {
        let base = Duration::from_secs(10);
        let a = super::jittered_backoff(base, "agent-1", 1);
        let b = super::jittered_backoff(base, "agent-1", 1);
        assert_eq!(a, b);
        assert!(a >= base / 2);
        assert!(a <= base);
    }

    #[test]
    fn jittered_backoff_returns_base_when_half_ms_is_zero() {
        let base = Duration::from_millis(1);
        assert_eq!(super::jittered_backoff(base, "agent-1", 1), base);
    }

    #[test]
    fn jittered_backoff_zero_base_is_zero() {
        assert_eq!(
            super::jittered_backoff(Duration::ZERO, "agent-1", 1),
            Duration::ZERO
        );
    }

    #[test]
    fn is_ws_error_detects_tungstenite_errors_in_chain() {
        let err = anyhow::Error::new(tokio_tungstenite::tungstenite::Error::ConnectionClosed);
        assert!(super::is_ws_error(&err));

        let err = anyhow::anyhow!("boom");
        assert!(!super::is_ws_error(&err));
    }
}
