use nom::{
    bytes::complete::{tag, tag_no_case, take_while, take_while1},
    character::complete::{alpha1, space0, space1},
    combinator::opt,
    IResult, Parser,
};

use super::{
    combinator::{
        blank_lines, eol_or_eof, line_starts_iter, node, token, trim_line_end, GreenElement,
        NodeBuilder,
    },
    element::element_nodes,
    input::Input,
    keyword::affiliated_keyword_nodes,
    SyntaxKind,
};

fn block_node_base(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, affiliated_keywords) = affiliated_keyword_nodes(input)?;
    let (input, (block_begin, name)) = block_begin_node(input)?;
    let (input, pre_blank) = blank_lines(input)?;

    let kind = match name {
        s if s.eq_ignore_ascii_case("COMMENT") => SyntaxKind::COMMENT_BLOCK,
        s if s.eq_ignore_ascii_case("EXAMPLE") => SyntaxKind::EXAMPLE_BLOCK,
        s if s.eq_ignore_ascii_case("EXPORT") => SyntaxKind::EXPORT_BLOCK,
        s if s.eq_ignore_ascii_case("SRC") => SyntaxKind::SOURCE_BLOCK,
        s if s.eq_ignore_ascii_case("CENTER") => SyntaxKind::CENTER_BLOCK,
        s if s.eq_ignore_ascii_case("QUOTE") => SyntaxKind::QUOTE_BLOCK,
        s if s.eq_ignore_ascii_case("VERSE") => SyntaxKind::VERSE_BLOCK,
        _ => SyntaxKind::SPECIAL_BLOCK,
    };

    for (input, contents) in line_starts_iter(&input).map(|i| input.take_split(i)) {
        if let Ok((input, block_end)) = block_end_node(input, name) {
            let (input, post_blank) = blank_lines(input)?;

            let mut children = vec![];
            children.extend(affiliated_keywords);
            children.push(block_begin);
            children.extend(pre_blank);
            if kind.is_greater_element() {
                children.push(node(SyntaxKind::BLOCK_CONTENT, element_nodes(contents)?));
            } else {
                children.push(node(
                    SyntaxKind::BLOCK_CONTENT,
                    comma_quoted_text_nodes(contents),
                ));
            }
            children.push(block_end);
            children.extend(post_blank);
            return Ok((input, node(kind, children)));
        }
    }

    Err(nom::Err::Error(()))
}

