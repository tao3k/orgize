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
        "rowan.build",
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
    eprintln!(
        "QUERY-SCALE complete backend={} program={} documents={count} workers={expected_workers} parity=full-projection observations={stages:?}",
        crate::runtime_backend().name(),
        crate::org_aot::org_event_parser_digest()
    );
}
