use super::{measure, stage};

#[test]
fn worker_observations_merge_only_after_join() {
    let (values, stages) = measure(|| {
        std::thread::scope(|scope| {
            let children = [7, 11].map(|value| {
                scope.spawn(move || {
                    super::worker(true, || {
                        super::collector::record("worker.test", value);
                        value
                    })
                })
            });
            assert!(super::is_active());
            children.map(|child| child.join().unwrap().receive())
        })
    });
    assert_eq!(values, [7, 11]);
    assert_eq!(stages["worker.test"], 18);
    assert_eq!(stages["query.workers_completed"], 2);
    assert!(stages.contains_key("query.worker_wall_sum"));
    assert!(measure(|| ()).1.is_empty());
    let (_, stages) = measure(|| {
        std::thread::spawn(|| {
            super::worker(false, || {
                assert!(!super::is_active());
            })
        })
        .join()
        .unwrap()
        .receive()
    });
    assert!(stages.is_empty());
}

#[test]
fn worker_observations_do_not_leak_between_requests_or_after_panic() {
    let children = [19, 23].map(|value| {
        std::thread::spawn(move || {
            measure(|| {
                std::thread::spawn(move || {
                    super::worker(true, || {
                        super::collector::record("worker.test", value);
                        Err::<(), _>(value)
                    })
                })
                .join()
                .unwrap()
                .receive()
            })
        })
    });
    for (child, expected) in children.into_iter().zip([19, 23]) {
        let (result, stages) = child.join().unwrap();
        assert_eq!(result, Err(expected));
        assert_eq!(stages["worker.test"], expected);
    }
    std::thread::spawn(|| {
        assert!(
            std::panic::catch_unwind(|| super::worker(true, || panic!("worker injection")))
                .is_err()
        );
        assert!(measure(|| ()).1.is_empty());
    })
    .join()
    .unwrap();
    assert!(measure(|| ()).1.is_empty());
}

#[cfg(not(feature = "runtime-scheme"))]
#[test]
fn owner_reply_records_only_on_the_receiving_request() {
    let (_, timings) = measure(|| {
        let submitted = super::Stamp::start();
        let reply = std::thread::spawn(move || {
            assert!(!super::is_active());
            super::OwnerReply::new(7, [11, 13, 17], submitted.child())
        })
        .join()
        .unwrap();
        assert_eq!(reply.receive(), 7);
    });
    assert_eq!(timings["native.owner_admission"], 11);
    assert_eq!(timings["native.owner_service_inclusive"], 13);
    assert_eq!(timings["native.result_copy"], 17);
    assert!(timings.contains_key("native.completion_handoff"));
    assert!(measure(|| ()).1.is_empty());
}

#[test]
fn transport_projection_retains_exact_request_local_values() {
    let (_, timings) = measure(|| super::record_transport([11, 13, 17], 19));
    assert_eq!(timings["native.owner_admission"], 11);
    assert_eq!(timings["native.owner_service_inclusive"], 13);
    assert_eq!(timings["native.result_copy"], 17);
    assert_eq!(timings["native.completion_handoff"], 19);
    super::record_transport([1, 2, 3], 4);
    assert!(measure(|| ()).1.is_empty());
}

#[test]
fn native_envelope_rejects_truncation_and_foreign_identity() {
    let mut bytes = b"OPR1".to_vec();
    bytes.extend_from_slice(&7_u64.to_le_bytes());
    bytes.extend_from_slice(&11_u64.to_le_bytes());
    bytes.extend_from_slice(&13_u64.to_le_bytes());
    bytes.extend_from_slice(b"OEV1payload");
    for length in 0..32 {
        assert!(super::wire::decode(&bytes[..length]).is_err());
    }
    assert_eq!(
        super::wire::decode(&bytes).unwrap(),
        (7, 11, 13, &bytes[28..])
    );
    let (_, timings) = measure(|| super::native_tape(&bytes).unwrap());
    assert_eq!(timings["native.scheme_fold"], 7);
    assert_eq!(timings["native.tape_encode"], 11);
    assert_eq!(timings["native.scheme_thread_cpu"], 13);
    bytes[0] = b'X';
    assert!(super::wire::decode(&bytes).is_err());
    bytes[0] = b'O';
    bytes[28] = b'X';
    assert!(super::wire::decode(&bytes).is_err());
}

#[test]
fn request_local_timings_are_isolated_and_reset_on_panic() {
    let (_, first) = measure(|| stage("test", || 7));
    assert_eq!(first.len(), 1);
    assert!(measure(|| ()).1.is_empty());
    assert!(std::panic::catch_unwind(|| measure(|| panic!("injected"))).is_err());
    assert!(measure(|| ()).1.is_empty());
    let (_, outer) = measure(|| {
        assert!(std::panic::catch_unwind(|| measure(|| ())).is_err());
        stage("outer", || ());
        std::thread::spawn(|| assert!(measure(|| ()).1.is_empty()))
            .join()
            .unwrap();
    });
    assert_eq!(outer.len(), 1);
    assert!(outer.contains_key("outer"));
}
