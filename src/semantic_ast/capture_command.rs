//! Capture command service for native Org entry plans.

use std::{fs, path::PathBuf};

use super::{
    AgendaDate, AgentCaptureInsertPosition, AgentCaptureKind, AgentCaptureMemoryPolicy,
    AgentCapturePlan, AgentCaptureReceipt, AgentCaptureReceiptKind, AgentCaptureRequest,
    AgentCaptureSource, AgentCaptureTarget, CONTRACT_ORG_PROPERTY, OrgContractEvaluation,
    OrgContractEvaluationScope, OrgContractRegistry, OrgContractScope, evaluate_org_contract,
    parse_contract_reference, parse_contracts_from_document,
};
use crate::Org;

pub enum OrgCapturePlanCommandOutput {
    Help(&'static str),
    Plan(String),
}

pub fn org_capture_plan_command(args: Vec<String>) -> Result<OrgCapturePlanCommandOutput, String> {
    let args = OrgCapturePlanArgs::parse(args)?;
    if args.help {
        return Ok(OrgCapturePlanCommandOutput::Help(CAPTURE_PLAN_USAGE));
    }

    let contract_check_args = args.contract_check_args()?;
    let plan = args.into_request()?.plan();
    let contract_check = capture_contract_check(&plan.org_entry, contract_check_args)?;
    Ok(OrgCapturePlanCommandOutput::Plan(render_org_capture_plan(
        &plan,
        contract_check.as_ref(),
    )))
}

struct OrgCapturePlanArgs {
    help: bool,
    kind: Option<AgentCaptureKind>,
    title: Option<String>,
    body: Option<String>,
    quote: Option<String>,
    target_kind: Option<OrgCapturePlanTargetKind>,
    target_file: Option<String>,
    outline_path: Vec<String>,
    contract_id: Option<String>,
    contract_registry_paths: Vec<PathBuf>,
    date: Option<AgendaDate>,
    insert_position: Option<AgentCaptureInsertPosition>,
    tags: Vec<String>,
    properties: Vec<(String, String)>,
    source_url: Option<String>,
    source_label: Option<String>,
    actor: Option<String>,
    memory_policy: Option<AgentCaptureMemoryPolicy>,
    requires_confirmation: bool,
}

#[derive(Clone, Copy)]
enum OrgCapturePlanTargetKind {
    Inbox,
    Datetree,
    OutlinePath,
    CurrentSection,
}

impl Default for OrgCapturePlanArgs {
    fn default() -> Self {
        Self {
            help: false,
            kind: None,
            title: None,
            body: None,
            quote: None,
            target_kind: None,
            target_file: None,
            outline_path: Vec::new(),
            contract_id: None,
            contract_registry_paths: Vec::new(),
            date: None,
            insert_position: None,
            tags: Vec::new(),
            properties: Vec::new(),
            source_url: None,
            source_label: None,
            actor: None,
            memory_policy: None,
            requires_confirmation: true,
        }
    }
}

impl OrgCapturePlanArgs {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            match arg.as_str() {
                "-h" | "--help" => parsed.help = true,
                "--kind" => {
                    index += 1;
                    parsed.kind = Some(parse_capture_kind(required_flag_value(
                        &args, index, "--kind",
                    )?)?);
                }
                "--title" => {
                    index += 1;
                    parsed.title = Some(required_flag_value(&args, index, "--title")?.to_string());
                }
                "--body" => {
                    index += 1;
                    parsed.body = Some(required_flag_value(&args, index, "--body")?.to_string());
                }
                "--quote" => {
                    index += 1;
                    parsed.quote = Some(required_flag_value(&args, index, "--quote")?.to_string());
                }
                "--target" => {
                    index += 1;
                    parsed.target_kind = Some(parse_capture_target_kind(required_flag_value(
                        &args, index, "--target",
                    )?)?);
                }
                "--target-file" => {
                    index += 1;
                    parsed.target_file =
                        Some(required_flag_value(&args, index, "--target-file")?.to_string());
                }
                "--outline" => {
                    index += 1;
                    parsed.outline_path =
                        parse_outline_path(required_flag_value(&args, index, "--outline")?);
                }
                "--contract" => {
                    index += 1;
                    parsed.contract_id =
                        Some(required_flag_value(&args, index, "--contract")?.to_string());
                }
                "--org-contract-registry" | "--contract-registry" => {
                    index += 1;
                    parsed
                        .contract_registry_paths
                        .push(PathBuf::from(required_flag_value(
                            &args,
                            index,
                            arg.as_str(),
                        )?));
                }
                "--date" => {
                    index += 1;
                    parsed.date = Some(parse_capture_date(required_flag_value(
                        &args, index, "--date",
                    )?)?);
                }
                "--insert" | "--insert-position" => {
                    index += 1;
                    parsed.insert_position = Some(parse_insert_position(required_flag_value(
                        &args,
                        index,
                        arg.as_str(),
                    )?)?);
                }
                "--tag" => {
                    index += 1;
                    parsed
                        .tags
                        .push(required_flag_value(&args, index, "--tag")?.to_string());
                }
                "--property" => {
                    index += 1;
                    parsed.properties.push(parse_property(required_flag_value(
                        &args,
                        index,
                        "--property",
                    )?)?);
                }
                "--source-url" => {
                    index += 1;
                    parsed.source_url =
                        Some(required_flag_value(&args, index, "--source-url")?.to_string());
                }
                "--source-label" => {
                    index += 1;
                    parsed.source_label =
                        Some(required_flag_value(&args, index, "--source-label")?.to_string());
                }
                "--actor" => {
                    index += 1;
                    parsed.actor = Some(required_flag_value(&args, index, "--actor")?.to_string());
                }
                "--memory-policy" => {
                    index += 1;
                    parsed.memory_policy = Some(parse_memory_policy(required_flag_value(
                        &args,
                        index,
                        "--memory-policy",
                    )?)?);
                }
                "--no-confirm" => parsed.requires_confirmation = false,
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown asp org capture flag `{arg}`"));
                }
                _ => {
                    return Err(format!(
                        "unexpected asp org capture positional argument `{arg}`; use --target-file for paths"
                    ));
                }
            }
            index += 1;
        }
        Ok(parsed)
    }

    fn into_request(self) -> Result<AgentCaptureRequest, String> {
        if self.contract_id.is_some()
            && self
                .properties
                .iter()
                .any(|(key, _)| key.eq_ignore_ascii_case(CONTRACT_ORG_PROPERTY))
        {
            return Err(
                "asp org capture --contract cannot be combined with --property CONTRACT_ORG=..."
                    .to_string(),
            );
        }
        let target = self.capture_target()?;
        let source = self.capture_source();
        let title = self.title.ok_or_else(|| {
            "asp org capture --title is required; pass the plan headline explicitly".to_string()
        })?;
        let mut request =
            AgentCaptureRequest::new(self.kind.unwrap_or(AgentCaptureKind::Task), title)
                .target(target)
                .source(source)
                .requires_confirmation(self.requires_confirmation);
        if let Some(body) = self.body {
            request = request.body(body);
        }
        if let Some(quote) = self.quote {
            request = request.quote(quote);
        }
        if let Some(memory_policy) = self.memory_policy {
            request = request.memory_policy(memory_policy);
        }
        if let Some(contract_id) = self.contract_id {
            request = request.property(CONTRACT_ORG_PROPERTY, contract_id);
        }
        for tag in self.tags {
            request = request.tag(tag);
        }
        for (key, value) in self.properties {
            request = request.property(key, value);
        }
        Ok(request)
    }

    fn capture_target(&self) -> Result<AgentCaptureTarget, String> {
        let kind = self.target_kind.unwrap_or_else(|| {
            if !self.outline_path.is_empty() {
                OrgCapturePlanTargetKind::OutlinePath
            } else if self.date.is_some() {
                OrgCapturePlanTargetKind::Datetree
            } else {
                OrgCapturePlanTargetKind::Inbox
            }
        });
        let mut target = match kind {
            OrgCapturePlanTargetKind::Inbox => AgentCaptureTarget::inbox(),
            OrgCapturePlanTargetKind::Datetree => {
                let Some(date) = self.date else {
                    return Err(
                        "asp org capture --target datetree requires --date YYYY-MM-DD".to_string(),
                    );
                };
                AgentCaptureTarget::datetree(date)
            }
            OrgCapturePlanTargetKind::OutlinePath => {
                if self.outline_path.is_empty() {
                    return Err(
                        "asp org capture --target outline requires --outline <PATH>".to_string()
                    );
                }
                AgentCaptureTarget::outline_path(self.outline_path.clone())
            }
            OrgCapturePlanTargetKind::CurrentSection => AgentCaptureTarget::current_section(),
        };
        if let Some(source_file) = &self.target_file {
            target = target.source_file(source_file);
        }
        if let Some(insert_position) = self.insert_position {
            target = target.insert_position(insert_position);
        }
        Ok(target)
    }

    fn capture_source(&self) -> AgentCaptureSource {
        let mut source = if let Some(url) = &self.source_url {
            AgentCaptureSource::url(
                url,
                self.source_label
                    .as_deref()
                    .filter(|label| !label.trim().is_empty())
                    .unwrap_or(url),
            )
        } else {
            AgentCaptureSource::conversation()
        };
        if let Some(actor) = &self.actor {
            source = source.actor(actor);
        }
        source
    }

    fn contract_check_args(&self) -> Result<Option<OrgCaptureContractCheckArgs>, String> {
        let Some(contract_id) = self.contract_id.clone() else {
            return Err("asp org capture requires --contract CONTRACT_ID".to_string());
        };
        if contract_id.trim().is_empty() {
            return Err("asp org capture --contract requires a non-empty contract id".to_string());
        }
        if self.contract_registry_paths.is_empty() {
            return Err(
                "asp org capture --contract requires --org-contract-registry PATH.org".to_string(),
            );
        }
        Ok(Some(OrgCaptureContractCheckArgs {
            contract_id,
            registry_paths: self.contract_registry_paths.clone(),
        }))
    }
}

