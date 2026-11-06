def key: [.documents, .callers, .domains, .repetition];
def qualified:
  length > 0 and all(.[];
    (.schema == "orgize.runtime-parallel.v1" or .schema == "orgize.runtime-parallel.v2") and .driver == "tokio-process-domains" and
    (if .schema == "orgize.runtime-parallel.v2" then
      (.mode == "qualification" and (.artifact | type) == "string" or
       .mode == "performance" and .artifact == null) else true end) and
    (.adapter | test("^[0-9a-f]{64}$")) and (.benchmark | test("^[0-9a-f]{64}$")) and
    .documents == .distinct_documents and .documents == .completed and
    .elapsed_ns > 0 and .documents_per_second > 0 and
    .max_in_flight == .callers and .max_active_worker_requests == .domains and
    (.worker_pids | unique | length) == .domains and
    (.worker_completed | length) == .domains and (.worker_completed | add) == .completed and
    all(.worker_completed[]; . > 0) and .worker_exit_success == true) and
  (map(key) | length == (unique | length));
def identity: [.schema, .driver, .program, .adapter, .benchmark, .architecture,
  .corpus, .artifact, .documents, .distinct_documents, .completed, .bytes,
  .callers, .domains, .repetition, .tokio_workers, .queue_capacity_per_domain,
  .native_queue_capacity_per_domain, .end_to_end_scope, .workload,
  .mode, .profile_enabled, .native_stage_scope];
if length != 2 then error("two feature receipts required") else . end
| (.[0] | sort_by(key)) as $rust
| (.[1] | sort_by(key)) as $scheme
| if ($rust | qualified | not) or ($scheme | qualified | not) or ($rust | length) != ($scheme | length)
  then error("incomplete/mismatched matrices") else . end
| [range(0; $rust | length) as $i
   | $rust[$i] as $r | $scheme[$i] as $s
   | if $r.backend != "runtime-rust" or $s.backend != "runtime-scheme"
       or ($r | identity) != ($s | identity)
       or $r.completed != $r.documents or $s.completed != $s.documents
       or $r.max_in_flight != $r.callers or $s.max_in_flight != $s.callers
       or $r.max_active_worker_requests != $r.domains or $s.max_active_worker_requests != $s.domains
       or $r.worker_exit_success != true or $s.worker_exit_success != true
     then error("feature/identity/completion/concurrency gate failed")
     else {documents: $r.documents, callers: $r.callers, domains: $r.domains,
       repetition: $r.repetition, rust_documents_per_second: $r.documents_per_second,
       scheme_documents_per_second: $s.documents_per_second,
       rust_over_scheme_throughput: ($r.documents_per_second / $s.documents_per_second),
       rust_p95_ms: ($r.end_to_end_latency_ns.p95 / 1000000),
       scheme_p95_ms: ($s.end_to_end_latency_ns.p95 / 1000000)} end]
