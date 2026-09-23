//! Focused interval-index checks for the Scheme-AOT contract executor.

use super::{descendant_intervals, in_intervals};

#[test]
fn descendant_ranges_merge_nested_targets_without_including_uncovered_targets() {
    let ends = [8, 5, 4, 4, 5, 8, 7, 8];
    let ranges = descendant_intervals(&[2, 1, 5], &ends);
    assert_eq!(ranges, vec![(2, 5), (6, 8)]);
    assert!(!in_intervals(1, &ranges));
    assert!(in_intervals(2, &ranges));
    assert!(!in_intervals(5, &ranges));
    assert!(in_intervals(7, &ranges));
}
