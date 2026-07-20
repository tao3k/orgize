//! Usage rendering helpers for CLI commands.

pub(crate) fn print_usage() {
    eprintln!(
        "Usage: orgize <agent-planning|capture-plan|contract|elements-query|eval|export|fmt|guide|lint|md|query|search|sdd|sparse-tree|task-list> [options] [PATH ...]"
    );
}

pub(crate) fn print_export_usage() {
    eprintln!("Usage: orgize export <md|markdown> [PATH ...]");
}

pub(crate) fn print_fmt_usage() {
    eprintln!("Usage: orgize fmt [--check] [PATH ...]");
}

pub(crate) fn print_lint_usage() {
    eprintln!(
        "Usage: orgize lint [--fix] [--format compact|text|json] [--priority-highest VALUE] [--priority-default VALUE] [--priority-lowest VALUE] [--property-schema-registry PATH.json] [--property-schema-registry PATH.json ...] [--org-contract-registry PATH.org] [--org-contract-registry PATH.org ...] [PATH ...]"
    );
}

pub(crate) fn print_sdd_usage() {
    eprintln!("Usage: orgize sdd <status|graph-diff> [options] [PATH ...]");
}

pub(crate) fn print_sdd_status_usage() {
    eprintln!("Usage: orgize sdd status [--json] [--issues-only] [--fail-on-issues] [PATH ...]");
}

pub(crate) fn print_sdd_graph_diff_usage() {
    eprintln!("Usage: orgize sdd graph-diff [--fail-on-drift] [PATH ...]");
}

pub(crate) fn print_agent_planning_usage() {
    eprintln!(
        "Usage: orgize agent-planning --date YYYY-MM-DD [--end YYYY-MM-DD] [--include-done] [--include-archived] [--include-comments] [--match EXPR] [PATH ...]"
    );
}

pub(crate) fn print_sparse_tree_usage() {
    eprintln!(
        "Usage: orgize sparse-tree [--text TEXT] [--match EXPR] [--exclude-done] [--exclude-archived] [--include-comments] [--explain-skips] [PATH ...]"
    );
}

pub(crate) fn print_task_list_usage() {
    eprintln!(
        "Usage: orgize task-list [--cached] [--view active|done|archived|achievement|archive-candidate|closure-needed|repeating] [--text TEXT] [--tag TAG] [--include-done] [--include-archived] [--limit N] [PATH ...]"
    );
}
