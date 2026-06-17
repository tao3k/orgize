use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::Org;

use super::{
    elements::{
        DocumentElement, DocumentLanguage, DocumentWalkConfig, count_kind, display_path,
        escape_field, filter_elements, filter_elements_by_query, has_flag, index_path,
        index_project_with_config, last_existing_path, option_value, option_values,
    },
    packets::{print_query_json, print_search_json, print_selector_query_json},
    source_selection::{SourceSelector, select_source},
};

pub fn run_org_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Org, args)
}

pub fn run_md_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Markdown, args)
}

pub fn run_document_command(
    language: DocumentLanguage,
    args: Vec<String>,
) -> Result<ExitCode, String> {
    run_document_command_with_walk_config(language, args, DocumentWalkConfig::default())
}

pub fn run_document_command_with_walk_config(
    language: DocumentLanguage,
    args: Vec<String>,
    walk_config: DocumentWalkConfig,
) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_guide(language);
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "guide" => {
            print_guide(language);
            Ok(ExitCode::SUCCESS)
        }
        "search" => run_search(language, args.collect(), &walk_config),
        "query" => run_query(language, args.collect(), &walk_config),
        "elements-query" => run_elements_query(language, args.collect()),
        "-h" | "--help" | "help" => {
            print_guide(language);
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!(
            "{}: unsupported document command `{command}`",
            language.id()
        )),
    }
}

