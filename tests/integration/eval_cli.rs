use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn eval_cli_renders_named_block_plan_without_running_code() {
    let dir = test_dir("eval-plan");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("notes.org"), eval_fixture()).unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args(["eval", "plan", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("orgize eval plan"), "stdout: {stdout}");
    assert!(stdout.contains("name: verify"), "stdout: {stdout}");
    assert!(stdout.contains("language: bash"), "stdout: {stdout}");
    assert!(
        stdout.contains("results: output replace"),
        "stdout: {stdout}"
    );
}

#[test]
fn eval_cli_plan_projects_builtin_typst_runtime_without_header_args() {
    let dir = test_dir("eval-plan-typst-runtime");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("notes.org"),
        r#"#+NAME: verify
#+BEGIN_SRC typst
"ok"
#+END_SRC
"#,
    )
    .unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args(["eval", "plan", "--json", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_success(&output);
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["runtime"]["id"], "typst");
    assert_eq!(plan["runtime"]["source"], "default");
    assert_eq!(plan["runtime"]["registered"], true);
    assert_eq!(plan["runtime"]["program"], "typst");
    assert_eq!(plan["runtime"]["bodyMode"], "stdin");
    assert_eq!(
        plan["runtime"]["args"],
        serde_json::json!(["compile", "--format", "svg", "-", "-"])
    );
}

#[test]
fn eval_cli_patch_writes_results_without_executing_code() {
    let dir = test_dir("eval-patch-write");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("notes.org");
    fs::write(&path, eval_fixture()).unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args([
            "eval",
            "patch",
            "--write",
            "--stdout",
            "ok",
            "--exit-code",
            "0",
            "verify",
            "notes.org",
        ])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("kind: insert"), "stdout: {stdout}");
    assert!(stdout.contains("written: true"), "stdout: {stdout}");
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo should-not-run
#+END_SRC

#+RESULTS: verify
: ok
"#
    );
}

#[test]
fn eval_cli_run_executes_shell_block_and_writes_results() {
    let dir = test_dir("eval-run-write");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("notes.org");
    fs::write(
        &path,
        r#"#+NAME: verify
#+BEGIN_SRC sh :results output replace
printf real-eval
#+END_SRC
"#,
    )
    .unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args(["eval", "run", "--write", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("orgize eval run"), "stdout: {stdout}");
    assert!(stdout.contains("exit-code: 0"), "stdout: {stdout}");
    assert!(stdout.contains("written: true"), "stdout: {stdout}");
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        r#"#+NAME: verify
#+BEGIN_SRC sh :results output replace
printf real-eval
#+END_SRC

#+RESULTS: verify
: real-eval
"#
    );
}

