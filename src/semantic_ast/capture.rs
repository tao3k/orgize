//! Agent-facing non-mutating capture plans.

use std::fmt::Write;

use super::{
    AgendaDate, AgendaTime, AgentCaptureApplication, AgentCaptureApplicationAction,
    AgentCaptureApplicationPrecondition, AgentCaptureApplicationPreconditionKind, AgentCaptureLink,
    AgentCaptureMemoryPolicy, AgentCapturePlan, AgentCaptureProperty, AgentCaptureReceipt,
    AgentCaptureReceiptKind, AgentCaptureRequest, AgentCaptureTargetKind, AgentCaptureTimestamp,
    AgentCaptureWarning, AgentCaptureWarningKind,
};

/// Renders an Agent capture request into a reviewable native Org entry plan.
pub fn agent_capture_plan(request: &AgentCaptureRequest) -> AgentCapturePlan {
    let mut warnings = Vec::new();
    let title = clean_title(request.title.as_str());
    let title = if title.is_empty() {
        warnings.push(warning(
            AgentCaptureWarningKind::EmptyTitle,
            "capture title was empty; rendered an explicit placeholder",
        ));
        "Untitled capture".to_string()
    } else {
        title
    };

    let tags = sanitize_tags(&request.tags, &mut warnings);
    let properties = capture_properties(request, &mut warnings);
    let org_entry = render_org_entry(request, title.as_str(), &tags, &properties);

    if request
        .body
        .as_ref()
        .map(|body| body.trim().is_empty())
        .unwrap_or(true)
        && request
            .quote
            .as_ref()
            .map(|quote| quote.trim().is_empty())
            .unwrap_or(true)
    {
        warnings.push(warning(
            AgentCaptureWarningKind::EmptyBody,
            "capture body and quote are both empty",
        ));
    }

    if request.target.kind == AgentCaptureTargetKind::CurrentSection {
        warnings.push(warning(
            AgentCaptureWarningKind::RuntimeOwnedTarget,
            "current-section insertion requires a runtime/editor context",
        ));
    }

    AgentCapturePlan {
        target: request.target.clone(),
        org_entry,
        application: capture_application(request),
        receipts: capture_receipts(request),
        warnings,
        requires_confirmation: request.requires_confirmation,
    }
}

fn render_org_entry(
    request: &AgentCaptureRequest,
    title: &str,
    tags: &[String],
    properties: &[AgentCaptureProperty],
) -> String {
    let mut output = String::new();
    output.push('*');
    output.push(' ');
    if let Some(todo) = request.kind.todo_keyword() {
        output.push_str(todo);
        output.push(' ');
    }
    output.push_str(title);
    if !tags.is_empty() {
        output.push(' ');
        output.push(':');
        output.push_str(&tags.join(":"));
        output.push(':');
    }
    output.push('\n');

    if !properties.is_empty() {
        output.push_str(":PROPERTIES:\n");
        for property in properties {
            let _ = writeln!(
                output,
                ":{}: {}",
                property.key,
                single_line(property.value.as_str())
            );
        }
        output.push_str(":END:\n");
    }

    if let Some(body) = request.body.as_ref().map(|body| body.trim()) {
        if !body.is_empty() {
            output.push('\n');
            output.push_str(body);
            output.push('\n');
        }
    }

    if let Some(quote) = request.quote.as_ref().map(|quote| quote.trim()) {
        if !quote.is_empty() {
            output.push_str("\n#+begin_quote\n");
            output.push_str(quote);
            output.push_str("\n#+end_quote\n");
        }
    }

    let mut links = request.links.clone();
    if let Some(uri) = request
        .source
        .uri
        .as_ref()
        .filter(|uri| !uri.trim().is_empty())
    {
        links.insert(
            0,
            AgentCaptureLink {
                url: uri.clone(),
                label: request.source.label.clone(),
            },
        );
    }
    if !links.is_empty() {
        output.push_str("\nLinks:\n");
        for link in &links {
            let _ = writeln!(output, "- {}", render_link(link));
        }
    }

    output
}

