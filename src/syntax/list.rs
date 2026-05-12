use memchr::{memchr, memchr2};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{alphanumeric1, digit1, space0, space1},
    combinator::{cond, map, opt, recognize, verify},
    sequence::preceded,
    IResult, Parser,
};

use super::{
    combinator::{
        at_token, blank_lines, colon2_token, eol_or_eof, l_bracket_token, line_starts_iter, node,
        r_bracket_token, GreenElement,
    },
    input::Input,
    keyword::affiliated_keyword_nodes,
    object::standard_object_nodes,
    paragraph::paragraph_nodes,
    parser_contract::ElementNodeParser,
    SyntaxKind,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn list_node(
    input: Input,
    element_node: ElementNodeParser,
) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(|input| list_node_base(input, element_node), input)
}

fn list_node_base(
    input: Input,
    element_node: ElementNodeParser,
) -> IResult<Input, GreenElement, ()> {
    let (input, affiliated_keywords) = affiliated_keyword_nodes(input)?;
    let (input, first_indent) = space0(input)?;
    let (input, (ends_with_empty_blank_lines, first_item)) =
        list_item_node(first_indent, input, element_node)?;

    let mut children = vec![];
    children.extend(affiliated_keywords);
    children.push(first_item);

    let mut input = input;
    while !ends_with_empty_blank_lines && !input.is_empty() {
        let (input_, indent) = space0(input)?;

        if indent.len() != first_indent.len() {
            break;
        }

        let Ok((input_, (ends_with_empty_blank_lines, list_item))) =
            list_item_node(indent, input_, element_node)
        else {
            break;
        };

        children.push(list_item);
        debug_assert!(
            input.len() > input_.len(),
            "{} > {}",
            input.len(),
            input_.len(),
        );
        input = input_;

        if ends_with_empty_blank_lines {
            break;
        }
    }

    let (input, post_blank) = blank_lines(input)?;

    children.extend(post_blank);

    Ok((input, node(SyntaxKind::LIST, children)))
}

#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "debug", skip(input, indent), fields(input = input.s))
)]
fn list_item_node<'a>(
    indent: Input<'a>,
    input: Input<'a>,
    element_node: ElementNodeParser,
) -> IResult<Input<'a>, (bool, GreenElement), ()> {
    let (input, bullet) = recognize((
        alt((
            tag("+"),
            tag("*"),
            tag("-"),
            preceded(digit1, tag(".")),
            preceded(digit1, tag(")")),
        )),
        alt((space1, eol_or_eof)),
    ))
    .parse(input)?;

    // list item cannot have an asterisk at the beginning of line
    if indent.is_empty() && bullet.s.starts_with('*') {
        return Err(nom::Err::Error(()));
    }

    if input.is_empty() {
        return Ok((
            input,
            (
                false,
                node(
                    SyntaxKind::LIST_ITEM,
                    [
                        indent.token(SyntaxKind::LIST_ITEM_INDENT),
                        bullet.token(SyntaxKind::LIST_ITEM_BULLET),
                    ],
                ),
            ),
        ));
    }

    let is_ordered = bullet.s.starts_with(|c: char| c.is_ascii_digit());
    let (input, counter) = opt(list_item_counter).parse(input)?;
    let (input, checkbox) = opt(list_item_checkbox).parse(input)?;
    let (input, tag) = cond(!is_ordered, opt(list_item_tag)).parse(input)?;
    let (input, (ends_with_empty_blank_lines, content)) =
        list_item_content_node(input, indent.len(), element_node)?;
    let (input, post_blank) = cond(!ends_with_empty_blank_lines, blank_lines).parse(input)?;

    let mut children = vec![
        indent.token(SyntaxKind::LIST_ITEM_INDENT),
        bullet.token(SyntaxKind::LIST_ITEM_BULLET),
    ];

    if let Some((counter, ws)) = counter {
        children.extend([counter, ws.ws_token()]);
    }
    if let Some((checkbox, ws)) = checkbox {
        children.extend([checkbox, ws.ws_token()]);
    }
    if let Some(Some((tag, ws))) = tag {
        children.extend([tag, ws.ws_token()]);
    }

    children.push(content);
    if let Some(post_blank) = post_blank {
        children.extend(post_blank);
    }

    Ok((
        input,
        (
            ends_with_empty_blank_lines,
            node(SyntaxKind::LIST_ITEM, children),
        ),
    ))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
