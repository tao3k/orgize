//! Tangle tagged Org Element queries through the generated Element graph.

use std::{collections::HashSet, env, fmt::Write as _, fs, path::Path};

#[path = "support/org_feature_source.rs"]
mod org_feature_source;

use org_feature_source::{feature_block, owning_headline, property, records};

fn tangle(source: &str, interface_module: &str) -> Result<String, String> {
    if interface_module.is_empty() {
        return Err("Org Elements interface module cannot be empty".into());
    }
    let records = records(source)?;
    let mut output = String::from(
        ";;; -*- Gerbil -*-\n;;; @generated from :org-elements blocks; do not edit.\n",
    );
    writeln!(
        output,
        "(import (only-in {interface_module:?} org-element-query org-elements"
    )
    .unwrap();
    output.push_str(
        "property property-contains all-of any-of at child-of descendant-of))\n\
         (export org-element-queries)\n(def org-element-queries (list\n",
    );
    let mut seen = HashSet::new();
    for section in records.iter().filter(|record| record.kind == "headline") {
        let Some(id) = property(&records, section.id, "QUERY_ID")? else {
            continue;
        };
        if id.is_empty() || !seen.insert(id) {
            return Err(format!("empty or duplicate QUERY_ID: {id}"));
        }
        let body = feature_block(&records, section.id, ":org-elements")?;
        writeln!(output, "  (org-element-query {id:?}").unwrap();
        output.push_str(body);
        if !body.ends_with('\n') {
            output.push('\n');
        }
        writeln!(output, "  )").unwrap();
    }
    for block in records.iter().filter(|record| {
        record.kind == "src-block"
            && record.field("language") == Some("scheme")
            && record.field("header").is_some_and(|header| {
                header
                    .split_ascii_whitespace()
                    .any(|part| part == ":org-elements")
            })
    }) {
        let owner = owning_headline(&records, block.id)
            .ok_or(":org-elements block requires an owning headline")?;
        if property(&records, owner, "QUERY_ID")?.is_none() {
            return Err(":org-elements block requires QUERY_ID".into());
        }
    }
    if seen.is_empty() {
        return Err("expected at least one QUERY_ID section".into());
    }
    output.push_str("))\n");
    Ok(output)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    let (input, output, interface_module) = match args.as_slice() {
        [_, input, output] => (input, output, "../interface.ss"),
        [_, input, output, interface] => (
            input,
            output,
            interface
                .to_str()
                .ok_or("interface module path must be valid UTF-8")?,
        ),
        _ => {
            return Err("usage: org_elements_tangle INPUT.org OUTPUT.ss [INTERFACE_MODULE]".into());
        }
    };
    let source = fs::read_to_string(Path::new(input))?;
    fs::write(Path::new(output), tangle(&source, interface_module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::tangle;

    #[test]
    fn tagged_queries_are_projected_from_org_elements() {
        let source = include_str!("../languages/org/v1/modules/org-elements/queries.org");
        let generated = tangle(source, "../interface.ss").expect("four named queries are admitted");
        assert_eq!(generated.matches("(org-element-query ").count(), 4);
        assert_eq!(
            generated,
            include_str!("../languages/org/v1/modules/org-elements/generated/query-source.ss")
        );
    }

    #[test]
    fn babel_without_feature_tag_cannot_define_a_query() {
        let source = "* Plain\n:PROPERTIES:\n:QUERY_ID: plain\n:END:\n#+begin_src scheme\n(org-elements headline)\n#+end_src\n";
        assert!(tangle(source, "../interface.ss").is_err());
    }

    #[test]
    fn missing_or_duplicate_ids_fail_closed() {
        let missing =
            "* Query\n#+begin_src scheme :org-elements\n(org-elements headline)\n#+end_src\n";
        assert!(tangle(missing, "../interface.ss").is_err());
        let duplicate = "* First\n:PROPERTIES:\n:QUERY_ID: same\n:END:\n#+begin_src scheme :org-elements\n(org-elements headline)\n#+end_src\n* Second\n:PROPERTIES:\n:QUERY_ID: same\n:END:\n#+begin_src scheme :org-elements\n(org-elements headline)\n#+end_src\n";
        assert!(tangle(duplicate, "../interface.ss").is_err());
    }

    #[test]
    fn consumer_can_select_its_own_interface_import() {
        let source = "* Query\n:PROPERTIES:\n:QUERY_ID: consumer.work\n:END:\n#+begin_src scheme :org-elements\n(org-elements headline (property todo-type \"todo\"))\n#+end_src\n";
        let generated = tangle(source, "/consumer/gerbil/org-elements/interface.ss")
            .expect("consumer query uses the same Org Elements interface");
        assert!(generated.contains("\"/consumer/gerbil/org-elements/interface.ss\""));
    }

    #[test]
    fn consumer_query_source_artifact_stays_in_sync() {
        let source = include_str!("../tests/fixtures/org-elements/customer-queries.org");
        let generated = tangle(
            source,
            "../../../../languages/org/v1/modules/org-elements/interface.ss",
        )
        .expect("consumer Org source declares one tagged query");
        assert_eq!(
            generated,
            include_str!("../tests/fixtures/org-elements/generated/customer-query-source.ss")
        );
    }
}
