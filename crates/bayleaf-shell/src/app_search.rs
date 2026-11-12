//! Matching and ordering for the launcher, independent of GIO so it can be unit-tested.

/// Text an app can be found by: name, executable, description — lower-cased once.
pub fn haystack(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.to_lowercase())
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

/// Every whitespace-separated term of `query` must occur in `haystack` (case-insensitive).
/// An empty query matches everything.
pub fn matches(haystack: &str, query: &str) -> bool {
    query
        .to_lowercase()
        .split_whitespace()
        .all(|term| haystack.contains(term))
}

/// Key for alphabetical ordering that ignores case.
pub fn sort_key(name: &str) -> String {
    name.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_matches_all() {
        assert!(matches(&haystack(&["Firefox"]), ""));
        assert!(matches(&haystack(&["Firefox"]), "   "));
    }

    #[test]
    fn matches_any_part_case_insensitive() {
        let h = haystack(&["Dolphin", "dolphin", "File Manager"]);
        assert!(matches(&h, "DOL"));
        assert!(matches(&h, "file"));
        assert!(!matches(&h, "konsole"));
    }

    #[test]
    fn all_terms_must_match() {
        let h = haystack(&["System Monitor", "plasma-systemmonitor"]);
        assert!(matches(&h, "sys mon"));
        assert!(!matches(&h, "sys editor"));
    }

    #[test]
    fn terms_do_not_match_across_parts() {
        let h = haystack(&["Kate", "kate"]);
        assert!(!matches(&h, "katekate"));
    }

    #[test]
    fn sort_ignores_case() {
        let mut names = vec!["dolphin", "Firefox", "ark"];
        names.sort_by_key(|n| sort_key(n));
        assert_eq!(names, ["ark", "dolphin", "Firefox"]);
    }
}