fn block_begin_node(input: Input<'_>) -> IResult<Input<'_>, (GreenElement, &str), ()> {
    let (input, (ws1, begin, name)) = (space0, tag_no_case("#+BEGIN_"), alpha1).parse(input)?;

    let mut b = NodeBuilder::new();
    b.ws(ws1);
    b.text(begin);
    b.text(name);

    if name.eq_ignore_ascii_case("SRC") {
        let (input, language) = opt((
            space1,
            take_while1(|c: char| c != ' ' && c != '\t' && c != '\n' && c != '\r'),
        ))
        .parse(input)?;
        let (input, switches) = opt((space1, source_block_switches)).parse(input)?;
        let (input, ws1) = space0(input)?;
        let (input, (parameters, ws2, nl)) = trim_line_end(input)?;

        if let Some((ws, language)) = language {
            b.ws(ws);
            b.token(SyntaxKind::SRC_BLOCK_LANGUAGE, language);
        }
        if let Some((ws, switches)) = switches {
            b.ws(ws);
            b.token(SyntaxKind::SRC_BLOCK_SWITCHES, switches);
        }
        b.ws(ws1);
        if !parameters.is_empty() {
            b.token(SyntaxKind::SRC_BLOCK_PARAMETERS, parameters);
        }
        b.ws(ws2);
        b.nl(nl);
        Ok((input, (b.finish(SyntaxKind::BLOCK_BEGIN), name.as_str())))
    } else if name.eq_ignore_ascii_case("EXAMPLE") {
        let (input, switches) = opt((space1, source_block_switches)).parse(input)?;
        let (input, ws1) = space0(input)?;
        let (input, (data, ws2, nl)) = trim_line_end(input)?;

        if let Some((ws, switches)) = switches {
            b.ws(ws);
            b.token(SyntaxKind::SRC_BLOCK_SWITCHES, switches);
        }
        b.ws(ws1);
        b.text(data);
        b.ws(ws2);
        b.nl(nl);
        Ok((input, (b.finish(SyntaxKind::BLOCK_BEGIN), name.as_str())))
    } else if name.eq_ignore_ascii_case("EXPORT") {
        let (input, ty) = opt((
            space1,
            take_while1(|c: char| c != ' ' && c != '\t' && c != '\n' && c != '\r'),
        ))
        .parse(input)?;
        let (input, data) = take_while(|c: char| c != '\n' && c != '\r').parse(input)?;
        let (input, nl) = eol_or_eof(input)?;

        if let Some((ws, ty)) = ty {
            b.ws(ws);
            b.token(SyntaxKind::EXPORT_BLOCK_TYPE, ty);
        }
        b.text(data);
        b.nl(nl);
        Ok((input, (b.finish(SyntaxKind::BLOCK_BEGIN), name.as_str())))
    } else {
        let (input, data) = take_while(|c: char| c != '\n' && c != '\r').parse(input)?;
        let (input, nl) = eol_or_eof(input)?;

        b.text(data);
        b.nl(nl);
        Ok((input, (b.finish(SyntaxKind::BLOCK_BEGIN), name.as_str())))
    }
}

fn source_block_switches(input: Input) -> IResult<Input, Input, ()> {
    let s = input.as_str();
    let mut cursor = 0;
    let mut parsed = false;

    loop {
        let token_start = if parsed {
            let spaces = leading_inline_space_len(&s[cursor..]);
            if spaces == 0 {
                break;
            }
            cursor + spaces
        } else {
            cursor
        };

        let Some(len) = source_block_switch_len(&s[token_start..]) else {
            break;
        };

        parsed = true;
        cursor = token_start + len;
    }

    if !parsed {
        Err(nom::Err::Error(()))
    } else {
        Ok(input.take_split(cursor))
    }
}

fn source_block_switch_len(s: &str) -> Option<usize> {
    let first = s.as_bytes().first()?;
    if !matches!(first, b'+' | b'-') {
        return None;
    }

    let name_len = s[1..].bytes().take_while(u8::is_ascii_alphabetic).count();
    if name_len == 0 {
        return None;
    }

    let token_len = 1 + name_len;
    let token = &s[..token_len];
    let rest = &s[token_len..];

    if matches!(token, "-n" | "+n") {
        let spaces = leading_inline_space_len(rest);
        let digits = rest[spaces..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if digits > 0 {
            return Some(token_len + spaces + digits);
        }
    }

    if token == "-l" {
        let spaces = leading_inline_space_len(rest);
        if spaces > 0 {
            if let Some(argument) = switch_argument_len(&rest[spaces..]) {
                return Some(token_len + spaces + argument);
            }
        }
    }

    Some(token_len)
}

fn switch_argument_len(s: &str) -> Option<usize> {
    if s.is_empty() || matches!(s.as_bytes()[0], b' ' | b'\t' | b'\n' | b'\r') {
        return None;
    }

    if s.as_bytes()[0] == b'"' {
        return s[1..].find('"').map(|index| index + 2);
    }

    Some(
        s.bytes()
            .take_while(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
            .count(),
    )
}

fn leading_inline_space_len(s: &str) -> usize {
    s.bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count()
}

fn block_end_node<'a>(input: Input<'a>, name: &str) -> IResult<Input<'a>, GreenElement, ()> {
    let (input, (ws, end, name, ws_, nl)) =
        (space0, tag_no_case("#+END_"), tag(name), space0, eol_or_eof).parse(input)?;

    let mut b = NodeBuilder::new();
    b.ws(ws);
    b.text(end);
    b.text(name);
    b.ws(ws_);
    b.nl(nl);

    Ok((input, b.finish(SyntaxKind::BLOCK_END)))
}

fn comma_quoted_text_nodes(input: Input) -> Vec<GreenElement> {
    let mut nodes = vec![];

    let s = input.as_str();

    let mut start = 0;
    for i in line_starts_iter(s) {
        // line must start with either ",*" or ",#+"
        if s.get(i..i + 2) != Some(",*") && s.get(i..i + 3) != Some(",#+") {
            continue;
        }

        let text = &s[start..i];
        if !text.is_empty() {
            nodes.push(token(SyntaxKind::TEXT, text));
        }

        nodes.push(token(SyntaxKind::COMMA, ","));
        start = i + 1;
    }

    if !s[start..].is_empty() {
        nodes.push(token(SyntaxKind::TEXT, &s[start..]));
    }

    nodes
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn block_node(input: Input) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(block_node_base, input)
}

#[test]
fn test_parse() {
    use crate::syntax_ast::{ExampleBlock, SourceBlock};
    use crate::tests::to_ast;

    let to_src_block = to_ast::<SourceBlock>(block_node);
    let to_example_block = to_ast::<ExampleBlock>(block_node);

    insta::assert_debug_snapshot!(
        to_example_block(
r#"#+BEGIN_EXAMPLE
,* headline
,#+block
text
    #+END_EXAMPLE"#
        ).syntax,
        @r###"
    EXAMPLE_BLOCK@0..59
      BLOCK_BEGIN@0..16
        TEXT@0..8 "#+BEGIN_"
        TEXT@8..15 "EXAMPLE"
        NEW_LINE@15..16 "\n"
      BLOCK_CONTENT@16..42
        COMMA@16..17 ","
        TEXT@17..28 "* headline\n"
        COMMA@28..29 ","
        TEXT@29..42 "#+block\ntext\n"
      BLOCK_END@42..59
        WHITESPACE@42..46 "    "
        TEXT@46..52 "#+END_"
        TEXT@52..59 "EXAMPLE"
    "###
    );

    insta::assert_debug_snapshot!(
        to_src_block(
r#"#+BEGIN_SRC


    #+END_SRC"#
        ).syntax,
        @r###"
    SOURCE_BLOCK@0..27
      BLOCK_BEGIN@0..12
        TEXT@0..8 "#+BEGIN_"
        TEXT@8..11 "SRC"
        NEW_LINE@11..12 "\n"
      BLANK_LINE@12..13 "\n"
      BLANK_LINE@13..14 "\n"
      BLOCK_CONTENT@14..14
      BLOCK_END@14..27
        WHITESPACE@14..18 "    "
        TEXT@18..24 "#+END_"
        TEXT@24..27 "SRC"
    "###
    );

    insta::assert_debug_snapshot!(
        to_src_block(
r#"#+begin_src
    #+end_src"#
        ).syntax,
        @r###"
    SOURCE_BLOCK@0..25
      BLOCK_BEGIN@0..12
        TEXT@0..8 "#+begin_"
        TEXT@8..11 "src"
        NEW_LINE@11..12 "\n"
      BLOCK_CONTENT@12..12
      BLOCK_END@12..25
        WHITESPACE@12..16 "    "
        TEXT@16..22 "#+end_"
        TEXT@22..25 "src"
    "###
    );

    insta::assert_debug_snapshot!(
        to_src_block(
r#"#+BEGIN_SRC javascript  -n 20 -r  :var n=0, l=2  :foo=bar
alert('Hello World!');
    #+END_SRC

    "#).syntax,
        @r###"
    SOURCE_BLOCK@0..100
      BLOCK_BEGIN@0..58
        TEXT@0..8 "#+BEGIN_"
        TEXT@8..11 "SRC"
        WHITESPACE@11..12 " "
        SRC_BLOCK_LANGUAGE@12..22 "javascript"
        WHITESPACE@22..24 "  "
        SRC_BLOCK_SWITCHES@24..32 "-n 20 -r"
        WHITESPACE@32..34 "  "
        SRC_BLOCK_PARAMETERS@34..57 ":var n=0, l=2  :foo=bar"
        NEW_LINE@57..58 "\n"
      BLOCK_CONTENT@58..81
        TEXT@58..81 "alert('Hello World!');\n"
      BLOCK_END@81..95
        WHITESPACE@81..85 "    "
        TEXT@85..91 "#+END_"
        TEXT@91..94 "SRC"
        NEW_LINE@94..95 "\n"
      BLANK_LINE@95..96 "\n"
      BLANK_LINE@96..100 "    "
    "###
    );

    insta::assert_debug_snapshot!(
        to_src_block(
r#"#+BEGIN_SRC rust +n 10 -r
fn main() {}
#+END_SRC"#
        ).syntax,
        @r###"
    SOURCE_BLOCK@0..48
      BLOCK_BEGIN@0..26
        TEXT@0..8 "#+BEGIN_"
        TEXT@8..11 "SRC"
        WHITESPACE@11..12 " "
        SRC_BLOCK_LANGUAGE@12..16 "rust"
        WHITESPACE@16..17 " "
        SRC_BLOCK_SWITCHES@17..25 "+n 10 -r"
        NEW_LINE@25..26 "\n"
      BLOCK_CONTENT@26..39
        TEXT@26..39 "fn main() {}\n"
      BLOCK_END@39..48
        TEXT@39..45 "#+END_"
        TEXT@45..48 "SRC"
    "###
    );

    insta::assert_debug_snapshot!(
        to_example_block(
r#"#+BEGIN_EXAMPLE -n 3
,* headline
#+END_EXAMPLE"#
        ).syntax,
        @r###"
    EXAMPLE_BLOCK@0..46
      BLOCK_BEGIN@0..21
        TEXT@0..8 "#+BEGIN_"
        TEXT@8..15 "EXAMPLE"
        WHITESPACE@15..16 " "
        SRC_BLOCK_SWITCHES@16..20 "-n 3"
        NEW_LINE@20..21 "\n"
      BLOCK_CONTENT@21..33
        COMMA@21..22 ","
        TEXT@22..33 "* headline\n"
      BLOCK_END@33..46
        TEXT@33..39 "#+END_"
        TEXT@39..46 "EXAMPLE"
    "###
    );

    // TODO: more testing
}
