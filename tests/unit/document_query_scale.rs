//! Real project-query qualification, not a replacement performance gate.
use super::elements::{filter_elements_by_query, query_project_with_config};
use super::model::{DocumentLanguage, DocumentWalkConfig};
use std::{collections::BTreeSet, fs, path::PathBuf};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned query scale fixture");
    }
}

#[test]
#[ignore = "focused native parallel project-query qualification"]
fn document_query_native_parallel_scale_qualification() {
    unsafe { crate::initialize_native_runtime() }.expect("native scale startup");
    eprintln!("QUERY-SCALE phase=startup-complete");
    let count = std::env::var("ORGIZE_QUERY_SCALE_DOCUMENTS")
        .expect("explicit document count")
        .parse::<usize>()
        .unwrap();
    assert!(matches!(count, 1000 | 10000));
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "orgize-query-scale-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    )));
    fs::create_dir(&fixture.0).unwrap();
    for index in 0..count {
        fs::write(fixture.0.join(format!("note-{index:05}.org")), format!(
            "#+TAGS: {{ @work(w) @home(h) }} [ GTD : Control Persp ]\n* Note {index}\n:PROPERTIES:\n:ID: id-{index}\n:END:\nUnique body α-{index}.\n"
        )).unwrap();
        if (index + 1) % 1000 == 0 {
            // Completed corpus writes, not a timer heartbeat. The watchdog
            // must still catch stalled startup, writes and native query work.
            eprintln!(
                "QUERY-SCALE phase=fixture-write-complete documents={count} completed={}",
                index + 1
            );
        }
    }
    assert!(!super::elements::should_index_sequentially(
        DocumentLanguage::Org,
        count,
        count as u64 * 96
    ));
    let run = || {
        query_project_with_config(
            DocumentLanguage::Org,
            &fixture.0,
            &DocumentWalkConfig::default(),
            &[],
            &[],
        )
        .expect("real parallel project query")
    };
    eprintln!("QUERY-SCALE phase=control-start documents={count}");
    // Same production query, with observation disabled; not a legacy parser.
    let control = run();
    eprintln!(
        "QUERY-SCALE phase=control-complete documents={count} facts={}",
        control.len()
    );
    let (observed, stages) = crate::runtime_profile::measure(|| {
        crate::runtime_profile::stage("query.total_inclusive", run)
    });
    assert_eq!(observed.len(), control.len());
    for (actual, expected) in observed.iter().zip(&control) {
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    }
    let paths = observed
        .iter()
        .map(|fact| fact.path.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(paths.len(), count);
    assert_eq!(
        observed
            .iter()
            .filter(|fact| fact.kind == "heading")
            .count(),
        count
    );
    let expected_workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(4)
        .min(count);
    // Count the actual chunks, not the nominal upper bound.
    let expected_workers = count.div_ceil(count.div_ceil(expected_workers));
    assert_eq!(stages["query.workers_completed"], expected_workers as u64);
    assert_eq!(stages["query.documents_completed"], count as u64);
    let chunk = count.div_ceil(expected_workers);
    let parser_packets: usize = (0..count)
        .step_by(chunk)
        .map(|start| {
            (count - start)
                .min(chunk)
                .div_ceil(crate::org_aot::BATCH_MAX_DOCUMENTS)
        })
        .sum();
    // Each small fixture packet needs one event request and one bounded
    // metadata-plan request, not a second request for every document.
    let expected_requests = (2 * parser_packets) as u64;
    assert_eq!(
        stages["native.requests_completed"], expected_requests,
        "one event packet plus one metadata packet; no per-document or keyword re-entry"
    );
    for name in [
        "query.total_inclusive",
        "query.worker_wall_sum",
        "native.owner_admission",
        "native.owner_service_inclusive",
        "native.scheme_fold",
        "native.scheme_thread_cpu",
        "native.batch_body_wall_sum",
        "native.batch_owner_thread_cpu_sum",
        "native.batch_process_cpu_interval_sum",
        "native.batch_vm_gc_cpu_interval_sum",
        "native.batch_vm_gc_wall_interval_sum",
        "native.batch_vm_gc_count_interval_sum",
        "native.tape_encode",
        "native.event_decode",
        "artifact.syntax_index",
        "graph.project",
    ] {
        assert!(stages.contains_key(name), "missing worker stage {name}");
    }
    assert!(
        filter_elements_by_query(
            observed,
            &["document_query_absent_fixture".to_string()],
            &[],
            &[]
        )
        .is_empty()
    );
    assert!(crate::runtime_profile::measure(|| ()).1.is_empty());
    // Matched native full-projection oracle, including hit and no-hit queries.
    // These cold queries still parse every source and retain the same workers.
    for (label, terms, fields) in [
        (
            "no-hit",
            vec!["document_query_absent_fixture".to_string()],
            vec![],
        ),
        ("hit", vec![], vec!["title=Note".to_string()]),
    ] {
        eprintln!("QUERY-SCALE phase=filtered-start query={label} documents={count}");
        let expected = filter_elements_by_query(control.clone(), &terms, &[], &fields);
        let (actual, filtered_stages) = crate::runtime_profile::measure(|| {
            crate::runtime_profile::stage("query.total_inclusive", || {
                query_project_with_config(
                    DocumentLanguage::Org,
                    &fixture.0,
                    &DocumentWalkConfig::default(),
                    &terms,
                    &fields,
                )
                .unwrap()
            })
        });
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
        assert_eq!(actual.len(), if label == "hit" { count } else { 0 });
        assert_eq!(filtered_stages["query.documents_completed"], count as u64);
        assert_eq!(
            filtered_stages["query.workers_completed"],
            expected_workers as u64
        );
        assert_eq!(
            filtered_stages["native.requests_completed"], expected_requests,
            "filtered queries retain the same bounded native handoff"
        );
        assert!(filtered_stages.contains_key("query.fact_projection"));
        eprintln!(
            "QUERY-SCALE filtered-complete query={label} documents={count} results={} parity=full-projection observations={filtered_stages:?}",
            actual.len()
        );
    }
    eprintln!(
        "QUERY-SCALE complete backend={} program={} documents={count} workers={expected_workers} parity=full-projection observations={stages:?}",
        crate::runtime_backend().name(),
        crate::org_aot::org_event_parser_digest()
    );
}
