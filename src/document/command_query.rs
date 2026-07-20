//! Document query execution and compact content rendering.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use super::{
    command_render::{print_query_guide, print_selector_frontier},
    command_search::walk_config_with_cli_excludes,
    elements::{
        display_path, escape_field, filter_elements_by_query, has_flag, index_path,
        last_existing_path, option_value, option_values, query_project_with_config,
    },
    model::{DocumentElement, DocumentLanguage, DocumentWalkConfig},
    packets::{print_query_json, print_selector_query_json},
    source_selection::{SourceSelector, select_source, structural_selector_fragment},
};

pub(crate) fn run_query(
    language: DocumentLanguage,
    args: Vec<String>,
    walk_config: &DocumentWalkConfig,
) -> Result<ExitCode, String> {
    if args.first().is_some_and(|arg| arg == "guide") {
        print_query_guide(language);
        return Ok(ExitCode::SUCCESS);
    }

    let json_output = has_flag(&args, "--json");
    let content_output = has_flag(&args, "--content");
    let verbatim_output = has_flag(&args, "--verbatim");
    let selector = option_value(&args, "--selector");
    let terms = option_values(&args, "--term");
    let kinds = option_values(&args, "--kind");
    let fields = option_values(&args, "--field");
    let view = option_value(&args, "--view").unwrap_or("metadata");
    if view != "metadata" {
        return Err(format!(
            "{} query: unsupported document view `{view}`",
            language.id()
        ));
    }
    if verbatim_output
        && (json_output
            || content_output
            || !terms.is_empty()
            || !kinds.is_empty()
            || !fields.is_empty())
    {
        return Err(format!(
            "{} query: --verbatim requires one exact structural --selector and cannot be combined with --json, --content, --term, --kind, or --field",
            language.id()
        ));
    }
    if verbatim_output && selector.is_none() {
        return Err(format!(
            "{} query: --verbatim requires one exact structural --selector",
            language.id()
        ));
    }
    if args.iter().any(|arg| arg == "--code") {
        return Err(format!(
            "{} query: document providers use --content for query projection; --code is reserved for source-language providers",
            language.id()
        ));
    }
    if content_output
        && selector.is_none()
        && terms.is_empty()
        && kinds.is_empty()
        && fields.is_empty()
    {
        return Err(format!(
            "{} query: --content requires --selector, --term, --kind, or --field so it cannot read the whole document set",
            language.id()
        ));
    }
    if let Some(selector) = selector {
        if verbatim_output {
            let selection = SourceSelector::parse_query(selector)?;
            if selection.structural_selector.is_none() {
                return Err(format!(
                    "{} query: --verbatim requires a parser-owned structural selector",
                    language.id()
                ));
            }
            let facts = selector_elements(language, &selection)?;
            let [fact] = facts.as_slice() else {
                return Err(format!(
                    "{} query: --verbatim selector must resolve to exactly one parser fact, found {}",
                    language.id(),
                    facts.len()
                ));
            };
            let source = fs::read_to_string(&selection.path)
                .map_err(|error| format!("{}: {error}", selection.path.display()))?;
            print!(
                "{}",
                select_source(
                    &source,
                    super::source_selection::SourceLineRange {
                        start_line: fact.line,
                        end_line: fact.end_line,
                    },
                )
            );
        } else if json_output {
            let selection = SourceSelector::parse_query(selector)?;
            let evidence = super::packets::document_query_evidence(
                [selection.path.clone()],
                Some(&selection.path),
            )?;
            let facts = selector_elements(language, &selection)?;
            let facts = filter_elements_by_query(facts, &terms, &kinds, &fields);
            print_selector_query_json(
                language,
                selector,
                &selection,
                &facts,
                content_output,
                evidence,
            )?;
        } else if content_output {
            let selection = SourceSelector::parse_query(selector)?;
            let facts = selector_elements(language, &selection)?;
            let facts = filter_elements_by_query(facts, &terms, &kinds, &fields);
            print_query_content(&facts);
        } else {
            let selection = SourceSelector::parse_query(selector)?;
            let facts = selector_elements(language, &selection)?;
            let facts = filter_elements_by_query(facts, &terms, &kinds, &fields);
            print_selector_frontier(language, selector, &facts);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let root = last_existing_path(&args).unwrap_or_else(|| PathBuf::from("."));
    let walk_config = walk_config_with_cli_excludes(walk_config, &args);
    let evidence = if json_output {
        let mut source_paths = Vec::new();
        super::elements::collect_document_paths(language, &root, &walk_config, &mut source_paths)?;
        source_paths.sort();
        source_paths.dedup();
        Some(super::packets::document_query_evidence(source_paths, None)?)
    } else {
        None
    };
    let facts = query_project_with_config(language, &root, &walk_config, &terms, &fields)?;
    let matches = filter_elements_by_query(facts, &terms, &kinds, &fields);
    if json_output {
        print_query_json(
            language,
            &terms,
            &root,
            &matches,
            content_output,
            evidence.ok_or_else(|| "missing semantic document query evidence".to_string())?,
        )?;
    } else if content_output {
        print_query_content(&matches);
    } else {
        print_query_matches(language, &terms, &root, &matches);
    }
    Ok(ExitCode::SUCCESS)
}
fn print_query_matches(
    language: DocumentLanguage,
    terms: &[String],
    root: &Path,
    facts: &[DocumentElement],
) {
    println!(
        "[query] lang={} terms={} root={} hit={}",
        language.id(),
        terms.len(),
        display_path(root),
        facts.len()
    );
    for fact in facts.iter().take(80) {
        println!("{}", fact.render());
    }
    if facts.is_empty() {
        print_query_no_hit(language, terms, root);
    }
}

fn print_query_no_hit(language: DocumentLanguage, terms: &[String], root: &Path) {
    let terms_display = if terms.is_empty() {
        "-".to_string()
    } else {
        terms
            .iter()
            .map(|term| escape_field(term))
            .collect::<Vec<_>>()
            .join(",")
    };
    println!("|no-hit reason=empty-intersection combine=all-terms terms={terms_display}");

    let prefix = language.command_prefix();
    let root_arg = shell_arg(&display_path(root));
    let first_term = terms.first().map(String::as_str).unwrap_or("<term>");
    let first_term_arg = if terms.is_empty() {
        "<term>".to_string()
    } else {
        shell_arg(first_term)
    };
    println!(
        "|next search-lexical=\"{prefix} search lexical {first_term_arg} --workspace {root_arg} --view seeds\""
    );
    println!(
        "|next query-single-term=\"{prefix} query --term {first_term_arg} --workspace {root_arg} --view metadata\""
    );
    println!("|next query-guide=\"{prefix} query guide --workspace {root_arg}\"");
    println!(
        "|next selector-source=\"rerun metadata query and use an emitted structuralSelector\""
    );
}

pub(super) fn shell_arg(value: &str) -> String {
    if value.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || matches!(
                character,
                '-' | '_' | '.' | '/' | ':' | '@' | '+' | '=' | '<' | '>'
            )
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn print_query_content(facts: &[DocumentElement]) {
    for content in projected_content_facts(facts)
        .iter()
        .take(80)
        .map(DocumentElement::content_text)
        .map(|content| compact_query_content(&content))
        .filter(|content| !content.is_empty())
    {
        println!("{content}");
    }
}

pub(crate) fn compact_query_content(content: &str) -> String {
    let mut compacted = String::with_capacity(content.len());
    let mut previous_blank = false;
    let mut inside_preserved_block = false;
    let mut after_forced_boundary = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !compacted.is_empty() {
                previous_blank = true;
            }
            continue;
        }

        if inside_preserved_block {
            if !compacted.is_empty() && !compacted.ends_with('\n') {
                compacted.push('\n');
            }
            compacted.push_str(line.trim_end());
            if ends_preserved_content_block(trimmed) {
                inside_preserved_block = false;
                after_forced_boundary = true;
            }
        } else {
            let forces_boundary = forces_compacted_content_boundary(trimmed);
            if previous_blank || after_forced_boundary || forces_boundary {
                if !compacted.is_empty() && !compacted.ends_with('\n') {
                    compacted.push('\n');
                }
            } else if !compacted.is_empty() && !compacted.ends_with('\n') {
                compacted.push(' ');
            }
            let compacted_line = compact_query_content_line(trimmed);
            compacted.push_str(&compacted_line);
            if starts_preserved_content_block(trimmed) {
                inside_preserved_block = true;
            }
            after_forced_boundary = forces_boundary;
        }
        previous_blank = false;
    }
    compacted
}

fn compact_query_content_line(line: &str) -> String {
    let mut words = line.split_whitespace();
    let Some(first_word) = words.next() else {
        return String::new();
    };
    let mut compacted = String::with_capacity(line.len());
    compacted.push_str(first_word);
    for word in words {
        compacted.push(' ');
        compacted.push_str(word);
    }
    compacted
}

fn starts_preserved_content_block(line: &str) -> bool {
    is_markdown_fence(line) || is_org_preserved_block_start(line)
}

fn forces_compacted_content_boundary(line: &str) -> bool {
    starts_preserved_content_block(line) || is_markdown_thematic_break(line)
}

fn ends_preserved_content_block(line: &str) -> bool {
    is_markdown_fence(line) || is_org_preserved_block_end(line)
}

fn is_markdown_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

fn is_markdown_thematic_break(line: &str) -> bool {
    if line.len() < 3 {
        return false;
    }
    let mut chars = line.chars().filter(|character| !character.is_whitespace());
    let Some(marker) = chars.next() else {
        return false;
    };
    matches!(marker, '-' | '_' | '*') && chars.all(|character| character == marker)
}

fn is_org_preserved_block_start(line: &str) -> bool {
    let line = line.to_ascii_lowercase();
    line.starts_with("#+begin_src") || line.starts_with("#+begin_example")
}

fn is_org_preserved_block_end(line: &str) -> bool {
    let line = line.to_ascii_lowercase();
    line.starts_with("#+end_src") || line.starts_with("#+end_example")
}

fn projected_content_facts(facts: &[DocumentElement]) -> Vec<DocumentElement> {
    let mut selected = Vec::new();
    for fact in facts {
        if content_shadowed_by_selected_container(fact, facts) {
            continue;
        }
        if selected
            .iter()
            .any(|existing: &DocumentElement| same_content_projection(existing, fact))
        {
            continue;
        }
        selected.push(fact.clone());
    }
    selected
}

fn content_shadowed_by_selected_container(
    fact: &DocumentElement,
    facts: &[DocumentElement],
) -> bool {
    if fact.kind == "paragraph" {
        return facts.iter().any(|candidate| {
            matches!(candidate.kind, "listItem" | "checklistItem")
                && !candidate.content_text().trim().is_empty()
                && contains_element_range(candidate, fact)
        });
    }
    if fact.kind == "list" {
        return facts.iter().any(|candidate| {
            matches!(candidate.kind, "listItem" | "checklistItem")
                && !candidate.content_text().trim().is_empty()
                && contains_element_range(fact, candidate)
        });
    }
    false
}

fn contains_element_range(container: &DocumentElement, nested: &DocumentElement) -> bool {
    container.path == nested.path
        && container.line <= nested.line
        && container.end_line >= nested.end_line
}

fn same_content_projection(left: &DocumentElement, right: &DocumentElement) -> bool {
    left.path == right.path
        && element_ranges_overlap(left, right)
        && left.content_text().trim() == right.content_text().trim()
        && !left.content_text().trim().is_empty()
}

fn element_ranges_overlap(left: &DocumentElement, right: &DocumentElement) -> bool {
    left.line <= right.end_line && right.line <= left.end_line
}
pub(super) fn heading_facts(facts: &[DocumentElement]) -> Vec<DocumentElement> {
    facts
        .iter()
        .filter(|fact| fact.kind == "heading")
        .cloned()
        .collect()
}

fn selector_elements(
    language: DocumentLanguage,
    selection: &SourceSelector,
) -> Result<Vec<DocumentElement>, String> {
    let facts = index_path(language, &selection.path)?;
    Ok(facts
        .into_iter()
        .filter(|fact| match selection.structural_fragment.as_deref() {
            Some(fragment) => structural_selector_fragment(&fact.structural_selector) == fragment,
            None => true,
        })
        .collect())
}
