//! Typed admission of Scheme-owned MACRO definition fields.

use crate::TextRange;

use super::{Keyword, MacroDefinition, ParsedAnnotation};
use crate::GraphRecord;

pub(super) fn macro_definition(
    record: &GraphRecord,
    keyword: Keyword<ParsedAnnotation>,
) -> Result<MacroDefinition<ParsedAnnotation>, (TextRange, String)> {
    let range = keyword.ann.range;
    let name = record
        .field("macro-name")
        .ok_or_else(|| (range, "MACRO keyword is missing a valid macro name".into()))?;

    Ok(MacroDefinition {
        ann: keyword.ann,
        name: name.to_string(),
        template: record
            .field("macro-template")
            .expect("native MACRO template")
            .to_owned(),
        raw_value: keyword.value,
    })
}
