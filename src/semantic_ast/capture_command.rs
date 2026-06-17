//! Capture-plan command service for native Org entry plans.

use super::{
    AgendaDate, AgentCaptureInsertPosition, AgentCaptureKind, AgentCaptureMemoryPolicy,
    AgentCapturePlan, AgentCaptureReceipt, AgentCaptureReceiptKind, AgentCaptureRequest,
    AgentCaptureSource, AgentCaptureTarget,
};

pub enum OrgCapturePlanCommandOutput {
    Help(&'static str),
    Plan(String),
}

pub fn org_capture_plan_command(args: Vec<String>) -> Result<OrgCapturePlanCommandOutput, String> {
    let args = OrgCapturePlanArgs::parse(args)?;
    if args.help {
        return Ok(OrgCapturePlanCommandOutput::Help(CAPTURE_PLAN_USAGE));
    }

    Ok(OrgCapturePlanCommandOutput::Plan(render_org_capture_plan(
        &args.into_request()?.plan(),
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
                    return Err(format!("unknown capture-plan flag `{arg}`"));
                }
                _ => {
                    return Err(format!(
                        "unexpected capture-plan positional argument `{arg}`; use --target-file for paths"
                    ));
                }
            }
            index += 1;
        }
        Ok(parsed)
    }

    fn into_request(self) -> Result<AgentCaptureRequest, String> {
        let target = self.capture_target()?;
        let source = self.capture_source();
        let title = self.title.ok_or_else(|| {
            "capture-plan --title is required; pass the plan headline explicitly".to_string()
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
                        "capture-plan --target datetree requires --date YYYY-MM-DD".to_string()
                    );
                };
                AgentCaptureTarget::datetree(date)
            }
            OrgCapturePlanTargetKind::OutlinePath => {
                if self.outline_path.is_empty() {
                    return Err(
                        "capture-plan --target outline requires --outline <PATH>".to_string()
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
}

fn render_org_capture_plan(plan: &AgentCapturePlan) -> String {
    let mut output = String::new();
    output.push_str("[CAPTURE_PLAN] orgize capture-plan\n");
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
    output.push_str("org-entry:\n");
    output.push_str(&plan.org_entry);
    if !plan.org_entry.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(
        "next: review org-entry, then apply through AST-patch/edit-plan; capture-plan performed no write\n",
    );
    output
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
        "agent-plan" | "agentplan" => Ok(AgentCaptureKind::AgentPlan),
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
            "capture kind is supplied by the caller, not inferred from Org templates".to_string()
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
        .ok_or_else(|| format!("capture-plan --date expects YYYY-MM-DD, got `{value}`"))
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
            "capture-plan --property expects KEY=VALUE, got `{value}`"
        ));
    };
    if key.trim().is_empty() {
        return Err("capture-plan --property key cannot be empty".to_string());
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

const CAPTURE_PLAN_USAGE: &str = "Usage: orgize capture-plan --title TITLE [--kind task|note|decision|idea|evidence|preference|correction|memory-candidate|article-note] [--body TEXT] [--quote TEXT] [--target inbox|datetree|outline|current-section] [--target-file PATH] [--outline A/B] [--date YYYY-MM-DD] [--insert append|prepend|first-child|last-child] [--tag TAG] [--property KEY=VALUE] [--source-url URL] [--source-label LABEL] [--actor ACTOR] [--memory-policy none|candidate|background|decision|transient|supersedes] [--no-confirm]";
