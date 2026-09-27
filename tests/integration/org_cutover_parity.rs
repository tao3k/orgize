//! Single-authority admission gate for the public Scheme AOT Org parser.

use std::fs;
use std::path::{Path, PathBuf};

use orgize::Org;

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

macro_rules! assert_org_graph {
    ($source:expr, $document:expr $(, $kind:literal => $count:expr)* $(,)?) => {{
        let source: &str = ($source).as_ref();
        let document = $document;
        assert_eq!(document.to_org(), source);
        assert_eq!(document.syntax().to_string(), source);
        let records = document.records();
        for (id, record) in records.iter().enumerate() {
            assert_eq!(record.id, id, "graph record identity");
            assert!(u32::from(record.range.end()) as usize <= source.len(), "source range");
            if let Some(parent_id) = record.parent_id {
                assert!(parent_id < id, "parent precedes child");
                assert!(records[parent_id].child_ids.contains(&id), "parent-child link");
            }
            for &child_id in &record.child_ids {
                assert_eq!(records[child_id].parent_id, Some(id), "child-parent link");
            }
        }
        $(
            assert_eq!(
                records.iter().filter(|record| record.kind == $kind).count(),
                $count,
                "{} count",
                $kind,
            );
        )*
    }};
}

#[test]
fn tracked_org_fixtures_have_lossless_public_aot_graphs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let fixtures = org_fixtures(&root);
    assert!(
        fixtures.len() >= 44,
        "tracked fixture corpus unexpectedly shrank"
    );
    for path in fixtures {
        let source = fs::read_to_string(&path).expect("tracked Org fixture is UTF-8");
        assert_org_graph!(&source, Org::parse(&source));
    }
}

#[test]
fn public_parser_recognizes_case_folded_latex_environments() {
    for source in ["\\BEGIN{AlIgN*}\nx\n\\EnD{aLiGn*}\n", "\\BEGIN{A}\\eNd{a}"] {
        assert_org_graph!(source, Org::parse(source), "latex-environment" => 1);
    }
}

#[test]
fn public_parser_recognizes_case_folded_footnote_definitions() {
    let source = "[FN:n] body\n* H\n";
    assert_org_graph!(source, Org::parse(source), "footnote-definition" => 1);
}
