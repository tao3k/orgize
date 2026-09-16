//! Document guide and Query frontier renderers.

use std::{fs, process::ExitCode};

use crate::org::Org;

use super::{
    elements::{escape_field, last_existing_path, option_value},
    model::{DocumentElement, DocumentLanguage},
};

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
            println!(
                "|field-map block fields=kind=source|export,lang,backend content=parser-owned-body"
            );
            println!("|field-map list fields=listKind=ordered|unordered,descriptive");
            println!("|field-map listItem fields=bullet,indent,counter,tag");
            println!(
                "|field-map checklistItem source=SyntaxListItem fields=bullet,indent,checkbox,checked,tag"
            );
            println!("|field-map link fields=target,description");
            println!("|field-map image fields=target,description");
            println!(
                "|recipe todo-tasks=orgize org query --kind task --field todo=TODO --workspace . --view metadata"
            );
            println!(
                "|recipe checked-checklist-items=orgize org query --kind checklistItem --field checked=true --workspace . --view metadata"
            );
            println!(
                "|recipe property-value=orgize org query --kind property --field key=<KEY> --workspace . --view metadata"
            );
            println!(
                "|recipe sdd-kind-properties=orgize org query --kind property --field key=SDD_KIND --workspace . --view metadata"
            );
            println!(
                "|recipe org-id-properties=orgize org query --kind property --field key=ID --field value=<ID> --workspace . --view metadata"
            );
            println!(
                "|recipe tagged-tasks=orgize org query --kind task --term <TEXT> --field tag=<TAG> --workspace . --view metadata"
            );
            println!(
                "|recipe done-tasks=orgize org query --kind task --field todo=DONE --workspace . --view metadata"
            );
            println!(
                "|recipe active-done-artifacts=orgize org query --kind task --field todo=DONE --exclude-dir archives --workspace <ORG_ARTIFACTS_ABS_PATH> --content"
            );
            println!(
                "|recipe capture-task=orgize org capture --contract agent.task.v1 --title <TITLE> --target-file <ORG_FILE>"
            );
            println!(
                "|recipe rust-blocks=orgize org query --kind block --field kind=source --field lang=rust --workspace . --view metadata"
            );
            println!(
                "|recipe source-block-content=orgize org query --kind block --field kind=source --field lang=<LANG> --workspace <PATH> --content"
            );
            println!(
                "|recipe paragraph-content=orgize org query --kind paragraph --term <term> --workspace . --content"
            );
            println!(
                "|recipe structural-selector=orgize org query --selector <structural-selector> --workspace . --view metadata"
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
            println!(
                "|recipe headings=orgize md query --kind heading --workspace . --view metadata"
            );
            println!(
                "|recipe checked-checklist-items=orgize md query --kind checklistItem --field checked=true --workspace . --view metadata"
            );
            println!(
                "|recipe code-blocks=orgize md query --kind block --field kind=code --workspace . --view metadata"
            );
            println!(
                "|recipe paragraph-content=orgize md query --kind paragraph --term <term> --workspace . --content"
            );
            println!(
                "|recipe structural-selector=orgize md query --selector <structural-selector> --workspace . --view metadata"
            );
        }
    }
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
