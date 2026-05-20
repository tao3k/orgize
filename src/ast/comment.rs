use crate::SyntaxKind;

use super::{Comment, filter_token};

impl Comment {
    /// Contents without pound signs
    ///
    /// ```rust
    /// use orgize::{syntax_ast::Comment, Org};
    ///
    /// let fixed = Org::parse("# A\n#\n# B\n# C").first_node::<Comment>().unwrap();
    /// assert_eq!(fixed.value(), "A\n\nB\nC");
    /// ```
    pub fn value(&self) -> String {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::TEXT))
            .fold(String::new(), |acc, text| acc + &text)
    }
}