#[cfg(unix)]
#[test]
fn eval_cli_run_executes_typst_with_compile_stdin() {
    let dir = test_dir("eval-run-typst");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(
        dir.join("notes.org"),
        r#"#+NAME: verify
#+BEGIN_SRC typst :results output replace
#let answer = 42
#+END_SRC
"#,
    )
    .unwrap();
    let typst = bin.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
printf '%s\n' "$@" > typst-args.txt
cat > typst-stdin.txt
printf '"ok"\n'
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let path = format!("{}:{}", bin.display(), env::var("PATH").unwrap_or_default());

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .env("PATH", path)
        .args(["eval", "run", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_success(&output);
    assert_eq!(
        fs::read_to_string(dir.join("typst-args.txt")).unwrap(),
        "compile\n--format\nsvg\n-\n-\n"
    );
    assert_eq!(
        fs::read_to_string(dir.join("typst-stdin.txt")).unwrap(),
        "#let answer = 42\n"
    );
    let stdout = stdout(&output);
    assert!(stdout.contains("language: typst"), "stdout: {stdout}");
    assert!(stdout.contains("exit-code: 0"), "stdout: {stdout}");
}

#[cfg(unix)]
#[test]
fn lint_cli_rejects_invalid_typst_from_builtin_compile_stdin() {
    let dir = test_dir("lint-typst-invalid-runtime");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(
        dir.join("notes.org"),
        "#+begin_src typst :runtime typst :lint :format\n#broken\n#+end_src\n",
    )
    .unwrap();
    let typst = bin.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
if [ "$1" != "compile" ] || [ "$2" != "--format" ] || [ "$3" != "svg" ] || [ "$4" != "-" ] || [ "$5" != "-" ] || [ "$#" -ne 5 ]; then
  printf 'unexpected typst invocation\n' >&2
  exit 64
fi
body=$(cat)
case "$body" in
  *'#broken'*) printf 'error: invalid Typst body\n' >&2; exit 1 ;;
esac
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let path = format!("{}:{}", bin.display(), env::var("PATH").unwrap_or_default());

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .env("PATH", path)
        .args(["lint", "notes.org"])
        .output()
        .unwrap();

    assert!(!output.status.success(), "stdout: {}", stdout(&output));
    let rendered = format!(
        "{}{}",
        stdout(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rendered.contains("ORG043"), "output: {rendered}");
    assert!(
        rendered.contains("invalid Typst body"),
        "output: {rendered}"
    );
    assert_no_typst_artifact(&dir);
}

#[cfg(unix)]
#[test]
fn lint_cli_accepts_valid_typst_from_builtin_compile_stdin() {
    let dir = test_dir("lint-typst-valid-runtime");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(
        dir.join("notes.org"),
        "#+begin_src typst\n#let answer = 42\n#+end_src\n",
    )
    .unwrap();
    let typst = bin.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
if [ "$1" != "compile" ] || [ "$2" != "--format" ] || [ "$3" != "svg" ] || [ "$4" != "-" ] || [ "$5" != "-" ] || [ "$#" -ne 5 ]; then
  printf 'unexpected typst invocation\n' >&2
  exit 64
fi
body=$(cat)
case "$body" in
  *'#let answer = 42'*) ;;
  *) printf 'missing typst source in stdin\n' >&2; exit 65 ;;
esac
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let path = format!("{}:{}", bin.display(), env::var("PATH").unwrap_or_default());

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .env("PATH", path)
        .args(["lint", "notes.org"])
        .output()
        .unwrap();

    assert_success(&output);
    assert_no_typst_artifact(&dir);
}

#[cfg(unix)]
#[test]
fn runtime_lint_resolves_relative_include_from_org_source_parent() {
    let dir = test_dir("lint-typst-relative-include");
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    let source_path = docs.join("notes.org");
    let source = "#+begin_src typst\n#include \"fragment.typ\"\n#+end_src\n";
    fs::write(&source_path, source).unwrap();
    fs::write(docs.join("fragment.typ"), "Relative include resolved.\n").unwrap();
    let typst = dir.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
if [ "$1" != "compile" ] || [ "$2" != "--format" ] || [ "$3" != "svg" ] || [ "$4" != "-" ] || [ "$5" != "-" ] || [ "$#" -ne 5 ]; then
  printf 'unexpected typst invocation\n' >&2
  exit 64
fi
body=$(cat)
case "$body" in
  *'fragment.typ'*) ;;
  *) printf 'missing relative include in stdin\n' >&2; exit 65 ;;
esac
if [ ! -f fragment.typ ]; then
  printf 'relative include not found from runtime cwd\n' >&2
  exit 66
fi
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let options = orgize::lint::LintOptions {
        source_path: Some(source_path),
        ..orgize::lint::LintOptions::default()
    };
    let policy = orgize::lint::RuntimeLintExecutionPolicy::default().with_program_binding(
        orgize::lint::RuntimeLintProgramBinding::Exact(typst.clone()),
    );

    let report = orgize::lint::lint_org_with_options_and_runtime_policy(source, &options, &policy);

    assert!(
        report.is_clean(),
        "report: {}",
        report.to_compact_text("docs/notes.org", source)
    );
    assert_no_typst_artifact(&dir);
}

