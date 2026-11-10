use super::{spawn_runtime, terminate_and_reap};
#[path = "../support/child_cleanup.rs"]
mod child_cleanup;
use child_cleanup::Children;
use std::{
    os::unix::process::{CommandExt, ExitStatusExt},
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[test]
fn timeout_cleanup_kills_another_live_group_member() {
    let mut leader = Command::new("sleep");
    leader.arg("30");
    let mut children = Children(vec![spawn_runtime(leader, "sleep").expect("group leader")]);
    let group = i32::try_from(children.0[0].id()).unwrap();
    let peer = Command::new("sleep")
        .arg("30")
        .process_group(group)
        .spawn()
        .expect("join the test process group");
    let peer_pid = i32::try_from(peer.id()).unwrap();
    children.0.push(peer);
    // SAFETY: the positive PID belongs to the peer retained in children.
    assert_eq!(unsafe { libc::getpgid(peer_pid) }, group);
    assert!(children.0[1].try_wait().unwrap().is_none());
    assert!(terminate_and_reap(&mut children.0[0]));

    // Own and reap the peer too; no orphan or PID file is needed.
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(status) = children.0[1].try_wait().unwrap() {
            assert_eq!(status.signal(), Some(libc::SIGKILL));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "process group peer survived cleanup"
        );
        thread::sleep(Duration::from_millis(1));
    }
}
