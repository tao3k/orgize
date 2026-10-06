mod export_cli_common;
mod export_cli_markdown;
mod export_cli_org;
mod export_cli_stdin;

pub(super) fn native_cases() -> Vec<(&'static str, fn())> {
    let mut cases = Vec::new();
    for group in [
        export_cli_markdown::NATIVE_CASES,
        export_cli_org::NATIVE_CASES,
        export_cli_stdin::NATIVE_CASES,
    ] {
        cases.extend_from_slice(group);
    }
    cases
}