#[cfg(unix)]
#[test]
fn runtime_lint_timeout_kills_and_reaps_child_with_org043() {
    let dir = test_dir("lint-typst-timeout-reap");
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    let source_path = docs.join("notes.org");
    let source = "#+begin_src typst\n#let answer = 42\n#+end_src\n";
    fs::write(&source_path, source).unwrap();
    let typst = dir.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
if [ "$1" != "compile" ] || [ "$2" != "--format" ] || [ "$3" != "svg" ] || [ "$4" != "-" ] || [ "$5" != "-" ] || [ "$#" -ne 5 ]; then
  printf 'unexpected typst invocation\n' >&2
  exit 64
fi
printf '%s\n' "$$" > .typst-child.pid
exec tail -f /dev/null
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let options = orgize::lint::LintOptions {
        source_path: Some(source_path),
        ..orgize::lint::LintOptions::default()
    };
    let policy = orgize::lint::RuntimeLintExecutionPolicy::bounded(
        std::time::Duration::from_secs(5),
        1_048_576,
    )
    .unwrap()
    .with_program_binding(orgize::lint::RuntimeLintProgramBinding::Exact(
        typst.clone(),
    ));

    let report = orgize::lint::lint_org_with_options_and_runtime_policy(source, &options, &policy);

    let rendered = report.to_compact_text("docs/notes.org", source);
    assert!(rendered.contains("ORG043"), "report: {rendered}");
    assert!(rendered.contains("timed out"), "report: {rendered}");
    let pid = fs::read_to_string(docs.join(".typst-child.pid")).unwrap();
    let alive = Command::new("kill")
        .args(["-0", pid.trim()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(!alive.success(), "timed-out child {pid} is still alive");
    assert_no_typst_artifact(&dir);
}

#[cfg(unix)]
#[test]
fn lint_cli_rejects_combined_typst_output_over_shared_budget() {
    let dir = test_dir("lint-typst-combined-output-budget");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(
        dir.join("notes.org"),
        "#+begin_src typst\n#let answer = 42\n#+end_src\n",
    )
    .unwrap();
    let typst = bin.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
if [ "$1" != "compile" ] || [ "$2" != "--format" ] || [ "$3" != "svg" ] || [ "$4" != "-" ] || [ "$5" != "-" ] || [ "$#" -ne 5 ]; then
  printf 'unexpected typst invocation\n' >&2
  exit 64
fi
cat >/dev/null
head -c 614400 /dev/zero
head -c 614400 /dev/zero >&2
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let path = format!("{}:{}", bin.display(), env::var("PATH").unwrap_or_default());

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .env("PATH", path)
        .args(["lint", "notes.org"])
        .output()
        .unwrap();

    assert!(!output.status.success(), "stdout: {}", stdout(&output));
    let rendered = format!(
        "{}{}",
        stdout(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        rendered.contains("combined runtime output exceeded 1048576 bytes"),
        "output: {rendered}"
    );
    assert_no_typst_artifact(&dir);
}

#[cfg(unix)]
#[test]
fn runtime_validation_evidence_emits_complete_schema_v1_body_free_receipt() {
    let dir = test_dir("runtime-validation-evidence-complete");
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    let source_path = docs.join("notes.org");
    let source = "#+begin_src typst\n#let answer = 42\n#+end_src\n";
    fs::write(&source_path, source).unwrap();
    let typst = dir.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
cat >/dev/null
printf 'out'
printf 'err' >&2
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let options = orgize::lint::LintOptions {
        source_path: Some(source_path.clone()),
        ..orgize::lint::LintOptions::default()
    };
    let context =
        orgize::lint::RuntimeValidationSourceContext::new(source_path, docs.clone(), dir.clone())
            .unwrap();
    let policy = orgize::lint::RuntimeLintExecutionPolicy::bounded(
        std::time::Duration::from_secs(5),
        1_048_576,
    )
    .unwrap()
    .with_program_binding(orgize::lint::RuntimeLintProgramBinding::Exact(typst));

    let evidence = orgize::lint::lint_org_with_runtime_validation_evidence(
        source,
        &options,
        Some(&context),
        &policy,
    )
    .unwrap();

    assert!(evidence.lint_report.is_clean());
    assert_eq!(evidence.receipts.len(), 1);
    let receipt = &evidence.receipts[0];
    assert_eq!(
        receipt.status,
        orgize::lint::RuntimeValidationStatus::Accepted
    );
    assert_eq!(
        receipt.observation.termination_outcome,
        orgize::lint::RuntimeValidationTerminationOutcome::Exited
    );
    assert_eq!(receipt.observation.exit_status.code(), Some(0));
    assert!(receipt.observation.child_lifecycle.child_reaped());
    assert_eq!(receipt.observation.output_bytes.stdout.get(), 3);
    assert_eq!(receipt.observation.output_bytes.stderr.get(), 3);
    let value = receipt.to_json_value();
    assert_eq!(
        value["schemaId"],
        serde_json::Value::String("org-babel-runtime-validation-receipt.v1".to_string())
    );
    assert_eq!(
        value["schemaVersion"],
        serde_json::Value::String("1".to_string())
    );
    assert_eq!(
        value["sourceContext"]["sourcePath"],
        source_path_to_json(&docs.join("notes.org"))
    );
    assert_eq!(
        value["sourceContext"]["workingDirectory"],
        source_path_to_json(&docs)
    );
    assert_eq!(
        value["sourceContext"]["packageRoot"],
        source_path_to_json(&dir)
    );
    assert_eq!(value["binding"]["kind"], "exact-path");
    assert_eq!(value["policy"]["timeoutMs"], 5_000);
    assert_eq!(value["policy"]["outputByteBudget"], 1_048_576);
    assert_eq!(value["observation"]["stdoutBytes"], 3);
    assert_eq!(value["observation"]["stderrBytes"], 3);
    assert_eq!(value["status"], "accepted");
    assert!(value["diagnosticCode"].is_null());
    let rendered = value.to_string();
    assert!(
        !rendered.contains("\"stdout\":"),
        "receipt leaked stdout: {rendered}"
    );
    assert!(
        !rendered.contains("\"stderr\":"),
        "receipt leaked stderr: {rendered}"
    );
}

#[cfg(unix)]
#[test]
fn runtime_validation_evidence_rejects_missing_or_mismatched_context() {
    let dir = test_dir("runtime-validation-evidence-context");
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    let source_path = docs.join("notes.org");
    let source = "#+begin_src typst\n#let answer = 42\n#+end_src\n";
    fs::write(&source_path, source).unwrap();
    let options = orgize::lint::LintOptions {
        source_path: Some(source_path.clone()),
        ..orgize::lint::LintOptions::default()
    };
    let policy = orgize::lint::RuntimeLintExecutionPolicy::default();

    let missing =
        orgize::lint::lint_org_with_runtime_validation_evidence(source, &options, None, &policy)
            .unwrap_err();
    assert!(
        missing.contains("requires explicit source context"),
        "error: {missing}"
    );
    let mismatch = orgize::lint::RuntimeValidationSourceContext::new(
        source_path.clone(),
        dir.clone(),
        dir.clone(),
    )
    .unwrap_err();
    assert!(
        mismatch.contains("must equal source parent"),
        "error: {mismatch}"
    );
    let context =
        orgize::lint::RuntimeValidationSourceContext::new(source_path, docs, dir).unwrap();
    let mismatched_options = orgize::lint::LintOptions {
        source_path: Some(PathBuf::from("other.org")),
        ..orgize::lint::LintOptions::default()
    };
    let mismatch = orgize::lint::lint_org_with_runtime_validation_evidence(
        source,
        &mismatched_options,
        Some(&context),
        &policy,
    )
    .unwrap_err();
    assert!(
        mismatch.contains("does not match lint source path"),
        "error: {mismatch}"
    );
}

#[cfg(unix)]
#[test]
fn runtime_validation_evidence_records_combined_budget_rejection() {
    let dir = test_dir("runtime-validation-evidence-budget");
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    let source_path = docs.join("notes.org");
    let source = "#+begin_src typst\n#let answer = 42\n#+end_src\n";
    fs::write(&source_path, source).unwrap();
    let typst = dir.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
cat >/dev/null
head -c 614400 /dev/zero
head -c 614400 /dev/zero >&2
exit 0
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let options = orgize::lint::LintOptions {
        source_path: Some(source_path.clone()),
        ..orgize::lint::LintOptions::default()
    };
    let context =
        orgize::lint::RuntimeValidationSourceContext::new(source_path, docs, dir.clone()).unwrap();
    let policy = orgize::lint::RuntimeLintExecutionPolicy::bounded(
        std::time::Duration::from_secs(5),
        1_048_576,
    )
    .unwrap()
    .with_program_binding(orgize::lint::RuntimeLintProgramBinding::Exact(typst));

    let evidence = orgize::lint::lint_org_with_runtime_validation_evidence(
        source,
        &options,
        Some(&context),
        &policy,
    )
    .unwrap();

    let receipt = &evidence.receipts[0];
    assert_eq!(
        receipt.observation.termination_outcome,
        orgize::lint::RuntimeValidationTerminationOutcome::OutputBudgetExceeded
    );
    assert_eq!(
        receipt.status,
        orgize::lint::RuntimeValidationStatus::Rejected
    );
    assert_eq!(
        receipt.diagnostic_code,
        Some(orgize::lint::RuntimeValidationDiagnosticCode::Org043)
    );
    assert!(receipt.observation.output_bytes.combined() > receipt.policy.output_byte_budget);
    assert!(receipt.observation.child_lifecycle.child_reaped());
}

#[cfg(unix)]
#[test]
fn runtime_validation_evidence_records_timeout_and_reaping() {
    let dir = test_dir("runtime-validation-evidence-timeout");
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    let source_path = docs.join("notes.org");
    let source = "#+begin_src typst\n#let answer = 42\n#+end_src\n";
    fs::write(&source_path, source).unwrap();
    let typst = dir.join("typst");
    fs::write(
        &typst,
        r#"#!/bin/sh
cat >/dev/null
printf '%s\n' "$$" > .typst-child.pid
exec tail -f /dev/null
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&typst).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&typst, permissions).unwrap();
    let options = orgize::lint::LintOptions {
        source_path: Some(source_path.clone()),
        ..orgize::lint::LintOptions::default()
    };
    let context =
        orgize::lint::RuntimeValidationSourceContext::new(source_path, docs.clone(), dir).unwrap();
    let policy = orgize::lint::RuntimeLintExecutionPolicy::bounded(
        std::time::Duration::from_secs(5),
        1_048_576,
    )
    .unwrap()
    .with_program_binding(orgize::lint::RuntimeLintProgramBinding::Exact(typst));

    let evidence = orgize::lint::lint_org_with_runtime_validation_evidence(
        source,
        &options,
        Some(&context),
        &policy,
    )
    .unwrap();

    let receipt = &evidence.receipts[0];
    assert_eq!(
        receipt.observation.termination_outcome,
        orgize::lint::RuntimeValidationTerminationOutcome::TimedOut
    );
    assert_eq!(
        receipt.status,
        orgize::lint::RuntimeValidationStatus::Rejected
    );
    assert!(receipt.observation.child_lifecycle.child_reaped());
    let pid = fs::read_to_string(docs.join(".typst-child.pid")).unwrap();
    let alive = Command::new("kill")
        .args(["-0", pid.trim()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(!alive.success(), "timed-out child {pid} is still alive");
}

#[test]
fn eval_cli_run_rejects_non_shell_blocks() {
    let dir = test_dir("eval-run-non-shell");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("notes.org"),
        r#"#+NAME: verify
#+BEGIN_SRC python :results output replace
print("no")
#+END_SRC
"#,
    )
    .unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args(["eval", "run", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("has no registered eval runtime `python`"),
        "stderr: {stderr}"
    );
}

#[test]
fn eval_cli_run_rejects_unregistered_runtime_override() {
    let dir = test_dir("eval-run-unregistered-runtime");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("notes.org"),
        r#"#+NAME: verify
#+BEGIN_SRC typst :runtime /usr/local/bin/typst
"ok"
#+END_SRC
"#,
    )
    .unwrap();

    let output = crate::library_cli::orgize_cli_command()
        .current_dir(&dir)
        .args(["eval", "run", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("has no registered eval runtime `/usr/local/bin/typst`"),
        "stderr: {stderr}"
    );
}

fn eval_fixture() -> &'static str {
    r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo should-not-run
#+END_SRC
"#
}

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        stdout(output),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn source_path_to_json(path: &std::path::Path) -> serde_json::Value {
    serde_json::Value::String(path.to_string_lossy().into_owned())
}

#[cfg(unix)]
fn assert_no_typst_artifact(dir: &std::path::Path) {
    let artifact = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("typ" | "pdf")
            )
        });
    assert!(
        artifact.is_none(),
        "unexpected Typst artifact: {artifact:?}"
    );
}

fn test_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("orgize-cli-tests")
        .join(format!("{name}-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    fs::canonicalize(path).unwrap()
}