#[derive(Clone, Debug)]
struct OrgCaptureContractCheckArgs {
    contract_id: String,
    registry_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
struct OrgCaptureContractCheck {
    contract_id: String,
    registry_paths: Vec<PathBuf>,
    evaluation: OrgContractEvaluation,
}

fn render_org_capture_plan(
    plan: &AgentCapturePlan,
    contract_check: Option<&OrgCaptureContractCheck>,
) -> String {
    let mut output = String::new();
    output.push_str("[CAPTURE] asp org capture\n");
    output.push_str("target: ");
    output.push_str(plan.target.kind.as_str());
    output.push('\n');
    output.push_str("target-file: ");
    output.push_str(plan.target.source_file.as_deref().unwrap_or("<runtime>"));
    output.push('\n');
    output.push_str("outline: ");
    if plan.target.outline_path.is_empty() {
        output.push_str("<none>");
    } else {
        output.push_str(&plan.target.outline_path.join(" / "));
    }
    output.push('\n');
    output.push_str("date: ");
    if let Some(date) = plan.target.date {
        output.push_str(&format!(
            "{:04}-{:02}-{:02}",
            date.year, date.month, date.day
        ));
    } else {
        output.push_str("<none>");
    }
    output.push('\n');
    output.push_str("insert-position: ");
    output.push_str(plan.target.insert_position.as_str());
    output.push('\n');
    output.push_str("requires-confirmation: ");
    output.push_str(if plan.requires_confirmation {
        "true"
    } else {
        "false"
    });
    output.push('\n');
    output.push_str("application: ");
    output.push_str(plan.application.action.as_str());
    output.push('\n');
    output.push_str("preconditions:\n");
    for precondition in &plan.application.preconditions {
        output.push_str("- ");
        output.push_str(precondition.kind.as_str());
        output.push_str(": ");
        output.push_str(&precondition.message);
        output.push('\n');
    }
    output.push_str("receipts:\n");
    for receipt in &plan.receipts {
        output.push_str("- ");
        output.push_str(org_capture_receipt_label(receipt.kind));
        output.push_str(": ");
        output.push_str(&org_capture_receipt_message(receipt));
        output.push('\n');
    }
    if !plan.warnings.is_empty() {
        output.push_str("warnings:\n");
        for warning in &plan.warnings {
            output.push_str("- ");
            output.push_str(warning.kind.as_str());
            output.push_str(": ");
            output.push_str(&warning.message);
            output.push('\n');
        }
    }
    if let Some(check) = contract_check {
        output.push_str("contract-check:\n");
        output.push_str("- contract: ");
        output.push_str(&check.contract_id);
        output.push('\n');
        output.push_str("- status: passed\n");
        output.push_str("- assertions: ");
        output.push_str(&check.evaluation.assertions.len().to_string());
        output.push('\n');
        output.push_str("- registries:");
        for path in &check.registry_paths {
            output.push(' ');
            output.push_str(&path.display().to_string());
        }
        output.push('\n');
    }
    output.push_str("org-entry:\n");
    output.push_str(&plan.org_entry);
    if !plan.org_entry.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(
        "next: review org-entry, then apply through AST-patch/edit-plan; asp org capture performed no write\n",
    );
    output
}

fn capture_contract_check(
    org_entry: &str,
    args: Option<OrgCaptureContractCheckArgs>,
) -> Result<Option<OrgCaptureContractCheck>, String> {
    let Some(args) = args else {
        return Ok(None);
    };
    let registry = load_capture_contract_registry(&args.registry_paths)?;
    let reference = parse_contract_reference(args.contract_id.as_str());
    let contract = registry.resolve(&reference).ok_or_else(|| {
        format!(
            "asp org capture --contract `{}` was not found in the loaded Org contract registry",
            args.contract_id
        )
    })?;
    let document = Org::parse(org_entry).document();
    let evaluation = match contract.scope {
        OrgContractScope::Document => {
            evaluate_org_contract(&document, contract, OrgContractEvaluationScope::document())
        }
        OrgContractScope::Subtree => {
            let section = document
                .sections
                .first()
                .ok_or_else(|| "asp org capture rendered no Org section to check".to_string())?;
            evaluate_org_contract(
                &document,
                contract,
                OrgContractEvaluationScope::section(
                    section.raw_title.trim_end(),
                    vec![section.raw_title.trim_end().to_string()],
                    section.ann.range,
                ),
            )
        }
    };
    let failed_assertions = evaluation
        .assertions
        .iter()
        .filter(|assertion| assertion.status.is_failed())
        .map(|assertion| assertion.assertion_id.as_str())
        .collect::<Vec<_>>();
    if !failed_assertions.is_empty() {
        return Err(format!(
            "asp org capture contract check failed for `{}`: {}",
            args.contract_id,
            failed_assertions.join(", ")
        ));
    }
    Ok(Some(OrgCaptureContractCheck {
        contract_id: args.contract_id,
        registry_paths: args.registry_paths,
        evaluation,
    }))
}

fn load_capture_contract_registry(paths: &[PathBuf]) -> Result<OrgContractRegistry, String> {
    let mut registry = OrgContractRegistry::default();
    for path in paths {
        let source =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let document = Org::parse(&source).document();
        let loaded = parse_contracts_from_document(&document, Some(path.as_path()));
        registry.contracts.extend(loaded.contracts);
    }
    Ok(registry)
}

fn parse_capture_kind(value: &str) -> Result<AgentCaptureKind, String> {
    match normalized(value).as_str() {
        "idea" => Ok(AgentCaptureKind::Idea),
        "article-note" | "articlenote" => Ok(AgentCaptureKind::ArticleNote),
        "task" => Ok(AgentCaptureKind::Task),
        "decision" => Ok(AgentCaptureKind::Decision),
        "preference" => Ok(AgentCaptureKind::Preference),
        "correction" => Ok(AgentCaptureKind::Correction),
        "memory-candidate" | "memorycandidate" => Ok(AgentCaptureKind::MemoryCandidate),
        "evidence" => Ok(AgentCaptureKind::Evidence),
        "note" => Ok(AgentCaptureKind::Note),
        _ => Err(format!("unsupported capture kind `{value}`")),
    }
}

fn org_capture_receipt_label(kind: AgentCaptureReceiptKind) -> &'static str {
    match kind {
        AgentCaptureReceiptKind::AgentInterpreted => "callerInterpreted",
        _ => kind.as_str(),
    }
}

