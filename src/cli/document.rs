use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use super::{
    document_index::{
        DocumentFact, DocumentLanguage, SourceSelector, count_kind, display_path, escape_field,
        filter_facts, has_flag, index_path, index_project, last_existing_path, option_value,
        option_values, select_source,
    },
    document_json::{print_query_json, print_search_json, print_selector_query_json},
};

pub(crate) fn run_org_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Org, args)
}

pub(crate) fn run_md_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Markdown, args)
}

fn run_document_command(language: DocumentLanguage, args: Vec<String>) -> Result<ExitCode, String> {
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
        "search" => run_search(language, args.collect()),
        "query" => run_query(language, args.collect()),
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

fn run_search(language: DocumentLanguage, args: Vec<String>) -> Result<ExitCode, String> {
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
            let facts = index_project(language, &root)?;
            if json_output {
                print_search_json(language, "prime", &root, &facts, None)?;
            } else {
                print_prime(language, &root, &facts);
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
            let Some(query) = args.get(1) else {
                return Err(format!("{} search fzf: expected query", language.id()));
            };
            let root = last_existing_path(&args[2..]).unwrap_or_else(|| PathBuf::from("."));
            let facts = index_project(language, &root)?;
            let matches = filter_facts(&facts, query);
            if json_output {
                print_search_json(language, "fzf", &root, &matches, Some(query))?;
            } else {
                print_fzf(language, query, &root, &matches);
            }
            Ok(ExitCode::SUCCESS)
        }
        view => Err(format!(
            "{} search: unsupported view `{view}`",
            language.id()
        )),
    }
}

fn run_query(language: DocumentLanguage, args: Vec<String>) -> Result<ExitCode, String> {
    if args.first().is_some_and(|arg| arg == "guide") {
        print_query_guide(language);
        return Ok(ExitCode::SUCCESS);
    }

    let json_output = has_flag(&args, "--json");
    let selector = option_value(&args, "--selector");
    let view = option_value(&args, "--view").unwrap_or("metadata");
    if !matches!(view, "metadata" | "content") {
        return Err(format!(
            "{} query: unsupported document view `{view}`",
            language.id()
        ));
    }
    let content_output = args.iter().any(|arg| arg == "--content");
    let content = content_output || view == "content";
    if args.iter().any(|arg| arg == "--code") {
        return Err(format!(
            "{} query: document selectors use --content; --code is reserved for source-language providers",
            language.id()
        ));
    }
    if content && selector.is_none() {
        return Err(format!(
            "{} query: --content or --view content requires --selector",
            language.id()
        ));
    }
    if json_output && content_output {
        return Err(format!(
            "{} query: --json cannot be combined with --content",
            language.id()
        ));
    }
    if let Some(selector) = selector {
        let selection = SourceSelector::parse(selector)?;
        let source = fs::read_to_string(&selection.path)
            .map_err(|error| format!("{}: {error}", selection.path.display()))?;
        if content {
            print!("{}", select_source(&source, selection.range));
        } else if json_output {
            print_selector_query_json(language, selector, &selection, &source)?;
        } else {
            print_selector_frontier(language, selector, &source, selection.range);
        }
        return Ok(ExitCode::SUCCESS);
    }

    let terms = option_values(&args, "--term");
    let root = last_existing_path(&args).unwrap_or_else(|| PathBuf::from("."));
    let facts = index_project(language, &root)?;
    let matches = if terms.is_empty() {
        facts
    } else {
        facts
            .into_iter()
            .filter(|fact| terms.iter().any(|term| fact.matches(term)))
            .collect()
    };
    if json_output {
        print_query_json(language, &terms, &root, &matches)?;
    } else {
        print_query_matches(language, &terms, &root, &matches);
    }
    Ok(ExitCode::SUCCESS)
}

fn print_guide(language: DocumentLanguage) {
    println!(
        "[guide] lang={} provider=orgize protocol=guide.v1 root=.",
        language.id()
    );
    println!("|surface search purpose=document-structure output=compact-seeds content=false");
    println!("|surface query purpose=selector-or-term output=metadata-frontier|pure-content");
    println!("|rule parser-authority={}", language.parser_authority());
    println!("|rule no=check,ast-patch,evidence reason=document-language");
    println!("|element-map heading,property,planning,table,block,list,listItem,task,link,image");
    println!(
        "|cmd search-prime={} search prime --view seeds .",
        language.command_prefix()
    );
    println!(
        "|cmd search-fzf={} search fzf <query> --view seeds .",
        language.command_prefix()
    );
    println!(
        "|cmd query-content={} query --selector <path:start-end> --content .",
        language.command_prefix()
    );
    println!(
        "|cmd query-metadata={} query --term <term> --view metadata .",
        language.command_prefix()
    );
}

fn print_search_guide(language: DocumentLanguage) {
    println!(
        "[search-guide] lang={} provider=orgize protocol=search-guide.v1 root=.",
        language.id()
    );
    println!(
        "|view prime returns=headings,properties,planning,tables,blocks,lists,tasks,links,images"
    );
    println!("|view fzf args=query returns=bounded-document-facts");
}

fn print_query_guide(language: DocumentLanguage) {
    println!(
        "[query-guide] lang={} provider=orgize protocol=query-guide.v1 root=.",
        language.id()
    );
    println!(
        "|mode content command=\"query --selector <path:start-end> --content\" output=pure-document-content"
    );
    println!(
        "|mode metadata command=\"query --term <term> --view metadata .\" output=compact-frontier"
    );
}

fn print_prime(language: DocumentLanguage, root: &Path, facts: &[DocumentFact]) {
    println!(
        "[search-prime] lang={} root={} doc={} heading={} property={} planning={} table={} block={} list={} task={} link={} image={}",
        language.id(),
        display_path(root),
        facts
            .iter()
            .map(|fact| fact.path.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        count_kind(facts, "heading"),
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
    println!("|next search:fzf,search:owner,query:selector");
}

fn print_owner(language: DocumentLanguage, owner: &str, facts: &[DocumentFact]) {
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

fn print_fzf(language: DocumentLanguage, query: &str, root: &Path, facts: &[DocumentFact]) {
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
    facts: &[DocumentFact],
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
}

fn print_selector_frontier(
    language: DocumentLanguage,
    selector: &str,
    source: &str,
    range: Option<(usize, usize)>,
) {
    let selected = select_source(source, range);
    println!(
        "[query-selector] lang={} selector={} bytes={} content=false",
        language.id(),
        escape_field(selector),
        selected.len()
    );
    println!(
        "|next content=\"{} query --selector {} --content .\"",
        language.command_prefix(),
        escape_field(selector)
    );
}
