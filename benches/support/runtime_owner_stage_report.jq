# Projection of existing Rust-native diagnostic receipts, not a new sampler.
# Wall/CPU intervals overlap; residuals below are diagnostic, not causal labels.
if type != "array" or length == 0
then error("nonempty native stage receipt array required") else . end
| map(
  .stage_timings_ns as $stages
  | if .profile_enabled != true or .completed != .documents or .documents <= 0
    or any(["native.scheme_fold", "native.tape_encode", "native.scheme_thread_cpu",
            "native.owner_admission", "native.owner_service_inclusive",
            "native.owner_thread_cpu_inclusive",
            "native.result_copy", "native.completion_handoff",
            "native.transport_inclusive", "artifact.syntax_index", "graph.project",
            "supervisor.queue"][];
           ($stages[.].total | type) != "number")
    then error("complete profiled native stage receipt required") else . end
  | .documents as $count
  | ($stages["native.scheme_fold"].total / $count) as $fold
  | ($stages["native.tape_encode"].total / $count) as $tape
  | ($stages["native.scheme_thread_cpu"].total / $count) as $cpu
  | {
      backend, mode, program, documents, callers, domains, repetition,
      owner_mean_ms: {
        admission: ($stages["native.owner_admission"].total / $count / 1000000),
        service_inclusive: ($stages["native.owner_service_inclusive"].total / $count / 1000000),
        service_thread_cpu_inclusive: ($stages["native.owner_thread_cpu_inclusive"].total / $count / 1000000),
        copy: ($stages["native.result_copy"].total / $count / 1000000),
        handoff: ($stages["native.completion_handoff"].total / $count / 1000000),
        fold_wall: ($fold / 1000000), tape_wall: ($tape / 1000000),
        fold_and_tape_thread_cpu: ($cpu / 1000000),
        fold_and_tape_wall_minus_cpu: (($fold + $tape - $cpu) / 1000000)
      },
      caller_mean_ms: {
        transport_inclusive: ($stages["native.transport_inclusive"].total / $count / 1000000),
        syntax_index: ($stages["artifact.syntax_index"].total / $count / 1000000),
        graph: ($stages["graph.project"].total / $count / 1000000),
        supervisor_queue: ($stages["supervisor.queue"].total / $count / 1000000)
      },
      cpu_scope: "owner service CPU covers parse/value replies including copy/root work; fold plus tape CPU is a nested interval; neither covers unprofiled Contract C ABI",
      residual_scope: "wall minus CPU includes descheduling and waits; not exclusive attribution"
    }
)
