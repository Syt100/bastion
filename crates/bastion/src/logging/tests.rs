use super::prune::prune_rotated_log_files;
use super::suffix::{is_daily_suffix, is_hourly_suffix};
use crate::config::LogRotation;

#[test]
fn daily_suffix_parsing() {
    assert!(is_daily_suffix("2025-12-31"));
    assert!(!is_daily_suffix("20251231"));
    assert!(!is_daily_suffix("2025-1-01"));
    assert!(!is_daily_suffix("2025-12-3a"));
}

#[test]
fn hourly_suffix_parsing() {
    assert!(is_hourly_suffix("2025-12-31-23"));
    assert!(!is_hourly_suffix("2025-12-31"));
    assert!(!is_hourly_suffix("2025-12-31-2"));
    assert!(!is_hourly_suffix("2025-12-31-aa"));
}

#[test]
fn pruning_keeps_newest_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let log_file = dir.path().join("bastion.log");

    std::fs::write(dir.path().join("bastion.log.2025-01-01"), "x").unwrap();
    std::fs::write(dir.path().join("bastion.log.2025-01-02"), "x").unwrap();
    std::fs::write(dir.path().join("bastion.log.2025-01-03"), "x").unwrap();

    let pruned = prune_rotated_log_files(&log_file, LogRotation::Daily, 2).unwrap();
    assert_eq!(pruned, 1);

    assert!(dir.path().join("bastion.log.2025-01-03").exists());
    assert!(dir.path().join("bastion.log.2025-01-02").exists());
    assert!(!dir.path().join("bastion.log.2025-01-01").exists());
}

#[test]
fn pruning_only_touches_expected_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let log_file = dir.path().join("bastion.log");

    std::fs::write(dir.path().join("bastion.log.2025-01-01"), "x").unwrap();
    std::fs::write(dir.path().join("bastion.log.2025-01-02"), "x").unwrap();
    std::fs::write(dir.path().join("bastion.log.old"), "x").unwrap();

    let pruned = prune_rotated_log_files(&log_file, LogRotation::Daily, 1).unwrap();
    assert_eq!(pruned, 1);

    assert!(!dir.path().join("bastion.log.2025-01-01").exists());
    assert!(dir.path().join("bastion.log.2025-01-02").exists());
    assert!(dir.path().join("bastion.log.old").exists());
}
