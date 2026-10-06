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
    let inputs = [
        "",
        "* α\r\nx^{β} x^2\n",
        "| a | b |\n",
        "#+begin_src rust\nlet x = 1;\n#+end_src\n",
        "* duplicate\n",
        "* duplicate\n",
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
