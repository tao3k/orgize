use super::{ParseConfig, parse_org_aot_batch, parse_org_aot_with_config};

pub(crate) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "native_batch_preserves_order_utf8_and_configuration",
        preserves_order_utf8_and_configuration,
    ),
    (
        "native_batch_small_index_matches_single_document_projection",
        small_index_matches_single_document_projection,
    ),
    (
        "native_batch_multiple_packets_and_oversized_source_match_single_projection",
        multiple_packets_and_oversized_source_match_single_projection,
    ),
];

fn preserves_order_utf8_and_configuration() {
    let packet = super::native_semantic_rows(
        24,
        &[
            "3", "4", "0", "0", "0", "0", "1", "invalid", "4", "0", "0", "0", "0",
        ],
    )
    .unwrap();
    assert_eq!(packet[0], ["ok", "1"]);
    assert_eq!(packet[1], ["0", "0"]);
    assert_eq!(packet[2][0], "error");
    assert_eq!(packet[3], ["ok", "1"]);
    assert_eq!(packet[4], ["0", "0"]);
    for fields in [
        &["65"][..],
        &["1", "5", "0"][..],
        &["1", "4", "0", "0", "0", "0", "trailing"][..],
    ] {
        assert!(super::native_semantic_rows(24, fields).is_err());
    }
    let inputs = [
        "",
        "* α\r\nx^{β} x^2\n",
        "| a | b |\n",
        "#+begin_src rust\nlet x = 1;\n#+end_src\n",
        "#+TODO: WAIT(w) | FINISHED(f)\n* WAIT [#A] 搜索 :tag:\n#+TITLE: λ\n",
        "* TODO duplicate\n",
    ];
    for policy in [
        crate::config::UseSubSuperscript::Nil,
        crate::config::UseSubSuperscript::Brace,
        crate::config::UseSubSuperscript::True,
    ] {
        let config = ParseConfig {
            use_sub_superscript: policy,
            inlinetask_min_level: 1,
            ..ParseConfig::default()
        };
        let results = parse_org_aot_batch(&inputs, &config).unwrap();
        assert_eq!(results.len(), inputs.len());
        #[cfg(feature = "runtime-profile")]
        {
            let (observed, observations) =
                crate::runtime_profile::measure(|| parse_org_aot_batch(&inputs, &config).unwrap());
            for (actual, expected) in observed.iter().zip(&results) {
                assert_eq!(
                    format!("{:?}", actual.as_ref().unwrap().records()),
                    format!("{:?}", expected.as_ref().unwrap().records())
                );
            }
            assert_eq!(observed.len(), results.len());
            assert!(observations["native.batch_body_wall_sum"] > 0);
            for name in [
                "native.batch_owner_thread_cpu_sum",
                "native.batch_process_cpu_interval_sum",
                "native.batch_vm_gc_cpu_interval_sum",
                "native.batch_vm_gc_wall_interval_sum",
                "native.batch_vm_gc_count_interval_sum",
            ] {
                assert!(observations.contains_key(name));
            }
            assert!(crate::runtime_profile::measure(|| ()).1.is_empty());
        }
        for (source, result) in inputs.iter().zip(results) {
            let document = result.unwrap();
            let individual = parse_org_aot_with_config(source, &config).unwrap();
            assert_eq!(document.syntax().to_string(), *source);
            assert_eq!(
                format!("{:?}", document.records()),
                format!("{:?}", individual.records())
            );
            assert_eq!(
                document.config.todo_keywords,
                individual.config.todo_keywords
            );
            for record in document.records() {
                assert_eq!(
                    document.headline_todo_type(record.id),
                    individual.headline_todo_type(record.id)
                );
                for field in ["title", "todo-keyword", "priority"] {
                    assert_eq!(
                        document.headline_derived_field(record.id, field),
                        individual.headline_derived_field(record.id, field)
                    );
                }
            }
        }
    }
    for word_count in [1024, 5000] {
        let config = ParseConfig {
            todo_keywords: (
                vec!["CONFIGURED_WAIT".into(); word_count],
                vec!["DONE".into()],
            ),
            ..ParseConfig::default()
        };
        let sources = ["* CONFIGURED_WAIT λ\n"; 8];
        let individual = parse_org_aot_with_config(sources[0], &config).unwrap();
        for document in parse_org_aot_batch(&sources, &config).unwrap() {
            let document = document.unwrap();
            assert_eq!(
                document.config.todo_keywords,
                individual.config.todo_keywords
            );
            for record in document.records() {
                assert_eq!(
                    document.headline_todo_type(record.id),
                    individual.headline_todo_type(record.id)
                );
                assert_eq!(
                    document.headline_todo_keyword(record.id),
                    individual.headline_todo_keyword(record.id)
                );
            }
        }
    }
    assert!(
        parse_org_aot_batch(&[], &ParseConfig::default())
            .unwrap()
            .is_empty()
    );
    assert!(parse_org_aot_batch(&[""; 65], &ParseConfig::default()).is_err());
    assert!(parse_org_aot_batch(&[&"x".repeat(65537)], &ParseConfig::default()).is_err());
    assert!(
        parse_org_aot_batch(&["* after rejected\n"], &ParseConfig::default()).unwrap()[0].is_ok()
    );
}

fn small_index_matches_single_document_projection() {
    crate::document::org_elements_aot_tests::batch_index_matches_individual_projection();
}

fn multiple_packets_and_oversized_source_match_single_projection() {
    crate::document::org_elements_aot_tests::batch_index_spans_bounded_packets_and_oversized_sources();
}