fn run_search(
    language: DocumentLanguage,
    args: Vec<String>,
    walk_config: &DocumentWalkConfig,
) -> Result<ExitCode, String> {
    let json_output = has_flag(&args, "--json");
    let Some(view) = args.first().map(String::as_str) else {
        return Err(format!("{} search: expected view", language.id()));
    };

    match view {
        "guide" => {
            print_search_guide(language);
            Ok(ExitCode::SUCCESS)
        }
        "prime" => {
            let root = last_existing_path(&args[1..]).unwrap_or_else(|| PathBuf::from("."));
            let facts = index_project_with_config(language, &root, walk_config)?;
            if json_output {
                print_search_json(language, "prime", &root, &facts, None)?;
            } else {
                print_prime(language, &root, &facts);
            }
            Ok(ExitCode::SUCCESS)
        }
        "toc" => {
            let root = last_existing_path(&args[1..]).unwrap_or_else(|| PathBuf::from("."));
            let facts = index_project_with_config(language, &root, walk_config)?;
            let headings = heading_facts(&facts);
            if json_output {
                print_search_json(language, "toc", &root, &headings, None)?;
            } else {
                print_toc(language, &root, &headings);
            }
            Ok(ExitCode::SUCCESS)
        }
        "owner" => {
            let Some(owner) = args.get(1) else {
                return Err(format!("{} search owner: expected path", language.id()));
            };
            let path = PathBuf::from(owner);
            let facts = index_path(language, &path)?;
            if json_output {
                let root = path.parent().unwrap_or_else(|| Path::new("."));
                print_search_json(language, "owner", root, &facts, Some(owner))?;
            } else {
                print_owner(language, owner, &facts);
            }
            Ok(ExitCode::SUCCESS)
        }
        "fzf" => {
            let fzf_args = &args[1..];
            let root_arg_index = last_existing_path_arg_index(fzf_args);
            let terms = fzf_query_terms(fzf_args, root_arg_index);
            if terms.is_empty() {
                return Err(format!("{} search fzf: expected query", language.id()));
            };
            let root = root_arg_index
                .map(|index| PathBuf::from(&fzf_args[index]))
                .unwrap_or_else(|| PathBuf::from("."));
            let toc_output = fzf_toc_requested(fzf_args, root_arg_index);
            let facts = index_project_with_config(language, &root, walk_config)?;
            if toc_output {
                let query = terms.join(" ");
                let headings = heading_facts_for_matching_documents(&facts, &terms);
                if json_output {
                    print_search_json(language, "fzf-toc", &root, &headings, Some(&query))?;
                } else {
                    print_fzf_toc(language, &query, &root, &headings);
                }
            } else {
                let query = terms
                    .first()
                    .expect("terms is non-empty after earlier check")
                    .as_str();
                let matches = filter_elements(&facts, query);
                if json_output {
                    print_search_json(language, "fzf", &root, &matches, Some(query))?;
                } else {
                    print_fzf(language, query, &root, &matches);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        view => Err(format!(
            "{} search: unsupported view `{view}`",
            language.id()
        )),
    }
}

fn run_query(
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
    let from_hook = option_value(&args, "--from-hook");
    let direct_read = from_hook.is_some_and(|value| value == "direct-source-read");
    if args.iter().any(|arg| arg == "--code") {
        return Err(format!(
            "{} query: document providers use --content for query projection; --code is reserved for source-language providers",
            language.id()
        ));
    }
    if direct_read && selector.is_none() {
        return Err(format!(
            "{} query: --from-hook direct-source-read requires --selector <path:start-end>; add --content to project source text",
            language.id()
        ));
    }
    if json_output && direct_read {
        return Err(format!(
            "{} query: --json cannot be combined with --from-hook direct-source-read",
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
        let selection = SourceSelector::parse(selector)?;
        if direct_read {
            let source = fs::read_to_string(&selection.path)
                .map_err(|error| format!("{}: {error}", selection.path.display()))?;
            print!("{}", select_source(&source, selection.range));
        } else if json_output {
            let facts = selector_elements(language, &selection)?;
            let facts = filter_elements_by_query(facts, &terms, &kinds, &fields);
            print_selector_query_json(language, selector, &selection, &facts, content_output)?;
        } else if content_output {
            let facts = selector_elements(language, &selection)?;
            let facts = filter_elements_by_query(facts, &terms, &kinds, &fields);
            print_query_content(&facts);
        } else {
            let facts = selector_elements(language, &selection)?;
            let facts = filter_elements_by_query(facts, &terms, &kinds, &fields);
            print_selector_frontier(language, selector, &facts);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let root = last_existing_path(&args).unwrap_or_else(|| PathBuf::from("."));
    let facts = index_project_with_config(language, &root, walk_config)?;
    let matches = filter_elements_by_query(facts, &terms, &kinds, &fields);
    if json_output {
        print_query_json(language, &terms, &root, &matches, content_output)?;
    } else if content_output {
        print_query_content(&matches);
    } else {
        print_query_matches(language, &terms, &root, &matches);
    }
    Ok(ExitCode::SUCCESS)
}

fn run_elements_query(language: DocumentLanguage, args: Vec<String>) -> Result<ExitCode, String> {
    if language != DocumentLanguage::Org {
        return Err(format!(
            "{} elements-query: Org elements packets are only supported for Org documents",
            language.id()
        ));
    }

    let packet = option_value(&args, "--packet").ok_or_else(|| {
        format!(
            "{} elements-query: expected --packet <json-query-packet>",
            language.id()
        )
    })?;
    let path = last_existing_path(&args).ok_or_else(|| {
        format!(
            "{} elements-query: expected an Org file path",
            language.id()
        )
    })?;
    if !path.is_file() {
        return Err(format!(
            "{} elements-query: expected an Org file path, got `{}`",
            language.id(),
            path.display()
        ));
    }

    let source =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let document = Org::parse(&source).document();
    let output = document
        .org_elements_index_query_packet_json(packet)
        .map_err(|error| format!("{} elements-query: {error}", language.id()))?;
    println!("{output}");
    Ok(ExitCode::SUCCESS)
}

fn print_guide(language: DocumentLanguage) {
    println!(
        "[guide] lang={} provider=orgize protocol=guide.v1 root=.",
        language.id()
    );
    println!("|surface search purpose=document-structure output=compact-seeds content=false");
    println!(
        "|surface query purpose=elements-by-selector-or-term output=metadata-frontier content=false"
    );
    if language == DocumentLanguage::Org {
        println!(
            "|surface elements-query purpose=org-elements-index-packet output=json content=false"
        );
        println!(
            "|surface contract-trace purpose=contract-org-evaluation-trace output=json content=false"
        );
    }
    println!("|surface direct-read purpose=hook-recovery output=pure-content content=true");
    println!("|rule parser-authority={}", language.parser_authority());
    println!("|rule no=check,ast-patch,evidence reason=document-language");
    println!("|rule content=query-projection reason=content-needs-selector-term-kind-or-field");
    println!("|rule project-walk skip=hidden-dirs,target,node_modules,__pycache__,venv,dist,build");
    print_element_guide(language);
    println!(
        "|cmd search-prime={} search prime --workspace . --view seeds",
        language.command_prefix()
    );
    println!(
        "|cmd search-toc={} search toc --workspace .",
        language.command_prefix()
    );
    println!(
        "|cmd search-fzf={} search fzf <query> --workspace . --view seeds",
        language.command_prefix()
    );
    println!(
        "|cmd search-fzf-toc={} search fzf <query...> --workspace . --view toc",
        language.command_prefix()
    );
    println!(
        "|cmd query-metadata={} query --term <term> --workspace . --view metadata",
        language.command_prefix()
    );
    println!(
        "|cmd query-selector={} query --selector <path:start-end> --workspace . --view metadata",
        language.command_prefix()
    );
    println!(
        "|cmd query-kind={} query --kind <element-kind> --workspace . --view metadata",
        language.command_prefix()
    );
    println!(
        "|cmd query-field={} query --field <key=value> --workspace . --view metadata",
        language.command_prefix()
    );
    if language == DocumentLanguage::Org {
        println!(
            "|cmd elements-query={} elements-query --packet <json-query-packet> <org-file>",
            language.command_prefix()
        );
        println!(
            "|cmd contract-trace={} contract trace --org-contract-registry <contract.org> <target.org>",
            language.command_prefix()
        );
    }
    println!(
        "|cmd query-content={} query --term <term> --workspace . --content",
        language.command_prefix()
    );
    println!(
        "|cmd query-content-kind={} query --kind paragraph --term <term> --workspace . --content",
        language.command_prefix()
    );
    println!(
        "|cmd query-content-selector={} query --selector <path:start-end> --workspace . --content",
        language.command_prefix()
    );
    println!(
        "|cmd direct-read={} query --from-hook direct-source-read --selector <path:start-end> --workspace . --content",
        language.command_prefix()
    );
}

fn print_element_guide(language: DocumentLanguage) {
    println!(
        "|query-axis term matches=kind,sourceKind,path,text,content,field-key,field-value combine=all-terms"
    );
    println!(
        "|query-axis selector matches=elements-overlapping-path-range combine=term,kind,field"
    );
    println!("|query-axis kind matches=exact-element-kind combine=all-kinds");
    println!("|query-axis field matches=key-or-key=value value-match=contains combine=all-fields");
    println!(
        "|query-axis content requires=selector|term|kind|field output=matched-element-content"
    );
    println!(
        "|query-axis direct-read requires=from-hook+selector+content output=source-preserved-content use=hook-recovery-only"
    );
    match language {
        DocumentLanguage::Org => {
            println!(
                "|element-map heading,paragraph,property,planning,table,block,list,listItem,task,link,image"
            );
            println!("|field-map heading fields=level,title,todo,todoType,priority,tag");
            println!("|field-map paragraph fields=text content=raw-paragraph");
            println!("|field-map property fields=key,value");
            println!("|field-map planning fields=scheduled,deadline,closed");
            println!("|field-map table fields=header");
            println!("|field-map block fields=kind=source|export,lang,backend");
            println!("|field-map list fields=listKind=ordered|unordered,descriptive");
            println!("|field-map listItem fields=bullet,indent,counter,tag");
            println!("|field-map task fields=bullet,indent,checkbox,checked,tag");
            println!("|field-map link fields=target,description");
            println!("|field-map image fields=target,description");
            println!(
                "|recipe todo-headings=asp org query --kind heading --field todo=TODO --workspace . --view metadata"
            );
            println!(
                "|recipe checked-tasks=asp org query --kind task --field checked=true --workspace . --view metadata"
            );
            println!(
                "|recipe property-value=asp org query --kind property --field key=<KEY> --workspace . --view metadata"
            );
            println!(
                "|recipe sdd-property=asp org query --kind property --field key=SDD_KIND --workspace . --view metadata"
            );
            println!(
                "|recipe rust-blocks=asp org query --kind block --field kind=source --field lang=rust --workspace . --view metadata"
            );
            println!(
                "|recipe paragraph-content=asp org query --kind paragraph --term <term> --workspace . --content"
            );
            println!(
                "|recipe range-elements=asp org query --selector <path:start-end> --workspace . --view metadata"
            );
        }
        DocumentLanguage::Markdown => {
            println!(
                "|element-map heading,paragraph,table,block,list,listItem,task,link,image,frontMatter,thematicBreak"
            );
            println!("|field-map heading fields=level,title");
            println!("|field-map paragraph fields=text content=paragraph-text");
            println!("|field-map block fields=kind=code,lang");
            println!("|field-map list fields=listKind,start");
            println!("|field-map task fields=checked,checkbox");
            println!("|field-map link fields=target");
            println!("|field-map image fields=target");
            println!("|recipe headings=asp md query --kind heading --workspace . --view metadata");
            println!(
                "|recipe checked-tasks=asp md query --kind task --field checked=true --workspace . --view metadata"
            );
            println!(
                "|recipe code-blocks=asp md query --kind block --field kind=code --workspace . --view metadata"
            );
            println!(
                "|recipe paragraph-content=asp md query --kind paragraph --term <term> --workspace . --content"
            );
            println!(
                "|recipe range-elements=asp md query --selector <path:start-end> --workspace . --view metadata"
            );
        }
    }
}

fn print_search_guide(language: DocumentLanguage) {
    println!(
        "[search-guide] lang={} provider=orgize protocol=search-guide.v1 root=.",
        language.id()
    );
    println!(
        "|view prime returns=headings,properties,planning,tables,blocks,lists,tasks,links,images"
    );
    println!(
        "|view toc returns=document-heading-outline fields=path,range,level,title,todo,priority,tag"
    );
    println!("|view fzf args=query returns=bounded-document-facts");
    println!(
        "|view fzf-toc args=query command=\"{} search fzf <query...> --workspace . --view toc\" returns=matched-document-heading-outline combine=document-all-terms",
        language.command_prefix()
    );
}

fn print_query_guide(language: DocumentLanguage) {
    println!(
        "[query-guide] lang={} provider=orgize protocol=query-guide.v1 root=.",
        language.id()
    );
    println!(
        "|mode metadata command=\"query --term <term> --workspace . --view metadata\" output=element-frontier"
    );
    println!(
        "|mode kind command=\"query --kind <element-kind> --workspace . --view metadata\" output=element-frontier"
    );
    println!(
        "|mode field command=\"query --field <key=value> --workspace . --view metadata\" output=element-frontier"
    );
    println!(
        "|mode selector command=\"query --selector <path:start-end> --workspace . --view metadata\" output=element-frontier"
    );
    println!(
        "|mode content command=\"query --term <term> --workspace . --content\" output=pure-query-content"
    );
    println!(
        "|mode direct-read command=\"query --from-hook direct-source-read --selector <path:start-end> --workspace . --content\" output=pure-document-content"
    );
    println!("|combine all=--selector+--term+--kind+--field semantics=intersection");
    println!(
        "|field-match key command=\"query --field <key> --workspace . --view metadata\" output=elements-with-field"
    );
    println!(
        "|field-match value command=\"query --field <key=value> --workspace . --view metadata\" output=elements-with-containing-value"
    );
    println!("|content-rule requires=--selector|--term|--kind|--field");
    println!("|direct-read-rule requires=--from-hook+--selector+--content forbids=--json");
}

fn print_prime(language: DocumentLanguage, root: &Path, facts: &[DocumentElement]) {
    println!(
        "[search-prime] lang={} root={} doc={} heading={} paragraph={} property={} planning={} table={} block={} list={} task={} link={} image={}",
        language.id(),
        display_path(root),
        facts
            .iter()
            .map(|fact| fact.path.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        count_kind(facts, "heading"),
        count_kind(facts, "paragraph"),
        count_kind(facts, "property"),
        count_kind(facts, "planning"),
        count_kind(facts, "table"),
        count_kind(facts, "block"),
        count_kind(facts, "list"),
        count_kind(facts, "task"),
        count_kind(facts, "link"),
        count_kind(facts, "image")
    );
    for fact in facts.iter().take(80) {
        println!("{}", fact.render());
    }
    println!("|next search:fzf,search:owner,query:term,query:selector");
}

fn print_toc(language: DocumentLanguage, root: &Path, headings: &[DocumentElement]) {
    print_toc_header(language, root, headings, "search-toc", None);
    print_toc_rows(language, headings);
}

fn print_fzf_toc(
    language: DocumentLanguage,
    query: &str,
    root: &Path,
    headings: &[DocumentElement],
) {
    print_toc_header(language, root, headings, "search-fzf-toc", Some(query));
    print_toc_rows(language, headings);
}

fn print_toc_header(
    language: DocumentLanguage,
    root: &Path,
    headings: &[DocumentElement],
    label: &str,
    query: Option<&str>,
) {
    let document_paths = headings
        .iter()
        .map(|heading| heading.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let max_level = headings
        .iter()
        .filter_map(|heading| heading_field(heading, "level")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0);
    if let Some(query) = query {
        println!(
            "[{label}] lang={} q={} root={} doc={} heading={} maxLevel={} alg=fd-fzf-doc-toc-v1",
            language.id(),
            escape_field(query),
            display_path(root),
            document_paths.len(),
            headings.len(),
            max_level
        );
    } else {
        println!(
            "[{label}] lang={} root={} doc={} heading={} maxLevel={}",
            language.id(),
            display_path(root),
            document_paths.len(),
            headings.len(),
            max_level
        );
    }
}

fn print_toc_rows(language: DocumentLanguage, headings: &[DocumentElement]) {
    let mut current_path = "";
    for heading in headings.iter().take(200) {
        if heading.path != current_path {
            current_path = &heading.path;
            let count = headings
                .iter()
                .filter(|candidate| candidate.path == heading.path)
                .count();
            println!(
                "|doc path=\"{}\" heading={count}",
                escape_field(current_path)
            );
        }
        let level = heading_field(heading, "level").unwrap_or("0");
        let title = heading_field(heading, "title").unwrap_or(heading.text.as_str());
        let selector = format!("{}:{}-{}", heading.path, heading.line, heading.end_line);
        let mut output = format!(
            "|toc path=\"{}\" range=\"{}:{}\" level={} title=\"{}\"",
            escape_field(&heading.path),
            heading.line,
            heading.end_line,
            level,
            escape_field(title)
        );
        for key in ["todo", "priority"] {
            if let Some(value) = heading_field(heading, key) {
                output.push(' ');
                output.push_str(key);
                output.push_str("=\"");
                output.push_str(&escape_field(value));
                output.push('"');
            }
        }
        let tags = heading_fields(heading, "tag");
        if !tags.is_empty() {
            output.push_str(" tag=\"");
            output.push_str(&escape_field(&tags.join(",")));
            output.push('"');
        }
        output.push_str(" next=\"");
        output.push_str(&escape_field(&format!(
            "{} query --selector {selector} --view metadata",
            language.command_prefix()
        )));
        output.push('"');
        println!("{output}");
    }
    println!("|next query:selector,query:kind=heading,query:content,direct-read");
}

fn last_existing_path_arg_index(args: &[String]) -> Option<usize> {
    args.iter()
        .enumerate()
        .rev()
        .filter(|(_, arg)| !arg.starts_with('-'))
        .find_map(|(index, arg)| PathBuf::from(arg).exists().then_some(index))
}

fn fzf_toc_requested(args: &[String], root_arg_index: Option<usize>) -> bool {
    option_value(args, "--view") == Some("toc")
        || args
            .iter()
            .enumerate()
            .any(|(index, arg)| index > 0 && Some(index) != root_arg_index && arg == "toc")
}

fn fzf_query_terms(args: &[String], root_arg_index: Option<usize>) -> Vec<String> {
    let mut terms = Vec::new();
    let mut skip_next = false;
    for (index, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if Some(index) == root_arg_index {
            continue;
        }
        if arg == "--view" {
            skip_next = true;
            continue;
        }
        if arg.starts_with("--") || (index > 0 && arg == "toc") {
            continue;
        }
        terms.push(arg.clone());
    }
    terms
}

fn heading_facts_for_matching_documents(
    facts: &[DocumentElement],
    terms: &[String],
) -> Vec<DocumentElement> {
    let paths = facts
        .iter()
        .map(|fact| fact.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let matching_paths = paths
        .into_iter()
        .filter(|path| {
            let document_facts = facts
                .iter()
                .filter(|candidate| candidate.path.as_str() == *path)
                .collect::<Vec<_>>();
            terms
                .iter()
                .all(|term| document_facts.iter().any(|fact| fact.matches(term)))
        })
        .collect::<std::collections::BTreeSet<_>>();
    facts
        .iter()
        .filter(|fact| fact.kind == "heading" && matching_paths.contains(fact.path.as_str()))
        .cloned()
        .collect()
}

fn print_owner(language: DocumentLanguage, owner: &str, facts: &[DocumentElement]) {
    println!(
        "[search-owner] lang={} q={} item={}",
        language.id(),
        owner,
        facts.len()
    );
    for fact in facts.iter().take(80) {
        println!("{}", fact.render());
    }
}

fn print_fzf(language: DocumentLanguage, query: &str, root: &Path, facts: &[DocumentElement]) {
    println!(
        "[search-fzf] lang={} q={} root={} hit={}",
        language.id(),
        escape_field(query),
        display_path(root),
        facts.len()
    );
    for fact in facts.iter().take(80) {
        println!("{}", fact.render());
    }
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
        "|next search-fzf=\"{prefix} search fzf {first_term_arg} --workspace {root_arg} --view seeds\""
    );
    println!(
        "|next query-single-term=\"{prefix} query --term {first_term_arg} --workspace {root_arg} --view metadata\""
    );
    println!("|next query-guide=\"{prefix} query guide --workspace {root_arg}\"");
    println!(
        "|next direct-read-requires=\"{prefix} query --from-hook direct-source-read --selector <path:start-end> --workspace {root_arg}\""
    );
}

fn shell_arg(value: &str) -> String {
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
    let mut words = content.split_whitespace();
    let Some(first_word) = words.next() else {
        return String::new();
    };
    let mut compacted = String::with_capacity(content.len());
    compacted.push_str(first_word);
    for word in words {
        compacted.push(' ');
        compacted.push_str(word);
    }
    compacted
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
            matches!(candidate.kind, "listItem" | "task")
                && !candidate.content_text().trim().is_empty()
                && contains_element_range(candidate, fact)
        });
    }
    if fact.kind == "list" {
        return facts.iter().any(|candidate| {
            matches!(candidate.kind, "listItem" | "task")
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

fn print_selector_frontier(language: DocumentLanguage, selector: &str, facts: &[DocumentElement]) {
    println!(
        "[query-selector] lang={} selector={} hit={} content=false",
        language.id(),
        escape_field(selector),
        facts.len()
    );
    for fact in facts.iter().take(80) {
        println!("{}", fact.render());
    }
    println!(
        "|next direct-read=\"{} query --from-hook direct-source-read --selector {} --workspace .\"",
        language.command_prefix(),
        escape_field(selector)
    );
}

fn heading_facts(facts: &[DocumentElement]) -> Vec<DocumentElement> {
    facts
        .iter()
        .filter(|fact| fact.kind == "heading")
        .cloned()
        .collect()
}

fn heading_field<'a>(heading: &'a DocumentElement, key: &str) -> Option<&'a str> {
    heading
        .fields
        .iter()
        .find(|(field_key, _)| field_key == key)
        .map(|(_, value)| value.as_str())
}

fn heading_fields<'a>(heading: &'a DocumentElement, key: &str) -> Vec<&'a str> {
    heading
        .fields
        .iter()
        .filter_map(|(field_key, value)| (field_key == key).then_some(value.as_str()))
        .collect()
}

fn selector_elements(
    language: DocumentLanguage,
    selection: &SourceSelector,
) -> Result<Vec<DocumentElement>, String> {
    let facts = index_path(language, &selection.path)?;
    Ok(facts
        .into_iter()
        .filter(|fact| match selection.range {
            Some(range) => fact.line <= range.end_line && fact.end_line >= range.start_line,
            None => true,
        })
        .collect())
}
