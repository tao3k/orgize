//! Backend-neutral LaTeX escaping and options for the Scheme AOT exporter.

use std::fmt::{self, Write as _};

/// Escape text for a LaTeX argument.
///
/// ```rust
/// use orgize::export::LatexEscape as Escape;
///
/// assert_eq!(format!("{}", Escape("a_b & 10%")), r"a\_b \& 10\%");
/// assert_eq!(
///     format!("{}", Escape(r"\path{a}")),
///     r"\textbackslash{}path\{a\}"
/// );
/// ```
pub struct LatexEscape<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> fmt::Display for LatexEscape<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.as_ref().chars() {
            match ch {
                '\\' => f.write_str(r"\textbackslash{}")?,
                '{' => f.write_str(r"\{")?,
                '}' => f.write_str(r"\}")?,
                '$' => f.write_str(r"\$")?,
                '&' => f.write_str(r"\&")?,
                '#' => f.write_str(r"\#")?,
                '%' => f.write_str(r"\%")?,
                '_' => f.write_str(r"\_")?,
                '^' => f.write_str(r"\textasciicircum{}")?,
                '~' => f.write_str(r"\textasciitilde{}")?,
                _ => f.write_char(ch)?,
            }
        }
        Ok(())
    }
}

/// Rendering options for the graph-backed LaTeX exporter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LatexExportOptions {
    /// Convert Org special strings such as `--`, `---`, and `...`.
    pub special_strings: bool,
    /// Expand Scheme-projected entities to their LaTeX backend value.
    pub expand_entities: bool,
}

impl Default for LatexExportOptions {
    fn default() -> Self {
        Self {
            special_strings: false,
            expand_entities: true,
        }
    }
}
