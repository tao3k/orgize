//! Document guide, prime, table-of-contents, and frontier renderers.

use std::{collections::BTreeSet, fs, path::Path, process::ExitCode};

use crate::org::Org;

use super::{
    command_format::{heading_field, heading_fields},
    elements::{count_kind, display_path, escape_field, last_existing_path, option_value},
    model::{DocumentElement, DocumentLanguage},
};

const DOCUMENT_PRIME_OWNER_LIMIT: usize = 12;

pub(crate) fn run_elements_query(
    language: DocumentLanguage,
    args: Vec<String>,
) -> Result<ExitCode, String> {
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

pub(crate) fn print_guide(language: DocumentLanguage) {
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
        println!(
            "|surface capture purpose=state-init-and-non-mutating-org-entry-plan output=compact-plan content=false"
        );
    }
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
    if language == DocumentLanguage::Org {
        println!(
            "|cmd search-memory={} search memory --workspace . --view seeds",
            language.command_prefix()
        );
    }
    println!(
        "|cmd query-metadata={} query --term <term> --workspace . --view metadata",
        language.command_prefix()
    );
    println!(
        "|cmd query-selector={} query --selector <structural-selector> --workspace . --view metadata",
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
        println!(
            "|cmd capture-init={} capture init --state-root <STATE_ROOT> --source-dir <LANGUAGES_ORG_DIR>",
            language.command_prefix()
        );
        println!(
            "|cmd capture={} capture --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>",
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
        "|cmd query-content-selector={} query --selector <structural-selector> --workspace . --content",
        language.command_prefix()
    );
}

fn print_element_guide(language: DocumentLanguage) {
    println!(
        "|query-axis term matches=kind,sourceKind,path,text,content,field-key,field-value combine=all-terms"
    );
    println!("|query-axis selector matches=parser-structural-selector combine=term,kind,field");
    println!("|query-axis kind matches=exact-element-kind combine=all-kinds");
    println!("|query-axis field matches=key-or-key=value value-match=contains combine=all-fields");
    println!(
        "|query-axis content requires=selector|term|kind|field output=matched-element-content"
    );
    match language {
        DocumentLanguage::Org => {
            println!(
                "|element-map heading,task,paragraph,property,planning,table,block,list,listItem,checklistItem,link,image"
            );
            println!("|field-map heading fields=level,title,todo,todoType,priority,tag");
            println!(
                "|field-map task source=Headline fields=level,title,todo,todoType,priority,tag"
            );
            println!("|field-map paragraph fields=text content=raw-paragraph");
            println!("|field-map property fields=key,value");
            println!("|field-map planning fields=scheduled,deadline,closed");
            println!("|field-map table fields=header");
            println!("|field-map block fields=kind=source|export,lang,backend");
            println!("|field-map list fields=listKind=ordered|unordered,descriptive");
            println!("|field-map listItem fields=bullet,indent,counter,tag");
            println!(
                "|field-map checklistItem source=SyntaxListItem fields=bullet,indent,checkbox,checked,tag"
            );
            println!("|field-map link fields=target,description");
            println!("|field-map image fields=target,description");
            println!(
                "|recipe todo-tasks=asp org query --kind task --field todo=TODO --workspace . --view metadata"
            );
            println!(
                "|recipe checked-checklist-items=asp org query --kind checklistItem --field checked=true --workspace . --view metadata"
            );
            println!(
                "|recipe property-value=asp org query --kind property --field key=<KEY> --workspace . --view metadata"
            );
            println!(
                "|recipe sdd-kind-properties=asp org query --kind property --field key=SDD_KIND --workspace . --view metadata"
            );
            println!(
                "|recipe org-id-properties=asp org query --kind property --field key=ID --field value=<ID> --workspace . --view metadata"
            );
            println!(
                "|recipe tagged-tasks=asp org query --kind task --term <TEXT> --field tag=<TAG> --workspace . --view metadata"
            );
            println!(
                "|recipe done-tasks=asp org query --kind task --field todo=DONE --workspace . --view metadata"
            );
            println!(
                "|recipe active-done-artifacts=asp org query --kind task --field todo=DONE --exclude-dir archives --workspace <ORG_ARTIFACTS_ABS_PATH> --content"
            );
            println!(
                "|recipe current-session-tasks=asp org search memory --session <SESSION_ID> --workspace <ORG_ARTIFACTS_ABS_PATH> --view seeds"
            );
            println!(
                "|recipe capture-task=asp org capture --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>"
            );
            println!(
                "|recipe rust-blocks=asp org query --kind block --field kind=source --field lang=rust --workspace . --view metadata"
            );
            println!(
                "|recipe paragraph-content=asp org query --kind paragraph --term <term> --workspace . --content"
            );
            println!(
                "|recipe structural-selector=asp org query --selector <structural-selector> --workspace . --view metadata"
            );
        }
        DocumentLanguage::Markdown => {
            println!(
                "|element-map heading,paragraph,table,block,list,listItem,checklistItem,link,image,frontMatter,thematicBreak"
            );
            println!("|field-map heading fields=level,title");
            println!("|field-map paragraph fields=text content=paragraph-text");
            println!("|field-map block fields=kind=code,lang");
            println!("|field-map list fields=listKind,start");
            println!("|field-map checklistItem fields=checked,checkbox");
            println!("|field-map link fields=target");
            println!("|field-map image fields=target");
            println!("|recipe headings=asp md query --kind heading --workspace . --view metadata");
            println!(
                "|recipe checked-checklist-items=asp md query --kind checklistItem --field checked=true --workspace . --view metadata"
            );
            println!(
                "|recipe code-blocks=asp md query --kind block --field kind=code --workspace . --view metadata"
            );
            println!(
                "|recipe paragraph-content=asp md query --kind paragraph --term <term> --workspace . --content"
            );
            println!(
                "|recipe structural-selector=asp md query --selector <structural-selector> --workspace . --view metadata"
            );
        }
    }
}

pub(super) fn print_search_guide(language: DocumentLanguage) {
    println!(
        "[search-guide] lang={} provider=orgize protocol=search-guide.v1 root=.",
        language.id()
    );
    println!(
        "|view prime returns=headings,tasks,properties,planning,tables,blocks,lists,checklistItems,links,images"
    );
    println!(
        "|view toc returns=document-heading-outline fields=path,range,level,title,todo,priority,tag"
    );
    println!("|view fzf args=query returns=bounded-document-facts");
    if language == DocumentLanguage::Org {
        println!("|view memory args=--session?,--plan?,--term? returns=current-org-memory-cards");
    }
    println!(
        "|view fzf-toc args=query command=\"{} search fzf <query...> --workspace . --view toc\" returns=matched-document-heading-outline combine=document-all-terms",
        language.command_prefix()
    );
}

pub(super) fn print_query_guide(language: DocumentLanguage) {
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
        "|mode selector command=\"query --selector <structural-selector> --workspace . --view metadata\" output=element-frontier"
    );
    println!(
        "|mode verbatim command=\"query --selector <structural-selector> --workspace . --verbatim\" output=exact-parser-node-source"
    );
    println!(
        "|mode content command=\"query --term <term> --workspace . --content\" output=pure-query-content"
    );
    println!("|combine all=--selector+--term+--kind+--field semantics=intersection");
    println!(
        "|field-match key command=\"query --field <key> --workspace . --view metadata\" output=elements-with-field"
    );
    println!(
        "|field-match value command=\"query --field <key=value> --workspace . --view metadata\" output=elements-with-containing-value"
    );
    println!("|walk-filter exclude-dir command=\"query --exclude-dir <DIR> --workspace .\"");
    println!("|content-rule requires=--selector|--term|--kind|--field");
}
pub(crate) fn print_prime(language: DocumentLanguage, root: &Path, facts: &[DocumentElement]) {
    let document_owners = document_prime_owners(root, facts);
    println!(
        "[search-prime] lang={} root={} doc={} heading={} paragraph={} property={} planning={} table={} block={} list={} task={} link={} image={}",
        language.id(),
        display_path(root),
        document_owners.len(),
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
    print_prime_owner_frontier(&document_owners);
    for fact in facts.iter().take(80) {
        println!("{}", fact.render());
    }
    println!("|next search:fzf,search:owner,query:term,query:selector");
}

fn document_prime_owners(root: &Path, facts: &[DocumentElement]) -> Vec<String> {
    facts
        .iter()
        .map(|fact| document_prime_owner_path(root, &fact.path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(DOCUMENT_PRIME_OWNER_LIMIT)
        .collect()
}

fn document_prime_owner_path(root: &Path, path: &str) -> String {
    let path = Path::new(path);
    let owner = path.strip_prefix(root).unwrap_or(path);
    owner.to_string_lossy().replace('\\', "/")
}

fn print_prime_owner_frontier(owners: &[String]) {
    println!(
        "legend: ID=kind:role(value)!next; entries profile(selectors=>returns); frontier ID.next"
    );
    println!("aliases: graph:{{G=search,O=owner}}");
    let owner_ids = owners
        .iter()
        .enumerate()
        .map(|(index, _)| {
            if index == 0 {
                "O".to_string()
            } else {
                format!("O{}", index + 1)
            }
        })
        .collect::<Vec<_>>();
    if owners.is_empty() {
        println!("G>{{}}");
    } else {
        println!(
            "{}",
            owners
                .iter()
                .zip(owner_ids.iter())
                .map(|(owner, owner_id)| format!("{owner_id}=owner:path({owner})!owner"))
                .collect::<Vec<_>>()
                .join(";")
        );
        println!(
            "G>{{{}}}",
            owner_ids
                .iter()
                .map(|owner_id| format!("{owner_id}:selects"))
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    println!(
        "rank={} frontier={}",
        owner_ids.join(","),
        owner_ids
            .iter()
            .map(|owner_id| format!("{owner_id}.owner"))
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("entries=owner-elements(O=>headings+metadata+query-selectors)");
}

pub(crate) fn print_toc(language: DocumentLanguage, root: &Path, headings: &[DocumentElement]) {
    print_toc_header(language, root, headings, "search-toc", None);
    print_toc_rows(language, headings);
}

pub(crate) fn print_fzf_toc(
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
        let selector = heading.structural_selector.as_str();
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

pub(crate) fn print_owner(language: DocumentLanguage, owner: &str, facts: &[DocumentElement]) {
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

pub(crate) fn print_fzf(
    language: DocumentLanguage,
    query: &str,
    root: &Path,
    facts: &[DocumentElement],
) {
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

pub(crate) fn print_selector_frontier(
    language: DocumentLanguage,
    selector: &str,
    facts: &[DocumentElement],
) {
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
        "|next content-query=\"{} query --selector {} --content --workspace .\"",
        language.command_prefix(),
        escape_field(selector)
    );
}