fn capture_application(request: &AgentCaptureRequest) -> AgentCaptureApplication {
    let action = match request.target.kind {
        AgentCaptureTargetKind::CurrentSection => {
            AgentCaptureApplicationAction::ResolveRuntimeTarget
        }
        AgentCaptureTargetKind::Inbox
        | AgentCaptureTargetKind::Datetree
        | AgentCaptureTargetKind::OutlinePath => AgentCaptureApplicationAction::InsertOrgEntry,
    };
    AgentCaptureApplication {
        action,
        target: request.target.clone(),
        preconditions: capture_application_preconditions(request),
    }
}

fn capture_application_preconditions(
    request: &AgentCaptureRequest,
) -> Vec<AgentCaptureApplicationPrecondition> {
    let mut preconditions = Vec::new();
    if request.requires_confirmation {
        preconditions.push(application_precondition(
            AgentCaptureApplicationPreconditionKind::UserConfirmation,
            "runtime must confirm with the user before writing the Org entry",
        ));
    }
    if request.target.source_file.is_none() {
        preconditions.push(application_precondition(
            AgentCaptureApplicationPreconditionKind::SourceFileResolution,
            "runtime must resolve the target Org source file without relying on a local absolute path",
        ));
    } else {
        preconditions.push(application_precondition(
            AgentCaptureApplicationPreconditionKind::WriteLock,
            "runtime must acquire a host-owned git write lock for the target source before applying",
        ));
    }
    match request.target.kind {
        AgentCaptureTargetKind::Datetree => preconditions.push(application_precondition(
            AgentCaptureApplicationPreconditionKind::DatetreeResolution,
            "runtime must create or resolve the datetree heading before insertion",
        )),
        AgentCaptureTargetKind::OutlinePath => preconditions.push(application_precondition(
            AgentCaptureApplicationPreconditionKind::OutlinePathResolution,
            "runtime must resolve the outline path before insertion",
        )),
        AgentCaptureTargetKind::CurrentSection => preconditions.push(application_precondition(
            AgentCaptureApplicationPreconditionKind::CurrentSectionResolution,
            "runtime/editor context must resolve the current section before insertion",
        )),
        AgentCaptureTargetKind::Inbox => {}
    }
    preconditions
}

fn capture_properties(
    request: &AgentCaptureRequest,
    warnings: &mut Vec<AgentCaptureWarning>,
) -> Vec<AgentCaptureProperty> {
    let mut properties = vec![
        property("CAPTURE_KIND", request.kind.as_str()),
        property("CAPTURE_SOURCE", request.source.kind.as_str()),
    ];
    if let Some(actor) = request
        .source
        .actor
        .as_ref()
        .filter(|actor| !actor.trim().is_empty())
    {
        properties.push(property("CAPTURE_ACTOR", actor.as_str()));
    }
    if let Some(captured_at) = request.captured_at {
        properties.push(property("CAPTURED_AT", format_timestamp(captured_at)));
    }
    if let Some(uri) = request
        .source
        .uri
        .as_ref()
        .filter(|uri| !uri.trim().is_empty())
    {
        properties.push(property("SOURCE_URL", uri.as_str()));
    }
    if let Some(label) = request
        .source
        .label
        .as_ref()
        .filter(|label| !label.trim().is_empty())
    {
        properties.push(property("SOURCE_LABEL", label.as_str()));
    }
    if request.memory_policy != AgentCaptureMemoryPolicy::None {
        properties.push(property("MEMORY_POLICY", request.memory_policy.as_str()));
    }

    for property in &request.properties {
        let key = sanitize_property_key(property.key.as_str());
        if key != property.key {
            warnings.push(warning(
                AgentCaptureWarningKind::SanitizedPropertyKey,
                format!(
                    "property key `{}` was rendered as `{key}`",
                    single_line(property.key.as_str())
                ),
            ));
        }
        if !key.is_empty() {
            properties.push(AgentCaptureProperty {
                key,
                value: property.value.clone(),
            });
        }
    }
    properties
}

