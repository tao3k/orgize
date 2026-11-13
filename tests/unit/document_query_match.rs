use super::{FactView, PreparedQuery, PreparedTerm, contains_ascii_folded};

#[test]
fn ascii_search_matches_allocating_reference_at_utf8_boundaries() {
    let values = [
        "",
        "ABC abc AbC",
        "α ABC Ä ä",
        "Straße STRASSE",
        " a\0B ",
        "éAé",
        "aaaaABaaaa",
    ];
    let needles = [
        "", "a", "AB", "abc", "α", "Ä", "ä", "straße", "STRASSE", "\0b", "éa", "absent", "aaaaaa",
    ];
    for value in values {
        for needle in needles {
            let normalized = needle.to_ascii_lowercase();
            assert_eq!(
                contains_ascii_folded(value, &PreparedTerm::new(needle)),
                value.to_ascii_lowercase().contains(&normalized),
                "value={value:?} needle={needle:?}",
            );
        }
    }
    for first in 0_u8..=127 {
        for second in 0_u8..=127 {
            let value = String::from_utf8(vec![first, second]).unwrap();
            let needle = value.to_ascii_lowercase();
            assert!(contains_ascii_folded(&value, &PreparedTerm::new(&needle)));
        }
    }
    let repeated = "a".repeat(65536);
    let needle = format!("{}B", "a".repeat(1024));
    let term = PreparedTerm::new(&needle);
    assert!(!contains_ascii_folded(&repeated, &term));
    assert!(contains_ascii_folded(&format!("{repeated}b"), &term));
}

#[test]
fn prepared_predicates_preserve_field_and_term_boundaries() {
    let fields = vec![
        ("Title".into(), "Alpha=BETA".into()),
        ("todoType".into(), "Done".into()),
    ];
    let view = || FactView {
        kind: "heading",
        source_kind: "Headline",
        path: "Mixed/File.org",
        text: "Visible TITLE α",
        content: "body Ä",
        fields: &fields,
    };
    for (terms, kinds, predicates, expected) in [
        (
            vec![" TITLE\tα ".into(), "body DONE".into()],
            vec![" HEADING ".into()],
            vec!["title=Alpha=BETA".into()],
            true,
        ),
        (vec![], vec![], vec!["title=alpha".into()], false),
        (vec![], vec![], vec![" TEXT = Visible ".into()], true),
        (vec![], vec![], vec!["text".into()], false),
        (vec![], vec![], vec!["".into(), " Title ".into()], true),
        (vec!["Ä".into()], vec![], vec![], true),
        (vec!["ä".into()], vec![], vec![], false),
        (vec!["αbody".into()], vec![], vec![], false),
        (vec![], vec!["heading".into(), "task".into()], vec![], false),
    ] {
        assert_eq!(
            PreparedQuery::new(&terms, &kinds, &predicates).matches_view(view()),
            expected
        );
    }
}
