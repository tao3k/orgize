//! Behavioral admission cases for the public native Scheme AOT Org parser.

use orgize::Org;

#[test]
fn explicit_startup_precedes_parallel_cases() {
    assert!(Org::try_parse("* Not initialized\n").is_err());
    // SAFETY: this binary has one startup scenario; application workers and
    // Tokio runtimes are created only after the startup boundary completes.
    unsafe { orgize::initialize_native_runtime() }.expect("exclusive startup");
    unsafe { orgize::initialize_native_runtime() }.expect("idempotent startup");
    let cases: [fn(); 8] = [
        native_runtime_keeps_parse_and_contract_requests_isolated,
        native_public_parse_preserves_combined_configuration,
        native_public_parse_is_deterministic_and_serves_concurrent_callers,
        nested_headlines_and_section_boundaries,
        block_contents_do_not_start_headlines,
        headline_title_reuses_scheme_inline_objects_without_losing_title_fields,
        affiliated_keyword_and_nested_objects,
        malformed_constructs_remain_lossless,
    ];
    std::thread::scope(|scope| {
        for case in cases {
            scope.spawn(case);
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                tokio::join!(
                    tokio_stream_backpressure_and_native_parsing_coexist(),
                    tokio_bounded_callers_preserve_results_and_scheduler_progress(),
                    tokio_canceled_waiter_does_not_release_running_native_admission(),
                );
            });
    });
    eprintln!("startup-native concurrent-cases=11 complete OK");
}

async fn tokio_stream_backpressure_and_native_parsing_coexist() {
    use std::{sync::Arc, time::Duration};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        sync::Semaphore,
        task::JoinSet,
    };
    let slots = Arc::new(Semaphore::new(8));
    let mut streams = JoinSet::new();
    for index in 0..32 {
        let slots = Arc::clone(&slots);
        streams.spawn(async move {
            let source = format!("* Stream-{index}\nα [[id:{index}][证据]]\n");
            // Capacity deliberately smaller than one document: writes must
            // yield while the same single-thread scheduler reads the stream.
            let (mut writer, mut reader) = tokio::io::duplex(8);
            let expected = source.clone();
            let producing = async move {
                writer.write_all(source.as_bytes()).await.unwrap();
                writer.shutdown().await.unwrap();
            };
            let consuming = async move {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await.unwrap();
                let source = String::from_utf8(bytes).unwrap();
                let permit = slots.acquire_owned().await.unwrap();
                tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    let parsed = Org::try_parse(&source).unwrap();
                    assert_eq!(parsed.to_org(), expected);
                    assert!(!parsed.records().is_empty());
                })
                .await
                .unwrap();
            };
            tokio::join!(producing, consuming);
        });
    }
    tokio::time::timeout(Duration::from_secs(20), async {
        let mut completed = 0;
        while let Some(result) = streams.join_next().await {
            result.unwrap();
            completed += 1;
            if completed % 8 == 0 {
                eprintln!("tokio-stream completed={completed}/32");
            }
        }
        assert_eq!(completed, 32);
    })
    .await
    .expect("Tokio stream/native integration deadline");
    assert_eq!(slots.available_permits(), 8);
}

async fn tokio_bounded_callers_preserve_results_and_scheduler_progress() {
    use std::{sync::Arc, time::Duration};
    use tokio::{sync::Semaphore, task::JoinSet};
    let slots = Arc::new(Semaphore::new(8));
    let mut tasks = JoinSet::new();
    for index in 0..128 {
        let slots = Arc::clone(&slots);
        tasks.spawn(async move {
            let permit = slots.acquire_owned().await.unwrap();
            tokio::task::spawn_blocking(move || {
                // Keep admission until the real work exits, even if its async
                // waiter is canceled; blocking/native work is not abortable.
                let _permit = permit;
                let source = format!("* Tokio-{index}\nα [[id:{index}][证据]]\n");
                let parsed = Org::try_parse(&source).unwrap();
                assert_eq!(parsed.to_org(), source);
                parsed.records().len()
            })
            .await
            .unwrap()
        });
    }
    let mut heartbeat = tokio::time::interval(Duration::from_millis(1));
    let mut ticks = 0;
    let mut completed = 0;
    tokio::time::timeout(Duration::from_secs(30), async {
        while completed < 128 {
            tokio::select! {
                _ = heartbeat.tick() => ticks += 1,
                result = tasks.join_next() => {
                    assert!(result.unwrap().unwrap() > 0);
                    completed += 1;
                    if completed % 16 == 0 { eprintln!("tokio-native completed={completed}/128"); }
                }
            }
        }
    })
    .await
    .expect("bounded Tokio/native batch deadline");
    assert!(
        ticks > 1,
        "single-thread async scheduler must keep progressing"
    );
    assert_eq!(slots.available_permits(), 8);
}

