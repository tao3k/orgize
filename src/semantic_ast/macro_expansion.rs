//! Opt-in macro expansion over the owned semantic AST.

use std::collections::HashMap;

use super::{AstRef, Document, MacroDefinition, MacroExpansion, MacroExpansionStatus, ObjectData};

impl<A: Clone> Document<A> {
    /// Return one opt-in expansion result for each semantic macro call.
    ///
    /// This does not rewrite the document or the lossless syntax tree. Callers
    /// can use the returned side table to decide if and where expansion should
    /// happen.
    pub fn macro_expansions(&self) -> Vec<MacroExpansion<A>> {
        let definitions = macro_definitions_by_name(&self.macro_definitions);
        let mut expansions = Vec::new();

        self.visit(|node| {
            let AstRef::Object(object) = node else {
                return;
            };
            let ObjectData::Macro { name, arguments } = &object.data else {
                return;
            };

            if let Some(definition) = definitions.get(name.as_str()) {
                expansions.push(MacroExpansion {
                    ann: object.ann.clone(),
                    name: name.clone(),
                    arguments: arguments.clone(),
                    template: Some(definition.template.clone()),
                    value: Some(expand_macro_template(&definition.template, arguments)),
                    status: MacroExpansionStatus::Expanded,
                });
            } else {
                expansions.push(MacroExpansion {
                    ann: object.ann.clone(),
                    name: name.clone(),
                    arguments: arguments.clone(),
                    template: None,
                    value: None,
                    status: MacroExpansionStatus::MissingDefinition,
                });
            }
        });

        expansions
    }
}

fn macro_definitions_by_name<A>(
    definitions: &[MacroDefinition<A>],
) -> HashMap<&str, &MacroDefinition<A>> {
    let mut by_name = HashMap::with_capacity(definitions.len());
    for definition in definitions {
        by_name.insert(definition.name.as_str(), definition);
    }
    by_name
}

fn expand_macro_template(template: &str, arguments: &[String]) -> String {
    let mut expanded =
        String::with_capacity(template.len() + arguments.iter().map(String::len).sum::<usize>());
    let mut all_arguments = None;
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '$' {
            expanded.push(ch);
            continue;
        }

        match chars.peek().copied() {
            Some('$') => {
                chars.next();
                expanded.push('$');
            }
            Some('0') => {
                chars.next();
                expanded.push_str(all_arguments.get_or_insert_with(|| arguments.join(", ")));
            }
            Some(digit) if digit.is_ascii_digit() => {
                chars.next();
                let index = digit
                    .to_digit(10)
                    .expect("ASCII digit must convert to a number")
                    .saturating_sub(1) as usize;
                if let Some(argument) = arguments.get(index) {
                    expanded.push_str(argument);
                }
            }
            _ => expanded.push('$'),
        }
    }

    expanded
}
