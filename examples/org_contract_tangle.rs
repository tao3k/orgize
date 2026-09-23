//! Tangle an Org-native contract's Scheme body through the generated Org CST.

use std::{env, fmt::Write as _, fs, path::Path};

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

fn belongs_to(records: &[GraphRecord], mut id: usize, ancestor: usize) -> bool {
    while let Some(parent) = records[id].parent_id {
        if parent == ancestor {
            return true;
        }
        id = parent;
    }
    false
}

fn property<'a>(records: &'a [GraphRecord], section: usize, key: &str) -> Option<&'a str> {
    records
        .iter()
        .filter(|record| record.kind == "node-property" && belongs_to(records, record.id, section))
        .find(|record| record.field("key") == Some(key))
        .and_then(|record| record.field("value"))
        .map(str::trim)
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
    let sections: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "headline")
        .filter(|record| property(&records, record.id, "CONTRACT_ID").is_some())
        .collect();
    let [section] = sections.as_slice() else {
        return Err("expected exactly one CONTRACT_ID section".into());
    };
    let id = property(&records, section.id, "CONTRACT_ID")
        .filter(|value| !value.is_empty())
        .ok_or("missing CONTRACT_ID")?;
    let scope = match property(&records, section.id, "CONTRACT_SCOPE") {
        Some("document") => "document",
        Some("subtree") => "subtree",
        _ => return Err("CONTRACT_SCOPE must be document or subtree".into()),
    };
    let blocks: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "src-block" && belongs_to(&records, record.id, section.id))
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
        return Err("expected exactly one scheme :org-contract block in contract section".into());
    };
    let body = block
        .field("body")
        .ok_or("Scheme source block has no body")?;
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
        "                 org-elements property child-of descendant-of))"
    )
    .unwrap();
    writeln!(output, "(export org-contract-definition)").unwrap();
    writeln!(output, "(def org-contract-definition").unwrap();
    writeln!(
        output,
        "  (make-org-contract-definition {id:?} '{scope} (org-contract-block"
    )
    .unwrap();
    output.push_str(body);
    if !body.ends_with('\n') {
        output.push('\n');
    }
    writeln!(output, "  )))").unwrap();
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
}