async fn tokio_canceled_waiter_does_not_release_running_native_admission() {
    use std::{sync::Arc, time::Duration};
    use tokio::sync::{Semaphore, oneshot};
    let slots = Arc::new(Semaphore::new(1));
    let permit = Arc::clone(&slots).acquire_owned().await.unwrap();
    let (started, entered) = oneshot::channel();
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        started.send(()).unwrap();
        gate.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            Org::parse("* canceled-waiter\n").to_org(),
            "* canceled-waiter\n"
        );
    });
    tokio::time::timeout(Duration::from_secs(5), entered)
        .await
        .unwrap()
        .unwrap();
    task.abort(); // A running blocking task must finish, not be falsely canceled.
    assert!(slots.try_acquire().is_err());
    release.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(slots.available_permits(), 1);
    assert_eq!(Org::parse("* subsequent\n").to_org(), "* subsequent\n");
}

fn native_runtime_keeps_parse_and_contract_requests_isolated() {
    use orgize::c_ffi::{ContractInput, ContractRow, evaluate_contract};
    std::thread::scope(|scope| {
        for caller in 0..8 {
            scope.spawn(move || {
                let title = format!("Evidence-{caller}");
                for _ in 0..16 {
                    let source = format!("* {title}\nα β\n");
                    assert_eq!(Org::parse(&source).to_org(), source);
                    let result = evaluate_contract(ContractInput {
                        rows: vec![
                            ContractRow {
                                id: 0,
                                parent_id: -1,
                                kind: "org-data".into(),
                                field_name: "".into(),
                                field_value: "".into(),
                            },
                            ContractRow {
                                id: 1,
                                parent_id: 0,
                                kind: "headline".into(),
                                field_name: "title".into(),
                                field_value: title.clone(),
                            },
                        ],
                        scope_id: 0,
                        kind: "headline".into(),
                        field_name: "title".into(),
                        field_value: title.clone(),
                        expectation: 1,
                        expected_count: 1,
                    })
                    .expect("same native owner executes the contract");
                    assert_eq!(result.matched_count, 1);
                    assert!(result.passed);
                }
            });
        }
    });
}

fn native_public_parse_preserves_combined_configuration() {
    use orgize::{ParseConfig, config::UseSubSuperscript};
    let source = "*************** TODO Inline\nx^2 x^{β}\n*************** END\n";
    for (policy, superscripts) in [
        (UseSubSuperscript::Nil, 0),
        (UseSubSuperscript::Brace, 1),
        (UseSubSuperscript::True, 2),
    ] {
        let parsed = ParseConfig {
            use_sub_superscript: policy,
            ..ParseConfig::default()
        }
        .parse(source);
        assert_eq!(parsed.to_org(), source);
        assert_eq!(
            parsed
                .records()
                .iter()
                .filter(|record| record.kind == "inlinetask")
                .count(),
            1
        );
        assert_eq!(
            parsed
                .records()
                .iter()
                .filter(|record| record.kind == "superscript")
                .count(),
            superscripts
        );
    }
    let parsed = ParseConfig {
        inlinetask_min_level: 16,
        ..ParseConfig::default()
    }
    .parse(source);
    assert_eq!(
        parsed
            .records()
            .iter()
            .filter(|record| record.kind == "inlinetask")
            .count(),
        0
    );
    assert_eq!(parsed.to_org(), source);
}