fn org_capture_receipt_message(receipt: &AgentCaptureReceipt) -> String {
    match receipt.kind {
        AgentCaptureReceiptKind::AgentInterpreted => {
            "capture arguments carry an explicit kind; Org templates do not infer it".to_string()
        }
        _ => receipt.message.clone(),
    }
}

fn parse_capture_target_kind(value: &str) -> Result<OrgCapturePlanTargetKind, String> {
    match normalized(value).as_str() {
        "inbox" => Ok(OrgCapturePlanTargetKind::Inbox),
        "datetree" => Ok(OrgCapturePlanTargetKind::Datetree),
        "outline" | "outline-path" | "outlinepath" => Ok(OrgCapturePlanTargetKind::OutlinePath),
        "current-section" | "currentsection" => Ok(OrgCapturePlanTargetKind::CurrentSection),
        _ => Err(format!("unsupported capture target `{value}`")),
    }
}

fn parse_insert_position(value: &str) -> Result<AgentCaptureInsertPosition, String> {
    match normalized(value).as_str() {
        "append" => Ok(AgentCaptureInsertPosition::Append),
        "prepend" => Ok(AgentCaptureInsertPosition::Prepend),
        "first-child" | "firstchild" => Ok(AgentCaptureInsertPosition::FirstChild),
        "last-child" | "lastchild" => Ok(AgentCaptureInsertPosition::LastChild),
        _ => Err(format!("unsupported insert position `{value}`")),
    }
}

