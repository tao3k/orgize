//! Differential admission gate for the public parser and transitional AOT path.

use std::fs;
use std::path::{Path, PathBuf};

use orgize::{Org, SyntaxKind, org_aot::parse_org_aot, rowan::ast::AstNode};

fn org_fixtures(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut fixtures = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).expect("tracked fixture directory") {
            let path = entry.expect("tracked fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "org") {
                fixtures.push(path);
            }
        }
    }
    fixtures.sort();
    fixtures
}

macro_rules! check_org_cutover_parity {
    ($path:expr, $source:expr) => {{
        let source = $source;
        let baseline = Org::parse(source);
        let aot = parse_org_aot(source).expect("Scheme AOT parses tracked Org fixture");
        assert_eq!(baseline.to_org(), source.as_str(), "{}", $path.display());
        assert_eq!(
            aot.syntax().to_string(),
            source.as_str(),
            "{}",
            $path.display()
        );
        let baseline_headlines = baseline
            .syntax_document()
            .syntax()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::HEADLINE)
            .count();
        let aot_headlines = aot
            .records()
            .iter()
            .filter(|record| record.kind == "headline")
            .count();
        assert_eq!(aot_headlines, baseline_headlines, "{}", $path.display());
        let baseline_headline_ranges: Vec<_> = baseline
            .syntax_document()
            .syntax()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::HEADLINE)
            .map(|node| node.text_range())
            .collect();
        let aot_headline_ranges: Vec<_> = aot
            .syntax()
            .descendants()
            .filter(|node| {
                orgize::org_aot::org_language_spec().kinds[usize::from(node.kind().0)].name
                    == "OrgSection"
            })
            .map(|node| node.text_range())
            .collect();
        assert_eq!(
            aot_headline_ranges,
            baseline_headline_ranges,
            "{}: headline ranges",
            $path.display()
        );
        let baseline_headline_depths: Vec<_> = baseline
            .syntax_document()
            .syntax()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::HEADLINE)
            .map(|node| {
                node.ancestors()
                    .filter(|ancestor| ancestor.kind() == SyntaxKind::HEADLINE)
                    .count()
            })
            .collect();
        let aot_headline_depths: Vec<_> = aot
            .syntax()
            .descendants()
            .filter(|node| {
                orgize::org_aot::org_language_spec().kinds[usize::from(node.kind().0)].name
                    == "OrgSection"
            })
            .map(|node| {
                node.ancestors()
                    .filter(|ancestor| {
                        orgize::org_aot::org_language_spec().kinds[usize::from(ancestor.kind().0)]
                            .name
                            == "OrgSection"
                    })
                    .count()
            })
            .collect();
        assert_eq!(
            aot_headline_depths,
            baseline_headline_depths,
            "{}: headline nesting",
            $path.display()
        );
        for (baseline_kind, aot_kind) in [
            (SyntaxKind::PROPERTY_DRAWER, "property-drawer"),
            (SyntaxKind::DRAWER, "drawer"),
            (SyntaxKind::LIST, "plain-list"),
            (SyntaxKind::LIST_ITEM, "item"),
            (SyntaxKind::ORG_TABLE, "table"),
            (SyntaxKind::SOURCE_BLOCK, "src-block"),
            (SyntaxKind::DYN_BLOCK, "dynamic-block"),
            (SyntaxKind::BABEL_CALL, "babel-call"),
            (SyntaxKind::QUOTE_BLOCK, "quote-block"),
            (SyntaxKind::EXAMPLE_BLOCK, "example-block"),
            (SyntaxKind::EXPORT_BLOCK, "export-block"),
        ] {
            let expected = baseline
                .syntax_document()
                .syntax()
                .descendants()
                .filter(|node| node.kind() == baseline_kind)
                .count();
            let actual = aot
                .records()
                .iter()
                .filter(|record| record.kind == aot_kind)
                .count();
            assert_eq!(actual, expected, "{}: {aot_kind}", $path.display());
        }
        let baseline_keywords = baseline
            .syntax_document()
            .syntax()
            .descendants()
            .filter(|node| {
                matches!(
                    node.kind(),
                    SyntaxKind::KEYWORD | SyntaxKind::AFFILIATED_KEYWORD
                )
            })
            .count();
        let aot_keywords = aot
            .records()
            .iter()
            .filter(|record| record.kind == "keyword")
            .count();
        assert_eq!(
            aot_keywords,
            baseline_keywords,
            "{}: keyword lines",
            $path.display()
        );
    }};
}

#[test]
fn tracked_org_fixtures_match_core_structure_and_source_ranges() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let fixtures = org_fixtures(&root);
    assert!(
        fixtures.len() >= 44,
        "tracked fixture corpus unexpectedly shrank"
    );
    for path in fixtures {
        let source = fs::read_to_string(&path).expect("tracked Org fixture is UTF-8");
        check_org_cutover_parity!(path, &source);
    }
}