fn native_public_parse_is_deterministic_and_serves_concurrent_callers() {
    let cases = [
        "",
        "* α\r\nβ *bold* [[id:x][中文]]\r\n",
        include_str!("../../benches/fixtures/doc.org"),
        include_str!("../../benches/fixtures/plain-links.org"),
        include_str!("../../benches/fixtures/quote-heavy.org"),
    ];
    let expected: Vec<_> = cases
        .iter()
        .map(|source| {
            format!(
                "{:#?}",
                Org::try_parse(source)
                    .expect("native reference parse")
                    .syntax()
            )
        })
        .collect();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let expected = &expected;
            scope.spawn(move || {
                for (source, expected) in cases.iter().zip(expected) {
                    let native = Org::try_parse(source).expect("actual linked Gerbil call");
                    assert_eq!(format!("{:#?}", native.syntax()), *expected);
                    assert_eq!(native.to_org(), *source);
                }
            });
        }
    });
}

macro_rules! assert_org_records {
    ($source:expr, $($kind:literal => $count:expr),+ $(,)?) => {{
        let source = $source;
        let parsed = Org::parse(source);
        assert_eq!(parsed.to_org(), source);
        assert_eq!(parsed.syntax().to_string(), source);
        $(
            assert_eq!(
                parsed.records().iter().filter(|record| record.kind == $kind).count(),
                $count,
                "{} count in {source:?}",
                $kind,
            );
        )+
        parsed
    }};
}

fn nested_headlines_and_section_boundaries() {
    let parsed = assert_org_records!(
        "preamble\n* Parent\nbody\n** Child\nmore\n* Sibling\n",
        "headline" => 3,
        "paragraph" => 3,
    );
    let headlines: Vec<_> = parsed
        .records()
        .iter()
        .filter(|record| record.kind == "headline")
        .collect();
    assert_eq!(headlines[0].field("title"), Some("Parent"));
    assert_eq!(headlines[1].field("title"), Some("Child"));
    assert_eq!(headlines[2].field("title"), Some("Sibling"));
    assert_eq!(headlines[1].parent_id, Some(headlines[0].id));
    assert_ne!(headlines[2].parent_id, Some(headlines[0].id));
}

fn block_contents_do_not_start_headlines() {
    assert_org_records!(
        "#+begin_src text\n* not a headline\n#+end_src\n* Real\n",
        "src-block" => 1,
        "headline" => 1,
    );
}

fn headline_title_reuses_scheme_inline_objects_without_losing_title_fields() {
    let parsed = assert_org_records!(
        "* TODO A *bold* [[id:target]] :work:\n",
        "headline" => 1,
        "bold" => 1,
        "link" => 1,
    );
    let headline = parsed.headlines().next().expect("headline view");
    assert_eq!(
        headline.display_title().as_deref(),
        Some("A *bold* [[id:target]]")
    );
    assert_eq!(headline.todo_keyword().as_deref(), Some("TODO"));
    assert_eq!(headline.local_tags().collect::<Vec<_>>(), ["work"]);
}

fn affiliated_keyword_and_nested_objects() {
    let parsed = assert_org_records!(
        "#+NAME: example\n#+CAPTION: A *bold* caption\n| a | b |\n\n* Link [[https://example.org][*bold*]]\n",
        "keyword" => 2,
        "table" => 1,
        "headline" => 1,
        "link" => 1,
        "bold" => 2,
    );
    let table = parsed
        .records()
        .iter()
        .find(|record| record.kind == "table")
        .expect("table record");
    assert_eq!(parsed.affiliated_keyword_ids(table.id).len(), 2);
}

fn malformed_constructs_remain_lossless() {
    assert_org_records!(
        "* Open\r\n#+begin_src rust\r\nlet x = 1;\r\n",
        "headline" => 1,
    );
}
