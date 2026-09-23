//! Tangle an Org-native contract's Scheme body through the generated Org CST.

use std::{collections::HashSet, env, fmt::Write as _, fs, path::Path};

use gerbil_parser_rowan::GraphRecord;

#[rustfmt::skip]
#[path = "../languages/org/v1/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/structure.rs"]
mod structure;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/graph.rs"]
mod graph;

fn owning_headline(records: &[GraphRecord], mut id: usize) -> Option<usize> {
    loop {
        let record = records.get(id)?;
        if record.kind == "headline" {
            return Some(id);
        }
        id = record.parent_id?;
    }
}

fn property<'a>(
    records: &'a [GraphRecord],
    section: usize,
    key: &str,
) -> Result<Option<&'a str>, String> {
    let values: Vec<_> = records
        .iter()
        .filter(|record| {
            record.kind == "node-property" && owning_headline(records, record.id) == Some(section)
        })
        .filter(|record| record.field("key") == Some(key))
        .map(|record| record.field("value").map(str::trim))
        .collect();
    match values.as_slice() {
        [] => Ok(None),
        [Some(value)] => Ok(Some(value)),
        _ => Err(format!(
            "duplicate or empty {key} property in contract section"
        )),
    }
}

fn tangle(source: &str) -> Result<String, String> {
    let parse = gerbil_parser_rowan::parse_structural_lines(
        &grammar::LANGUAGE,
        &structure::STRUCTURE,
        source,
    )
    .map_err(|error| format!("invalid Org contract source: {error:?}"))?;
    let records = gerbil_parser_rowan::project_syntax_graph(
        &grammar::LANGUAGE,
        &graph::GRAPH,
        &parse.syntax(),
    )
    .map_err(|error| format!("invalid Org Element projection: {error:?}"))?;
    let mut output = String::new();
    writeln!(output, ";;; -*- Gerbil -*-").unwrap();
    writeln!(
        output,
        ";;; @generated from the Org Element CST; do not edit."
    )
    .unwrap();
    writeln!(output, "(import (only-in \"../interface.ss\"").unwrap();
    writeln!(
        output,
        "                 org-contract-block assert-org-element"
    )
    .unwrap();
    writeln!(output, "                 make-org-contract-definition)").unwrap();
    writeln!(
        output,
        "        (only-in \"../../org-elements/interface.ss\""
    )
    .unwrap();
    writeln!(
        output,
        "                 org-elements property property-contains at child-of descendant-of))"
    )
    .unwrap();
    writeln!(output, "(export org-contract-definitions)").unwrap();
    writeln!(output, "(def org-contract-definitions (list").unwrap();
    let mut seen = HashSet::new();
    for section in records.iter().filter(|record| record.kind == "headline") {
        let Some(id) = property(&records, section.id, "CONTRACT_ID")? else {
            continue;
        };
        if id.is_empty() || !seen.insert(id) {
            return Err(format!("empty or duplicate CONTRACT_ID: {id}"));
        }
        let scope = match property(&records, section.id, "CONTRACT_SCOPE")? {
            Some("document") => "document",
            Some("subtree") => "subtree",
            _ => return Err(format!("CONTRACT_SCOPE must be document or subtree: {id}")),
        };
        let blocks: Vec<_> = records
            .iter()
            .filter(|record| {
                record.kind == "src-block"
                    && owning_headline(&records, record.id) == Some(section.id)
            })
            .filter(|record| record.field("language") == Some("scheme"))
            .filter(|record| {
                record.field("header").is_some_and(|header| {
                    header
                        .split_ascii_whitespace()
                        .any(|part| part == ":org-contract")
                })
            })
            .collect();
        let [block] = blocks.as_slice() else {
            return Err(format!(
                "expected exactly one scheme :org-contract block: {id}"
            ));
        };
        let body = block
            .field("body")
            .ok_or("Scheme :org-contract source block has no body")?;
        writeln!(
            output,
            "  (make-org-contract-definition {id:?} '{scope} (org-contract-block"
        )
        .unwrap();
        output.push_str(body);
        if !body.ends_with('\n') {
            output.push('\n');
        }
        writeln!(output, "  ))").unwrap();
    }
    if seen.is_empty() {
        return Err("expected at least one CONTRACT_ID section".into());
    }
    writeln!(output, "))").unwrap();
    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    let [_, input, output] = args.as_slice() else {
        return Err("usage: org_contract_tangle INPUT.org OUTPUT.ss".into());
    };
    let source = fs::read_to_string(Path::new(input))?;
    let tangled = tangle(&source)?;
    fs::write(Path::new(output), tangled)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::tangle;

    #[test]
    fn tangling_uses_element_ancestry_and_org_metadata() {
        let source = include_str!("../languages/org/v1/modules/org-contract/contracts.org");
        let tangled = tangle(source).unwrap();
        assert!(tangled.contains("make-org-contract-definition \"section.scope.v1\" 'subtree"));
        assert!(
            tangled.contains("make-org-contract-definition \"document.headlines.v1\" 'document")
        );
        assert!(tangled.contains("(org-contract-block\n(assert-org-element"));
        assert!(!tangled.contains("(assert count"));
    }

    #[test]
    fn missing_scope_fails_closed() {
        let source = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:END:\n#+begin_src scheme :org-contract\n(list)\n#+end_src\n";
        assert!(tangle(source).is_err());
    }

    #[test]
    fn plain_scheme_babel_block_is_not_a_contract() {
        let source = "* Contract\n:PROPERTIES:\n:CONTRACT_ID: x\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme\n(list)\n#+end_src\n";
        assert!(tangle(source).is_err());
    }

    #[test]
    fn nested_block_does_not_satisfy_parent_contract() {
        let source = "* Parent\n:PROPERTIES:\n:CONTRACT_ID: parent\n:CONTRACT_SCOPE: subtree\n:END:\n** Child\n:PROPERTIES:\n:CONTRACT_ID: child\n:CONTRACT_SCOPE: subtree\n:END:\n#+begin_src scheme :org-contract\n(assert-org-element \"a\" error (bindings) (org-elements headline) (expect at-least 1))\n#+end_src\n";
        assert!(tangle(source).is_err());
    }

    #[test]
    fn duplicate_contract_ids_are_rejected() {
        let source = "* One\n:PROPERTIES:\n:CONTRACT_ID: same\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme :org-contract\n(assert-org-element \"a\" error (bindings) (org-elements headline) (expect at-least 1))\n#+end_src\n* Two\n:PROPERTIES:\n:CONTRACT_ID: same\n:CONTRACT_SCOPE: document\n:END:\n#+begin_src scheme :org-contract\n(assert-org-element \"b\" error (bindings) (org-elements headline) (expect at-least 1))\n#+end_src\n";
        assert!(tangle(source).is_err());
    }
}
