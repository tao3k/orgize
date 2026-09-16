//! Typed rendering for named Org source-block templates.

use std::{collections::BTreeMap, error::Error, fmt};

use super::{Document, ParsedAnnotation, SourceBlockRecordKind};

/// One unique, language-checked source block ready for repeated rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedSourceBlockTemplate {
    name: String,
    language: String,
    value: String,
}

impl NamedSourceBlockTemplate {
    /// Expands `{{{NAME}}}` bindings without reparsing the owning Org document.
    pub fn render<'a, I>(&self, bindings: I) -> Result<String, NamedSourceBlockTemplateError>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let bindings = bindings.into_iter().collect::<BTreeMap<_, _>>();
        render_template_macros(&self.value, &bindings)
    }

    /// Returns the canonical block name resolved by the Org AST.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source-block language checked during compilation.
    pub fn language(&self) -> &str {
        &self.language
    }
}

/// Failure emitted while resolving or rendering a named source-block template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedSourceBlockTemplateError {
    MissingBlock {
        name: String,
    },
    DuplicateBlock {
        name: String,
        count: usize,
    },
    LanguageMismatch {
        name: String,
        expected: String,
        actual: Option<String>,
    },
    InvalidMacro {
        name: String,
    },
    MissingBinding {
        name: String,
    },
    UnusedBinding {
        name: String,
    },
    UnterminatedMacro,
}

impl fmt::Display for NamedSourceBlockTemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBlock { name } => {
                write!(formatter, "named Org source block `{name}` is missing")
            }
            Self::DuplicateBlock { name, count } => write!(
                formatter,
                "named Org source block `{name}` must be unique, found {count}"
            ),
            Self::LanguageMismatch {
                name,
                expected,
                actual,
            } => write!(
                formatter,
                "named Org source block `{name}` must use language `{expected}`, found `{}`",
                actual.as_deref().unwrap_or("<none>")
            ),
            Self::InvalidMacro { name } => {
                write!(
                    formatter,
                    "Org source-block template macro `{name}` is invalid"
                )
            }
            Self::MissingBinding { name } => {
                write!(
                    formatter,
                    "Org source-block template macro `{name}` is unbound"
                )
            }
            Self::UnusedBinding { name } => {
                write!(
                    formatter,
                    "Org source-block template binding `{name}` is unused"
                )
            }
            Self::UnterminatedMacro => {
                formatter.write_str("Org source-block template macro is unterminated")
            }
        }
    }
}

impl Error for NamedSourceBlockTemplateError {}

impl Document<ParsedAnnotation> {
    /// Compiles one unique named block for repeated rendering.
    pub fn named_source_block_template(
        &self,
        name: &str,
        expected_language: &str,
    ) -> Result<NamedSourceBlockTemplate, NamedSourceBlockTemplateError> {
        let matching = self
            .source_block_records()
            .into_iter()
            .filter(|record| {
                record.kind == SourceBlockRecordKind::Block && record.name.as_deref() == Some(name)
            })
            .collect::<Vec<_>>();
        let record = match matching.as_slice() {
            [] => {
                return Err(NamedSourceBlockTemplateError::MissingBlock {
                    name: name.to_string(),
                });
            }
            [record] => record,
            records => {
                return Err(NamedSourceBlockTemplateError::DuplicateBlock {
                    name: name.to_string(),
                    count: records.len(),
                });
            }
        };
        if record.language.as_deref() != Some(expected_language) {
            return Err(NamedSourceBlockTemplateError::LanguageMismatch {
                name: name.to_string(),
                expected: expected_language.to_string(),
                actual: record.language.clone(),
            });
        }
        Ok(NamedSourceBlockTemplate {
            name: name.to_string(),
            language: expected_language.to_string(),
            value: record.value.clone(),
        })
    }

    /// Resolves exactly one named block through the semantic AST and expands
    /// `{{{NAME}}}` bindings. Missing, duplicate, malformed, and unused facts
    /// fail closed so consumers never silently render a partial contract.
    pub fn render_named_source_block_template<'a, I>(
        &self,
        name: &str,
        expected_language: &str,
        bindings: I,
    ) -> Result<String, NamedSourceBlockTemplateError>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        self.named_source_block_template(name, expected_language)?
            .render(bindings)
    }
}

fn render_template_macros(
    template: &str,
    bindings: &BTreeMap<&str, &str>,
) -> Result<String, NamedSourceBlockTemplateError> {
    let mut rendered = String::with_capacity(template.len());
    let mut remaining = template;
    let mut used = BTreeMap::<&str, ()>::new();
    while let Some(start) = remaining.find("{{{") {
        rendered.push_str(&remaining[..start]);
        let macro_source = &remaining[start + 3..];
        let Some(end) = macro_source.find("}}}") else {
            return Err(NamedSourceBlockTemplateError::UnterminatedMacro);
        };
        let name = &macro_source[..end];
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(NamedSourceBlockTemplateError::InvalidMacro {
                name: name.to_string(),
            });
        }
        let value =
            bindings
                .get(name)
                .ok_or_else(|| NamedSourceBlockTemplateError::MissingBinding {
                    name: name.to_string(),
                })?;
        rendered.push_str(value);
        used.insert(name, ());
        remaining = &macro_source[end + 3..];
    }
    rendered.push_str(remaining);
    if let Some(unused) = bindings
        .keys()
        .find(|binding| !used.contains_key(**binding))
    {
        return Err(NamedSourceBlockTemplateError::UnusedBinding {
            name: (*unused).to_string(),
        });
    }
    Ok(rendered.trim().to_string())
}
