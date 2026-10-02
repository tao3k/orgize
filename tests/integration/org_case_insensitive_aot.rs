//! ASCII case-insensitive Org structure remains Scheme-owned after AOT.

#[test]
fn org_structural_syntax_accepts_mixed_ascii_case_without_changing_source() {
    let source = "#+sEq_ToDo: WAIT | DONE\n\
                  * WAIT Parent\n\
                  sChEdUlEd: <2026-09-27 Sun>\n\
                  :pRoPeRtIeS:\n\
                  :ID: parent\n\
                  :eNd:\n\
                  #+BeGiN_SrC rust\n\
                  ** Not a headline\n\
                  #+eNd_sRc\n\
                  ** DONE Child\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("mixed-case Org syntax");
    assert!(orgize::org_aot::org_language_spec().case_insensitive);
    assert_eq!(document.syntax().to_string(), source);
    let kinds = document
        .records()
        .iter()
        .map(|record| record.kind)
        .collect::<Vec<_>>();
    assert_eq!(kinds.iter().filter(|&&kind| kind == "headline").count(), 2);
    assert!(kinds.contains(&"src-block"));
    assert!(kinds.contains(&"property-drawer"));
    assert!(kinds.contains(&"planning"));
    let headlines = document.headlines().collect::<Vec<_>>();
    assert_eq!(headlines[0].todo_keyword().as_deref(), Some("WAIT"));
    assert_eq!(headlines[1].todo_keyword().as_deref(), Some("DONE"));
}
