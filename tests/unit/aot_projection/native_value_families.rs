//! Complete native value-family plans, framing and large admitted batches.
use crate::config::ParseConfig;
use crate::org_aot::native_semantic_rows as rows;

pub(super) fn native_value_families_drive_public_consumers() {
    let org = ParseConfig::default()
        .parse("#+seq_todo: WAIT(w) | DONE(d)\n* WAIT [#A] COMMENT λ :work:\n");
    let doc = org.document();
    assert_eq!(doc.children.len(), 1);
    assert_eq!(
        rows(16, &["planning-key-kind", "scheduled"]).unwrap(),
        [vec!["scheduled"]]
    );
    assert_eq!(
        rows(
            16,
            &["memory-headline-state", "todo", "false", "false", "true"]
        )
        .unwrap(),
        [vec!["archived"]]
    );
    assert_eq!(
        rows(
            18,
            &[
                "file:λ.org::*Heading",
                "id:local::#target",
                "attachment:doc.pdf::12"
            ]
        )
        .unwrap()[0][4],
        "λ.org"
    );
    assert_eq!(
        rows(19, &["1", "NAME", "3", "name", "ATTR_HTML", "TODO"]).unwrap(),
        [vec!["true"], vec!["true"], vec!["false"]]
    );
    let log = rows(
        20,
        &["- State \"DONE\" from \"WAIT\"\r\nCLOCK: => 1:02\r\n"],
    )
    .unwrap();
    assert_eq!(&log[0][4..8], ["state", "complete", "DONE", "WAIT"]);
    assert_eq!(&log[1][8..], ["present", "1:02"]);
    assert_eq!(
        rows(21, &[" <l10> ", "<64>", "<9223372036854775808>", "<c>"]).unwrap(),
        [vec!["left"], vec!["width"], vec![""], vec!["center"]]
    );
    assert_eq!(
        rows(
            22,
            &[
                "é Alpha Beta Alpha Alphabet Alpha-Beta",
                "Alpha",
                "Alpha Beta",
                "Alpha"
            ]
        )
        .unwrap(),
        [vec!["3", "13", "1"], vec!["14", "19", "0"]]
    );
}

pub(super) fn native_value_batches_handle_ten_thousand_entries() {
    for count in [1_000usize, 10_000] {
        let mut fields = vec![
            "1".to_owned(),
            "TODO".into(),
            "1".into(),
            "DONE".into(),
            "0".into(),
            (count * 3).to_string(),
        ];
        for _ in 0..count {
            fields.extend([
                "TODO [#A] λ :work:".into(),
                "TODO [#A] λ ".into(),
                "true".into(),
            ]);
        }
        let refs = fields.iter().map(String::as_str).collect::<Vec<_>>();
        let values = rows(17, &refs).unwrap();
        assert_eq!(values.len(), count + 1);
        assert_eq!(
            &values[count][..],
            ["todo", "TODO", "[#A] λ :work:", "A", "λ", "λ ", "false"]
        );
        let paths = vec!["file:λ.org::*Heading"; count];
        assert_eq!(rows(18, &paths).unwrap().len(), count);
        let text = "é Alpha ".repeat(count);
        let matches = rows(22, &[&text, "Alpha"]).unwrap();
        assert_eq!(matches.len(), count);
        assert_eq!(
            matches[count - 1][0].parse::<usize>().unwrap(),
            (count - 1) * "é Alpha ".len() + 3
        );
    }
}

pub(super) fn native_value_framing_rejects_without_poisoning_owner() {
    for fields in [
        vec!["missing"],
        vec!["headline-display-title", "λ", "maybe"],
        vec!["todo-open-words"],
        vec!["org-affiliated-keyword?", "NAME", "-1"],
        vec!["planning-key-kind", "CLOSED", "extra"],
    ] {
        assert!(rows(16, &fields).is_err());
    }
    // Counted fields retain embedded NUL; it is not a C-string terminator.
    assert_eq!(
        rows(16, &["planning-key-kind", "CLO\0SED"]).unwrap(),
        [vec![""]]
    );
    assert!(rows(17, &["-1"]).is_err());
    assert!(rows(19, &["1", "NAME", "2", "NAME"]).is_err());
    assert_eq!(
        rows(16, &["planning-key-kind", "CLOSED"]).unwrap(),
        [vec!["closed"]]
    );
}
