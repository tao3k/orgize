use std::path::Path;

use super::source_prefilter::SourcePrefilter;

#[test]
fn uses_text_terms_and_preserves_metadata_terms() {
    let text_filter = SourcePrefilter::new(&["capture init".to_string()], &[]);
    assert!(text_filter.matches_path_or_source(Path::new("notes.org"), "capture point init"));
    assert!(!text_filter.matches_path_or_source(Path::new("notes.org"), "capture only"));

    let metadata_filter = SourcePrefilter::new(&["task".to_string()], &[]);
    assert!(
        metadata_filter.matches_path_or_source(Path::new("notes.org"), ""),
        "metadata terms must not skip files before parsing"
    );

    let field_filter = SourcePrefilter::new(&[], &["key=GOVERNING_CONTRACT".to_string()]);
    assert!(
        field_filter
            .matches_path_or_source(Path::new("notes.org"), ":GOVERNING_CONTRACT: agent.plan.v1")
    );
    assert!(!field_filter.matches_path_or_source(Path::new("notes.org"), ":OTHER: value"));
}