fn capture_receipts(request: &AgentCaptureRequest) -> Vec<AgentCaptureReceipt> {
    let mut receipts = vec![
        receipt(
            AgentCaptureReceiptKind::NonMutating,
            "orgize rendered a plan without inserting or editing source",
        ),
        receipt(
            AgentCaptureReceiptKind::NativeOrgEntry,
            "capture artifact uses ordinary Org headline, tags, properties, links, and blocks",
        ),
        receipt(
            AgentCaptureReceiptKind::AgentInterpreted,
            format!(
                "capture kind `{}` is supplied by the Agent/caller, not inferred from Emacs Lisp",
                request.kind.as_str()
            ),
        ),
        receipt(
            AgentCaptureReceiptKind::SourceProvenance,
            format!(
                "source kind `{}` recorded in property drawer",
                request.source.kind.as_str()
            ),
        ),
    ];
    if request.memory_policy != AgentCaptureMemoryPolicy::None {
        receipts.push(receipt(
            AgentCaptureReceiptKind::MemoryPolicy,
            format!(
                "memory policy `{}` is advisory evidence for downstream authority projection",
                request.memory_policy.as_str()
            ),
        ));
    }
    if request.requires_confirmation {
        receipts.push(receipt(
            AgentCaptureReceiptKind::RequiresConfirmation,
            "a runtime should ask for confirmation before applying this plan",
        ));
    }
    receipts.push(receipt(
        AgentCaptureReceiptKind::ApplicationPlan,
        "application intent is explicit for downstream runtimes but orgize still performs no write",
    ));
    receipts
}

fn sanitize_tags(tags: &[String], warnings: &mut Vec<AgentCaptureWarning>) -> Vec<String> {
    tags.iter()
        .filter_map(|tag| sanitized_tag(tag, warnings))
        .fold(Vec::new(), push_unique_tag)
}

fn sanitized_tag(tag: &str, warnings: &mut Vec<AgentCaptureWarning>) -> Option<String> {
    let sanitized = sanitize_tag(tag);
    if sanitized != tag {
        warnings.push(warning(
            AgentCaptureWarningKind::SanitizedTag,
            format!("tag `{}` was rendered as `{sanitized}`", single_line(tag)),
        ));
    }
    (!sanitized.is_empty()).then_some(sanitized)
}

fn push_unique_tag(mut tags: Vec<String>, tag: String) -> Vec<String> {
    if !tags.contains(&tag) {
        tags.push(tag);
    }
    tags
}

fn sanitize_tag(tag: &str) -> String {
    tag.trim_matches(':')
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '@' | '#' | '%') {
                Some(ch)
            } else if ch.is_whitespace() || matches!(ch, '-' | '.') {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn sanitize_property_key(key: &str) -> String {
    key.trim_matches(':')
        .chars()
        .filter_map(|ch| {
            let upper = ch.to_ascii_uppercase();
            if upper.is_ascii_alphanumeric() || upper == '_' {
                Some(upper)
            } else if ch.is_whitespace() || matches!(ch, '-' | '.') {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn clean_title(title: &str) -> String {
    single_line(title).trim().to_string()
}

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn render_link(link: &AgentCaptureLink) -> String {
    let url = single_line(link.url.as_str());
    match link
        .label
        .as_ref()
        .map(|label| single_line(label))
        .filter(|label| !label.is_empty())
    {
        Some(label) => format!("[[{url}][{label}]]"),
        None => format!("[[{url}]]"),
    }
}

fn format_timestamp(timestamp: AgentCaptureTimestamp) -> String {
    let mut output = format_date(timestamp.date);
    if let Some(time) = timestamp.time {
        output.push(' ');
        output.push_str(format_time(time).as_str());
    }
    format!("[{output}]")
}

fn format_date(date: AgendaDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}

fn format_time(time: AgendaTime) -> String {
    format!("{:02}:{:02}", time.hour, time.minute)
}

fn property(key: impl Into<String>, value: impl Into<String>) -> AgentCaptureProperty {
    AgentCaptureProperty {
        key: key.into(),
        value: value.into(),
    }
}

fn application_precondition(
    kind: AgentCaptureApplicationPreconditionKind,
    message: impl Into<String>,
) -> AgentCaptureApplicationPrecondition {
    AgentCaptureApplicationPrecondition {
        kind,
        message: message.into(),
    }
}

fn receipt(kind: AgentCaptureReceiptKind, message: impl Into<String>) -> AgentCaptureReceipt {
    AgentCaptureReceipt {
        kind,
        message: message.into(),
    }
}

fn warning(kind: AgentCaptureWarningKind, message: impl Into<String>) -> AgentCaptureWarning {
    AgentCaptureWarning {
        kind,
        message: message.into(),
    }
}