fn list_item_counter(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    let (input, node) = map(
        (l_bracket_token, at_token, alphanumeric1, r_bracket_token),
        |(l_bracket, at, char, r_bracket)| {
            node(
                SyntaxKind::LIST_ITEM_COUNTER,
                [l_bracket, at, char.text_token(), r_bracket],
            )
        },
    )
    .parse(input)?;

    let (input, ws) = space0(input)?;

    Ok((input, (node, ws)))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
fn list_item_checkbox(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    let (input, node) = map(
        (
            l_bracket_token,
            verify(take(1usize), |input: &Input| {
                input.s == " " || input.s == "X" || input.s == "-"
            }),
            r_bracket_token,
        ),
        |(l_bracket, char, r_bracket)| {
            node(
                SyntaxKind::LIST_ITEM_CHECK_BOX,
                [l_bracket, char.text_token(), r_bracket],
            )
        },
    )
    .parse(input)?;

    let (input, ws) = space0(input)?;

    Ok((input, (node, ws)))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
fn list_item_tag(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    let bytes = input.as_bytes();

    let (input, tag) = match memchr2(b'\n', b':', bytes) {
        Some(idx) if idx > 0 && bytes[idx] == b':' => input.take_split(idx),
        _ => return Err(nom::Err::Error(())),
    };
    let (input, ws) = space0(input)?;
    let (input, colon2) = colon2_token(input)?;

    let mut children = standard_object_nodes(tag);
    children.push(colon2);

    Ok((input, (node(SyntaxKind::LIST_ITEM_TAG, children), ws)))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
fn list_item_content_node(
    input: Input,
    indent: usize,
    element_node: ElementNodeParser,
) -> IResult<Input, (bool, GreenElement), ()> {
    if memchr(b'\n', input.as_bytes()).is_none() {
        return Ok((
            input.of(""),
            (
                false,
                node(
                    SyntaxKind::LIST_ITEM_CONTENT,
                    [node(SyntaxKind::PARAGRAPH, standard_object_nodes(input))],
                ),
            ),
        ));
    };

    ListItemContentParser::new(input, indent, element_node).finish()
}

type ListItemContentResult<'a> = IResult<Input<'a>, (bool, GreenElement), ()>;

enum ListItemContentStep<'a> {
    Continue,
    Done(ListItemContentResult<'a>),
}

struct ListItemContentParser<'a> {
    root: Input<'a>,
    cursor: Input<'a>,
    indent: usize,
    element_node: ElementNodeParser,
    skip_one: bool,
    children: Vec<GreenElement>,
    previous_blank_line: Option<(Input<'a>, Input<'a>)>,
}

impl<'a> ListItemContentParser<'a> {
    fn new(input: Input<'a>, indent: usize, element_node: ElementNodeParser) -> Self {
        Self {
            root: input,
            cursor: input,
            indent,
            element_node,
            skip_one: true,
            children: Vec::new(),
            previous_blank_line: None,
        }
    }

    fn finish(mut self) -> ListItemContentResult<'a> {
        std::iter::from_fn(|| self.next_step())
            .find_map(|step| match step {
                ListItemContentStep::Continue => None,
                ListItemContentStep::Done(result) => Some(result),
            })
            .unwrap_or_else(|| self.finish_with(self.root.of(""), false))
    }

    fn next_step(&mut self) -> Option<ListItemContentStep<'a>> {
        if self.cursor.is_empty() {
            return Some(ListItemContentStep::Done(
                self.finish_with(self.root.of(""), false),
            ));
        }

        let cursor = self.cursor;
        let offsets = line_starts_iter(cursor.as_str())
            // The first line in list item content is always a paragraph, so the
            // first scan skips it.
            .skip(usize::from(self.skip_one))
            .collect::<Vec<_>>();

        for (input, head) in offsets.into_iter().map(|idx| cursor.take_split(idx)) {
            let previous_cursor_len = self.cursor.len();
            match self.step_at_line(input, head) {
                ListItemContentStep::Continue if self.cursor.len() == previous_cursor_len => {
                    continue;
                }
                ListItemContentStep::Continue => return Some(ListItemContentStep::Continue),
                done => return Some(done),
            }
        }

        Some(ListItemContentStep::Done(
            self.finish_with_remaining_cursor(),
        ))
    }

    fn step_at_line(&mut self, input: Input<'a>, head: Input<'a>) -> ListItemContentStep<'a> {
        match get_line_indent(input.as_str()) {
            Some(next_indent) if next_indent <= self.indent => {
                let (input, head) = self.previous_blank_line.take().unwrap_or((input, head));
                self.push_paragraph_head(head)
                    .map(|_| self.finish_with(input, false))
                    .map_or_else(
                        |err| ListItemContentStep::Done(Err(err)),
                        ListItemContentStep::Done,
                    )
            }
            Some(_) => self.step_at_nested_line(input, head),
            None => self.step_at_blank_line(input, head),
        }
    }

    fn step_at_nested_line(
        &mut self,
        input: Input<'a>,
        head: Input<'a>,
    ) -> ListItemContentStep<'a> {
        self.previous_blank_line = None;
        if let Ok((tail, element)) = (self.element_node)(input) {
            if let Err(err) = self.push_paragraph_head(head) {
                return ListItemContentStep::Done(Err(err));
            }
            self.children.push(element);
            debug_assert!(
                tail.len() < self.cursor.len(),
                "{} < {}",
                tail.len(),
                self.cursor.len()
            );
            self.cursor = tail;
            self.skip_one = false;
            return ListItemContentStep::Continue;
        }

        ListItemContentStep::Continue
    }

    fn step_at_blank_line(&mut self, input: Input<'a>, head: Input<'a>) -> ListItemContentStep<'a> {
        if let Some((input, head)) = self.previous_blank_line.take() {
            self.push_paragraph_head(head)
                .map(|_| self.finish_with(input, true))
                .map_or_else(
                    |err| ListItemContentStep::Done(Err(err)),
                    ListItemContentStep::Done,
                )
        } else {
            self.previous_blank_line = Some((input, head));
            ListItemContentStep::Continue
        }
    }

    fn push_paragraph_head(&mut self, head: Input<'a>) -> Result<(), nom::Err<()>> {
        if !head.is_empty() {
            self.children.extend(paragraph_nodes(head)?);
        }
        Ok(())
    }

    fn finish_with_remaining_cursor(&mut self) -> ListItemContentResult<'a> {
        self.push_paragraph_head(self.cursor)?;
        Ok((
            self.root.of(""),
            (
                false,
                node(
                    SyntaxKind::LIST_ITEM_CONTENT,
                    std::mem::take(&mut self.children),
                ),
            ),
        ))
    }

    fn finish_with(&mut self, input: Input<'a>, has_more: bool) -> ListItemContentResult<'a> {
        Ok((
            input,
            (
                has_more,
                node(
                    SyntaxKind::LIST_ITEM_CONTENT,
                    std::mem::take(&mut self.children),
                ),
            ),
        ))
    }
}

