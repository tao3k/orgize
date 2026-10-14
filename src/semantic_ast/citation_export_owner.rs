//! Typed admission of Scheme-owned citation export policy.
use super::{
    CitationBibliography, CitationExportOption, CitationExportPlan, CitationProcessor, Keyword,
    ParsedAnnotation, PrintBibliography, org_values,
};

pub(super) fn project(
    keywords: &[&Keyword<ParsedAnnotation>],
    plan: &mut CitationExportPlan<ParsedAnnotation>,
) {
    if !keywords.is_empty() {
        let fields: Vec<_> = keywords
            .iter()
            .flat_map(|keyword| [keyword.key.as_str(), keyword.value.as_str()])
            .collect();
        let mut previous = None;
        let mut printed = None;
        for row in org_values::rows("citation-export-keywords", &fields) {
            assert!(row.len() >= 2, "native citation row arity");
            let index: usize = row[0].parse().expect("native citation keyword index");
            assert!(
                previous.is_none_or(|previous| previous <= index),
                "native citation row order"
            );
            previous = Some(index);
            let keyword = keywords.get(index).expect("native citation index bounds");
            match row[1].as_str() {
                "bibliography" => plan.bibliographies.push(CitationBibliography {
                    ann: keyword.ann.clone(),
                    files: row[2..].to_vec(),
                    raw: keyword.value.clone(),
                }),
                "processor" => {
                    assert!(row.len() >= 4, "native processor arity");
                    let style = optional(&row[3..]);
                    plan.processors.push(CitationProcessor {
                        ann: keyword.ann.clone(),
                        processor: row[2].clone(),
                        style,
                        raw: keyword.value.clone(),
                    });
                }
                "print" => {
                    assert_eq!(row.len(), 2, "native print arity");
                    plan.print_bibliographies.push(PrintBibliography {
                        ann: keyword.ann.clone(),
                        options: Vec::new(),
                        raw: keyword.value.clone(),
                    });
                    printed = Some((index, plan.print_bibliographies.len() - 1));
                }
                "option" => {
                    assert!(row.len() >= 5, "native option arity");
                    let (owner, target) = printed.expect("native option requires print owner");
                    assert_eq!(owner, index, "native option owner index");
                    plan.print_bibliographies[target]
                        .options
                        .push(CitationExportOption {
                            key: row[2].clone(),
                            raw: row[3].clone(),
                            value: optional(&row[4..]),
                        });
                }
                _ => panic!("unknown native citation row"),
            }
        }
    }
    if !plan.citations.is_empty() {
        let styles: Vec<_> = plan
            .citations
            .iter()
            .map(|citation| citation.style.as_str())
            .collect();
        let rows = org_values::rows("citation-nocite", &styles);
        assert_eq!(rows.len(), styles.len(), "native citation style count");
        for (citation, row) in plan.citations.iter_mut().zip(rows) {
            assert_eq!(row.len(), 1, "native citation style arity");
            citation.nocite = match row[0].as_str() {
                "true" => true,
                "false" => false,
                _ => panic!("native citation boolean"),
            };
        }
    }
}

fn optional(fields: &[String]) -> Option<String> {
    match fields {
        [none] if none == "none" => None,
        [some, value] if some == "some" => Some(value.clone()),
        _ => panic!("native citation optional arity/tag"),
    }
}
