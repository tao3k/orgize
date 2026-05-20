use nom::{Compare, CompareResult, FindSubstring, Input as NomInput, Needed, Offset};
use std::{
    ops::{Bound, Deref, RangeBounds},
    str::{CharIndices, Chars},
};

use super::{
    SyntaxKind,
    green::{GreenElement, token},
};
use crate::config::ParseConfig;

/// A custom Input struct
///
/// It helps us to pass the `ParseConfig` all the way down to each parsers
#[derive(Clone, Copy, Debug)]
pub(crate) struct Input<'a> {
    pub(crate) s: &'a str,
    pub(crate) c: &'a ParseConfig,
}

impl<'a> Input<'a> {
    #[inline]
    pub(crate) fn of(&self, i: &'a str) -> Input<'a> {
        Input { s: i, c: self.c }
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &'a str {
        self.s
    }

    #[inline]
    pub(crate) fn slice<R>(&self, range: R) -> Input<'a>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.s.len(),
        };
        self.of(&self.s[start..end])
    }

    #[inline]
    pub(crate) fn take(&self, index: usize) -> Input<'a> {
        self.of(&self.s[..index])
    }

    #[inline]
    pub(crate) fn take_from(&self, index: usize) -> Input<'a> {
        self.of(&self.s[index..])
    }

    #[inline]
    pub(crate) fn take_split(&self, index: usize) -> (Input<'a>, Input<'a>) {
        let (prefix, suffix) = self.s.split_at(index);
        (self.of(suffix), self.of(prefix))
    }

    #[inline]
    pub(crate) fn token(&self, kind: SyntaxKind) -> GreenElement {
        token(kind, self.s)
    }

    #[inline]
    pub(crate) fn text_token(&self) -> GreenElement {
        token(SyntaxKind::TEXT, self.s)
    }

    #[inline]
    pub(crate) fn ws_token(&self) -> GreenElement {
        token(SyntaxKind::WHITESPACE, self.s)
    }

    #[inline]
    pub(crate) fn nl_token(&self) -> GreenElement {
        token(SyntaxKind::NEW_LINE, self.s)
    }
}

impl<'a> Deref for Input<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &'a str {
        self.s
    }
}

impl<'a> From<(&'a str, &'a ParseConfig)> for Input<'a> {
    fn from(value: (&'a str, &'a ParseConfig)) -> Self {
        Input {
            s: value.0,
            c: value.1,
        }
    }
}

impl<'a, 'b> FindSubstring<&'b str> for Input<'a> {
    fn find_substring(&self, substr: &str) -> Option<usize> {
        self.s.find(substr)
    }
}

impl<'a, 'b> Compare<&'b str> for Input<'a> {
    #[inline]
    fn compare(&self, t: &'b str) -> CompareResult {
        self.s.compare(t)
    }

    #[inline]
    fn compare_no_case(&self, t: &'b str) -> CompareResult {
        self.s.compare_no_case(t)
    }
}

impl<'a> NomInput for Input<'a> {
    type Item = char;
    type Iter = Chars<'a>;
    type IterIndices = CharIndices<'a>;

    #[inline]
    fn input_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn take(&self, index: usize) -> Self {
        self.take(index)
    }

    #[inline]
    fn take_from(&self, index: usize) -> Self {
        self.take_from(index)
    }

    #[inline]
    fn take_split(&self, index: usize) -> (Self, Self) {
        self.take_split(index)
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.s.find(predicate)
    }

    #[inline]
    fn iter_elements(&self) -> Self::Iter {
        self.s.chars()
    }

    #[inline]
    fn iter_indices(&self) -> Self::IterIndices {
        self.s.char_indices()
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.s
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once(self.s.len()))
            .nth(count)
            .ok_or(Needed::Unknown)
    }
}

impl<'a> Offset for Input<'a> {
    fn offset(&self, second: &Self) -> usize {
        self.s.offset(second.s)
    }
}