fn get_line_indent(input: &str) -> Option<usize> {
    input
        .bytes()
        .take_while(|b| *b != b'\n')
        .position(|b| !b.is_ascii_whitespace())
}

#[test]
fn parse() {
    use crate::{syntax_ast::SyntaxList, tests::to_ast, ParseConfig};

    let to_list =
        to_ast::<SyntaxList>(|input| list_node(input, crate::syntax::element::element_node));

    insta::assert_debug_snapshot!(
        to_list("1)").syntax,
        @r###"
    LIST@0..2
      LIST_ITEM@0..2
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "1)"
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("+ ").syntax,
        @r###"
    LIST@0..2
      LIST_ITEM@0..2
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("-\n").syntax,
        @r###"
    LIST@0..2
      LIST_ITEM@0..2
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "-\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("+ 1").syntax,
        @r###"
    LIST@0..3
      LIST_ITEM@0..3
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_CONTENT@2..3
          PARAGRAPH@2..3
            TEXT@2..3 "1"
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("+ 1\n").syntax,
        @r###"
    LIST@0..4
      LIST_ITEM@0..4
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_CONTENT@2..4
          PARAGRAPH@2..4
            TEXT@2..4 "1\n"
    "###
    );

    // list ends with two consecutive blank lines, and these blank lines
    // will be the post_blank of list node
    insta::assert_debug_snapshot!(
        to_list("+ [@A] 1\n\n\n+ 2").syntax,
        @r###"
    LIST@0..11
      LIST_ITEM@0..9
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_COUNTER@2..6
          L_BRACKET@2..3 "["
          AT@3..4 "@"
          TEXT@4..5 "A"
          R_BRACKET@5..6 "]"
        WHITESPACE@6..7 " "
        LIST_ITEM_CONTENT@7..9
          PARAGRAPH@7..9
            TEXT@7..9 "1\n"
      BLANK_LINE@9..10 "\n"
      BLANK_LINE@10..11 "\n"
    "###
    );

    // empty line between list item, the empty line will be
    // the post_blank of first item
    insta::assert_debug_snapshot!(
        to_list("+ *TAG* :: item1\n\n+ [X] item2").syntax,
        @r###"
    LIST@0..29
      LIST_ITEM@0..18
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_TAG@2..10
          BOLD@2..7
            STAR@2..3 "*"
            TEXT@3..6 "TAG"
            STAR@6..7 "*"
          TEXT@7..8 " "
          COLON2@8..10 "::"
        WHITESPACE@10..10 ""
        LIST_ITEM_CONTENT@10..17
          PARAGRAPH@10..17
            TEXT@10..17 " item1\n"
        BLANK_LINE@17..18 "\n"
      LIST_ITEM@18..29
        LIST_ITEM_INDENT@18..18 ""
        LIST_ITEM_BULLET@18..20 "+ "
        LIST_ITEM_CHECK_BOX@20..23
          L_BRACKET@20..21 "["
          TEXT@21..22 "X"
          R_BRACKET@22..23 "]"
        WHITESPACE@23..24 " "
        LIST_ITEM_CONTENT@24..29
          PARAGRAPH@24..29
            TEXT@24..29 "item2"
    "###
    );

    // nested list
    let list = to_list(
        r#"+ item1
  + item2"#,
    );
    insta::assert_debug_snapshot!(
        list.syntax,
        @r###"
    LIST@0..17
      LIST_ITEM@0..17
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_CONTENT@2..17
          PARAGRAPH@2..8
            TEXT@2..8 "item1\n"
          LIST@8..17
            LIST_ITEM@8..17
              LIST_ITEM_INDENT@8..10 "  "
              LIST_ITEM_BULLET@10..12 "+ "
              LIST_ITEM_CONTENT@12..17
                PARAGRAPH@12..17
                  TEXT@12..17 "item2"
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("+ item1\nitem2").syntax,
        @r###"
    LIST@0..8
      LIST_ITEM@0..8
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_CONTENT@2..8
          PARAGRAPH@2..8
            TEXT@2..8 "item1\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("+ item1\n\n  still item 1").syntax,
        @r###"
    LIST@0..23
      LIST_ITEM@0..23
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_CONTENT@2..23
          PARAGRAPH@2..9
            TEXT@2..8 "item1\n"
            BLANK_LINE@8..9 "\n"
          PARAGRAPH@9..23
            TEXT@9..23 "  still item 1"
    "###
    );

    let list = to_list(
        r#"+ item1
      + item2
    "#,
    );
    insta::assert_debug_snapshot!(
        list.syntax,
        @r###"
    LIST@0..26
      LIST_ITEM@0..26
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..2 "+ "
        LIST_ITEM_CONTENT@2..26
          PARAGRAPH@2..8
            TEXT@2..8 "item1\n"
          LIST@8..26
            LIST_ITEM@8..26
              LIST_ITEM_INDENT@8..14 "      "
              LIST_ITEM_BULLET@14..16 "+ "
              LIST_ITEM_CONTENT@16..26
                PARAGRAPH@16..26
                  TEXT@16..22 "item2\n"
                  BLANK_LINE@22..26 "    "
    "###
    );

    let list = to_list(
        r#"1. item1

    - item2

3. item 3"#,
    );
    assert!(list.is_ordered());
    insta::assert_debug_snapshot!(
        list.syntax,
        @r###"
    LIST@0..32
      LIST_ITEM@0..23
        LIST_ITEM_INDENT@0..0 ""
        LIST_ITEM_BULLET@0..3 "1. "
        LIST_ITEM_CONTENT@3..23
          PARAGRAPH@3..10
            TEXT@3..9 "item1\n"
            BLANK_LINE@9..10 "\n"
          LIST@10..23
            LIST_ITEM@10..23
              LIST_ITEM_INDENT@10..14 "    "
              LIST_ITEM_BULLET@14..16 "- "
              LIST_ITEM_CONTENT@16..22
                PARAGRAPH@16..22
                  TEXT@16..22 "item2\n"
              BLANK_LINE@22..23 "\n"
      LIST_ITEM@23..32
        LIST_ITEM_INDENT@23..23 ""
        LIST_ITEM_BULLET@23..26 "3. "
        LIST_ITEM_CONTENT@26..32
          PARAGRAPH@26..32
            TEXT@26..32 "item 3"
    "###
    );

    // nested list
    insta::assert_debug_snapshot!(
        to_list("  + item1\n\n  + item2").syntax,
        @r###"
    LIST@0..20
      LIST_ITEM@0..11
        LIST_ITEM_INDENT@0..2 "  "
        LIST_ITEM_BULLET@2..4 "+ "
        LIST_ITEM_CONTENT@4..10
          PARAGRAPH@4..10
            TEXT@4..10 "item1\n"
        BLANK_LINE@10..11 "\n"
      LIST_ITEM@11..20
        LIST_ITEM_INDENT@11..13 "  "
        LIST_ITEM_BULLET@13..15 "+ "
        LIST_ITEM_CONTENT@15..20
          PARAGRAPH@15..20
            TEXT@15..20 "item2"
    "###
    );

    insta::assert_debug_snapshot!(
        to_list("  1. item1\n        2. item2\n      3. item3").syntax,
        @r###"
    LIST@0..42
      LIST_ITEM@0..42
        LIST_ITEM_INDENT@0..2 "  "
        LIST_ITEM_BULLET@2..5 "1. "
        LIST_ITEM_CONTENT@5..42
          PARAGRAPH@5..11
            TEXT@5..11 "item1\n"
          LIST@11..28
            LIST_ITEM@11..28
              LIST_ITEM_INDENT@11..19 "        "
              LIST_ITEM_BULLET@19..22 "2. "
              LIST_ITEM_CONTENT@22..28
                PARAGRAPH@22..28
                  TEXT@22..28 "item2\n"
          LIST@28..42
            LIST_ITEM@28..42
              LIST_ITEM_INDENT@28..34 "      "
              LIST_ITEM_BULLET@34..37 "3. "
              LIST_ITEM_CONTENT@37..42
                PARAGRAPH@37..42
                  TEXT@37..42 "item3"
    "###
    );

    // Indentation of lines within other greater elements do not count
    insta::assert_debug_snapshot!(
        to_list("  1. item1\n    #+begin_example\nhello\n#+end_example\n").syntax,
        @r###"
    LIST@0..51
      LIST_ITEM@0..51
        LIST_ITEM_INDENT@0..2 "  "
        LIST_ITEM_BULLET@2..5 "1. "
        LIST_ITEM_CONTENT@5..51
          PARAGRAPH@5..11
            TEXT@5..11 "item1\n"
          EXAMPLE_BLOCK@11..51
            BLOCK_BEGIN@11..31
              WHITESPACE@11..15 "    "
              TEXT@15..23 "#+begin_"
              TEXT@23..30 "example"
              NEW_LINE@30..31 "\n"
            BLOCK_CONTENT@31..37
              TEXT@31..37 "hello\n"
            BLOCK_END@37..51
              TEXT@37..43 "#+end_"
              TEXT@43..50 "example"
              NEW_LINE@50..51 "\n"
    "###
    );

    to_list("- ");
    to_list("-\t");
    to_list("-\r");
    to_list("-\t\n");
    to_list("-\r\n");
    to_list("-");

    let config = &ParseConfig::default();

    assert!(list_node(("-a", config).into(), crate::syntax::element::element_node).is_err());
    assert!(list_node(
        ("*\r\n", config).into(),
        crate::syntax::element::element_node
    )
    .is_err());
    assert!(list_node(("* ", config).into(), crate::syntax::element::element_node).is_err());
}
