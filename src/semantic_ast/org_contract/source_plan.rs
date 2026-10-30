//! Call-local admission of one Scheme-owned document source plan.
use super::blocks::{self, ContractBlock};
use crate::ast::{
    ASSERT_ID_PROPERTY, ASSERT_SEVERITY_PROPERTY, CONTRACT_ALIAS_PROPERTY, CONTRACT_ID_PROPERTY,
    CONTRACT_KIND_PROPERTY, CONTRACT_SCOPE_PROPERTY, Document, OrgContractKind, OrgContractScope,
    OrgContractSeverity, ParsedAnnotation, Section, SourceBlockSyntaxRecord,
};
use crate::org_aot::NativeExpressionValue;
use std::collections::HashMap;

pub(super) struct SectionFacts {
    values: [Option<String>; 6],
    pub(super) kind: Option<OrgContractKind>,
    pub(super) scope: Option<OrgContractScope>,
    pub(super) severity: Option<OrgContractSeverity>,
    pub(super) aliases: Vec<String>,
}
impl SectionFacts {
    pub(super) fn value(&self, key: &str) -> Option<String> {
        let i = match key {
            CONTRACT_ID_PROPERTY => 0,
            CONTRACT_KIND_PROPERTY => 1,
            CONTRACT_SCOPE_PROPERTY => 2,
            CONTRACT_ALIAS_PROPERTY => 3,
            ASSERT_ID_PROPERTY => 4,
            ASSERT_SEVERITY_PROPERTY => 5,
            _ => panic!("unknown admitted Contract field"),
        };
        self.values[i].clone()
    }
}
pub(super) struct SourcePlan<'a> {
    // Call-local Rust section identity only; never dereferenced or sent to Scheme.
    source: &'a Document<ParsedAnnotation>,
    sections: HashMap<usize, SectionFacts>,
    pub(super) blocks: Vec<ContractBlock>,
}
impl std::ops::Deref for SourcePlan<'_> {
    type Target = [ContractBlock];
    fn deref(&self) -> &Self::Target {
        &self.blocks
    }
}
impl SourcePlan<'_> {
    pub(super) fn section(&self, section: &Section<ParsedAnnotation>) -> &SectionFacts {
        let _ = self.source;
        self.sections
            .get(&(std::ptr::from_ref(section) as usize))
            .expect("section belongs to the source plan")
    }
}
fn sections<'a>(
    input: &'a [Section<ParsedAnnotation>],
    out: &mut Vec<&'a Section<ParsedAnnotation>>,
) {
    for section in input {
        out.push(section);
        sections(&section.subsections, out);
    }
}
pub(super) fn plan_document(document: &Document<ParsedAnnotation>) -> SourcePlan<'_> {
    plan_document_records(document, document.source_block_syntax_records())
}
pub(super) fn plan_document_records(
    document: &Document<ParsedAnnotation>,
    records: Vec<SourceBlockSyntaxRecord>,
) -> SourcePlan<'_> {
    let mut source_sections = Vec::new();
    sections(&document.sections, &mut source_sections);
    if source_sections.is_empty() && records.is_empty() {
        return SourcePlan {
            source: document,
            sections: HashMap::new(),
            blocks: Vec::new(),
        };
    }
    let mut fields = vec![source_sections.len().to_string()];
    for section in &source_sections {
        fields.push(section.properties.len().to_string());
        for property in &section.properties {
            fields.extend([property.key.clone(), property.value.clone()]);
        }
    }
    fields.push(records.len().to_string());
    for record in &records {
        fields.extend([
            record.language.clone().unwrap_or_default(),
            record.name.clone().unwrap_or_default(),
            record.parameters.clone().unwrap_or_default(),
            record.value.clone(),
        ]);
    }
    let fields = fields.iter().map(String::as_str).collect::<Vec<_>>();
    let rows = crate::ast::org_values::rows("contract-document-plan", &fields);
    let mut facts = (0..source_sections.len()).map(|_| None).collect::<Vec<_>>();
    let mut blocks = (0..records.len()).map(|_| None).collect::<Vec<_>>();
    let mut records = records.into_iter().map(Some).collect::<Vec<_>>();
    let mut aliases = vec![Vec::new(); source_sections.len()];
    let mut streams: HashMap<(usize, String), Option<Vec<Vec<String>>>> = HashMap::new();
    for row in rows {
        assert!(row.len() >= 2, "native Contract source row header");
        let index: usize = row[1].parse().expect("native source index");
        match row[0].as_str() {
            "section" => {
                let slot = facts.get_mut(index).expect("native section bounds");
                assert!(slot.is_none(), "duplicate native section");
                assert_eq!(row.len(), 17, "native section arity");
                let values = std::array::from_fn(|i| {
                    blocks::optional(&row[2 + i * 2], row[3 + i * 2].clone())
                });
                *slot = Some(SectionFacts {
                    values,
                    kind: blocks::kind(&row[14]),
                    scope: blocks::scope(&row[15]),
                    severity: blocks::severity(&row[16]),
                    aliases: Vec::new(),
                });
            }
            "alias" => {
                assert_eq!(row.len(), 3, "native alias arity");
                aliases
                    .get_mut(index)
                    .expect("native alias section")
                    .push(row[2].clone());
            }
            "block" => {
                let record = records
                    .get_mut(index)
                    .expect("native block bounds")
                    .take()
                    .expect("duplicate native block");
                blocks[index] = Some(blocks::admit_block(record, row[2..].to_vec()));
            }
            "forms" => {
                assert_eq!(row.len(), 4, "native forms arity");
                assert!(index < blocks.len(), "native forms bounds");
                assert!(
                    matches!(row[2].as_str(), "query" | "contract" | "expect"),
                    "native forms mode"
                );
                let stream = match row[3].as_str() {
                    "ok" => Some(Vec::new()),
                    "invalid" => None,
                    _ => panic!("native forms status"),
                };
                assert!(
                    streams.insert((index, row[2].clone()), stream).is_none(),
                    "duplicate native forms"
                );
            }
            "form" => {
                assert_eq!(row.len(), 5, "native form arity");
                streams
                    .get_mut(&(index, row[2].clone()))
                    .expect("native form header")
                    .as_mut()
                    .expect("forms follow valid header")
                    .push(row[3..].to_vec());
            }
            _ => panic!("unknown native Contract source row"),
        }
    }
    let sections = source_sections
        .into_iter()
        .enumerate()
        .map(|(i, section)| {
            let mut fact = facts[i].take().expect("missing native section");
            fact.aliases = std::mem::take(&mut aliases[i]);
            (std::ptr::from_ref(section) as usize, fact)
        })
        .collect();
    let blocks = blocks
        .into_iter()
        .enumerate()
        .map(|(i, block)| {
            let mut block = block.expect("missing native block");
            if matches!(block.role.as_str(), "query" | "selector" | "contract") {
                block.query_forms = stream(&mut streams, i, "query");
            }
            if block.role == "contract" {
                block.contract_forms = stream(&mut streams, i, "contract");
            }
            if block.role == "expect" {
                block.expect_forms = stream(&mut streams, i, "expect");
            }
            block
        })
        .collect();
    assert!(streams.is_empty(), "unexpected native form streams");
    SourcePlan {
        source: document,
        sections,
        blocks,
    }
}
fn stream(
    streams: &mut HashMap<(usize, String), Option<Vec<Vec<String>>>>,
    index: usize,
    mode: &str,
) -> Option<Vec<NativeExpressionValue>> {
    streams
        .remove(&(index, mode.to_owned()))
        .expect("missing native form stream")
        .map(|rows| {
            let mut stack = vec![Vec::new()];
            for row in rows {
                match row[0].as_str() {
                    "1" => {
                        assert!(row[1].is_empty());
                        stack.push(Vec::new());
                    }
                    "2" => {
                        assert!(row[1].is_empty());
                        assert!(stack.len() > 1, "native form stack");
                        let value = NativeExpressionValue::List(stack.pop().unwrap());
                        stack.last_mut().unwrap().push(value);
                    }
                    "3" | "4" => stack.last_mut().unwrap().push(if row[0] == "3" {
                        NativeExpressionValue::Atom(row[1].clone())
                    } else {
                        NativeExpressionValue::String(row[1].clone())
                    }),
                    _ => panic!("native form tag"),
                }
            }
            assert_eq!(stack.len(), 1, "unfinished native form");
            stack.pop().unwrap()
        })
}
