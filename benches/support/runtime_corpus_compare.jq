# Compare only fully completed, exactly matched corpus runs.
def identity: [.documents, .callers, .repetition];
def qualified:
  length > 0 and
  all(.[]; .schema == "orgize.runtime-corpus.v1" and
    (.adapter | type == "string") and (.adapter | test("^[0-9a-f]{64}$")) and
    .documents == .distinct_documents and .documents == .completed and
    .elapsed_ns > 0 and .documents_per_second > 0) and
  ([.[] | identity] | length == (unique | length));
if length != 2 then error("expected two receipt files") else . end |
.[0] as $rust | .[1] as $scheme |
if ($rust | qualified) and ($scheme | qualified) and
   ($rust | all(.[]; .backend == "runtime-rust")) and
   ($scheme | all(.[]; .backend == "runtime-scheme")) and
   ([$rust[] | identity] | sort) == ([$scheme[] | identity] | sort)
then [
  $rust[] as $r | $scheme[] | select(identity == ($r | identity)) |
  if ([.program, .adapter, .benchmark, .architecture, .corpus, .artifact, .bytes, .driver,
       .tokio_workers, .blocking_callers_limit,
       .consumers, .queue_capacity] ==
      [$r.program, $r.adapter, $r.benchmark, $r.architecture, $r.corpus, $r.artifact,
       $r.bytes, $r.driver, $r.tokio_workers, $r.blocking_callers_limit,
       $r.consumers, $r.queue_capacity])
  then {documents, callers, repetition,
        rust_documents_per_second: $r.documents_per_second,
        scheme_documents_per_second: .documents_per_second,
        rust_p95_ms: ($r.parse_latency_ns.p95 / 1000000),
        scheme_p95_ms: (.parse_latency_ns.p95 / 1000000),
        matched: true}
  else error("source, artifact or runtime configuration mismatch") end
]
else error("incomplete, duplicate or unmatched receipt batches") end