fn parse_memory_policy(value: &str) -> Result<AgentCaptureMemoryPolicy, String> {
    match normalized(value).as_str() {
        "none" => Ok(AgentCaptureMemoryPolicy::None),
        "candidate" => Ok(AgentCaptureMemoryPolicy::Candidate),
        "background" => Ok(AgentCaptureMemoryPolicy::Background),
        "decision" => Ok(AgentCaptureMemoryPolicy::Decision),
        "transient" => Ok(AgentCaptureMemoryPolicy::Transient),
        "supersedes" => Ok(AgentCaptureMemoryPolicy::Supersedes),
        _ => Err(format!("unsupported memory policy `{value}`")),
    }
}

fn parse_capture_date(value: &str) -> Result<AgendaDate, String> {
    AgendaDate::parse_ymd(value)
        .ok_or_else(|| format!("asp org capture --date expects YYYY-MM-DD, got `{value}`"))
}

fn parse_outline_path(value: &str) -> Vec<String> {
    value
        .split(['/', '>'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_property(value: &str) -> Result<(String, String), String> {
    let Some((key, property_value)) = value.split_once('=') else {
        return Err(format!(
            "asp org capture --property expects KEY=VALUE, got `{value}`"
        ));
    };
    if key.trim().is_empty() {
        return Err("asp org capture --property key cannot be empty".to_string());
    }
    Ok((key.trim().to_string(), property_value.trim().to_string()))
}

fn required_flag_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &str,
) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn normalized(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

const CAPTURE_PLAN_USAGE: &str = "Usage: orgize capture-plan --title TITLE [--contract CONTRACT_ID --org-contract-registry PATH.org] [--kind task|note|decision|idea|evidence|preference|correction|memory-candidate|article-note] [--body TEXT] [--quote TEXT] [--target inbox|datetree|outline|current-section] [--target-file PATH] [--outline A/B] [--date YYYY-MM-DD] [--insert append|prepend|first-child|last-child] [--tag TAG] [--property KEY=VALUE] [--source-url URL] [--source-label LABEL] [--actor ACTOR] [--memory-policy none|candidate|background|decision|transient|supersedes] [--no-confirm]";
