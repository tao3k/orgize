use memchr::memrchr_iter;
use nom::{
    IResult, Parser, bytes::complete::take_while1, character::complete::space0, combinator::opt,
};

use super::{
    SyntaxKind,
    combinator::{GreenElement, NodeBuilder, line_starts_iter, node, token, trim_line_end},
    drawer::property_drawer_node,
    input::Input,
    object::standard_object_nodes,
    parser_contract::ElementNodesParser,
    planning::planning_node,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn headline_node(
    input: Input,
    element_nodes: ElementNodesParser,
) -> IResult<Input, GreenElement, ()> {
    debug_assert!(!input.is_empty());
    crate::lossless_parser!(|input| headline_node_base(input, element_nodes), input)
}

fn headline_node_base(
    input: Input,
    element_nodes: ElementNodesParser,
) -> IResult<Input, GreenElement, ()> {
    let (input, stars) = headline_stars(input)?;

    let mut b = NodeBuilder::new();

    b.token(SyntaxKind::HEADLINE_STARS, stars);

    let (input, ws) = space0(input)?;
    b.ws(ws);

    let (input, headline_keyword) = opt(headline_keyword_token).parse(input)?;

    if let Some((headline_keyword, ws)) = headline_keyword {
        b.push(headline_keyword);
        b.ws(ws);
    }

    let (input, headline_priority) = opt(headline_priority_node).parse(input)?;

    if let Some((headline_priority, ws)) = headline_priority {
        b.push(headline_priority);
        b.ws(ws);
    }

    let (input, (title_and_tags, ws_, nl)) = trim_line_end(input)?;
    let (title, tags) = opt(headline_tags_node).parse(title_and_tags)?;

    if !title.is_empty() {
        b.push(node(
            SyntaxKind::HEADLINE_TITLE,
            standard_object_nodes(title),
        ));
    }
    b.push_opt(tags);
    b.ws(ws_);
    b.nl(nl);

    if input.is_empty() {
        return Ok((input, b.finish(SyntaxKind::HEADLINE)));
    }

    let (input, planning) = opt(planning_node).parse(input)?;
    b.push_opt(planning);

    if input.is_empty() {
        return Ok((input, b.finish(SyntaxKind::HEADLINE)));
    }

    let (input, property_drawer) = opt(property_drawer_node).parse(input)?;
    b.push_opt(property_drawer);

    if input.is_empty() {
        return Ok((input, b.finish(SyntaxKind::HEADLINE)));
    }

    let (input, section) = opt(|input| section_node(input, element_nodes)).parse(input)?;
    b.push_opt(section);

    let mut i = input;
    let current_level = stars.len();
    while !i.is_empty() {
        let next_level = i.bytes().take_while(|&c| c == b'*').count();

        if next_level <= current_level {
            break;
        }

        let (input, headline) = headline_node(i, element_nodes)?;
        b.push(headline);
        debug_assert!(i.len() > input.len(), "{} > {}", i.len(), input.len());
        i = input;
    }

    Ok((i, b.finish(SyntaxKind::HEADLINE)))
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn section_node(
    input: Input,
    element_nodes: ElementNodesParser,
) -> IResult<Input, GreenElement, ()> {
    debug_assert!(!input.is_empty());
    let (input, section) = section_text(input)?;
    Ok((input, node(SyntaxKind::SECTION, element_nodes(section)?)))
}

fn section_text(input: Input) -> IResult<Input, Input, ()> {
    for (input, section) in line_starts_iter(input.as_str()).map(|i| input.take_split(i)) {
        if headline_stars(input).is_ok() && !is_inlinetask_start(input) {
            if section.is_empty() {
                return Err(nom::Err::Error(()));
            }

            return Ok((input, section));
        }
    }

    Ok(input.take_split(input.len()))
}

fn is_inlinetask_start(input: Input) -> bool {
    headline_stars(input)
        .map(|(_, stars)| stars.len() >= input.c.effective_inlinetask_min_level())
        .unwrap_or(false)
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(super) fn headline_stars(input: Input) -> IResult<Input, Input, ()> {
    let bytes = input.as_bytes();
    let level = bytes.iter().take_while(|&&c| c == b'*').count();

    if level == 0 {
        Err(nom::Err::Error(()))
    }
    // headline stars must be followed by space
    else if matches!(bytes.get(level), Some(b' ')) {
        Ok(input.take_split(level))
    } else {
        Err(nom::Err::Error(()))
    }
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(super) fn headline_tags_node(input: Input) -> IResult<Input, GreenElement, ()> {
    if !input.s.ends_with(':') {
        return Err(nom::Err::Error(()));
    };

    let bytes = input.as_bytes();

    // we're going to skip to first colon, so we start from the
    // second last character
    let mut i = input.len() - 1;
    let mut can_not_be_ws = true;
    let mut children = vec![token(SyntaxKind::COLON, ":")];

    for ii in memrchr_iter(b':', bytes).skip(1) {
        let item = &bytes[ii + 1..i];

        if item.is_empty() {
            children.push(token(SyntaxKind::COLON, ":"));
            can_not_be_ws = false;
            debug_assert!(i > ii, "{} > {}", i, ii);
            i = ii;
        } else if String::from_utf8_lossy(item)
            .chars()
            // https://github.com/yyr/org-mode/blob/d8494b5668ad4d4e68e83228ae8451eaa01d2220/lisp/org-element.el#L922C25-L922C32
            .all(|c| c.is_alphanumeric() || c == '_' || c == '@' || c == '#' || c == '%')
        {
            children.push(input.slice(ii + 1..i).text_token());
            children.push(token(SyntaxKind::COLON, ":"));
            can_not_be_ws = false;
            debug_assert!(i > ii, "{} > {}", i, ii);
            i = ii;
        } else if item.iter().all(|&c| c == b' ' || c == b'\t') && !can_not_be_ws {
            children.push(input.slice(ii + 1..i).ws_token());
            children.push(token(SyntaxKind::COLON, ":"));
            can_not_be_ws = true;
            debug_assert!(i > ii, "{} > {}", i, ii);
            i = ii;
        } else {
            break;
        }
    }

    if children.len() <= 2 {
        return Err(nom::Err::Error(()));
    }

    if i != 0 && bytes[i - 1] != b' ' && bytes[i - 1] != b'\t' {
        return Err(nom::Err::Error(()));
    }

    // we parse headline tag from right to left,
    // so we need to reverse the result after it finishes
    children.reverse();

    Ok((input.slice(0..i), node(SyntaxKind::HEADLINE_TAGS, children)))
}

pub(super) fn headline_keyword_token(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    let (input, word) = take_while1(|c: char| !c.is_ascii_whitespace()).parse(input)?;
    let (input, ws) = space0(input)?;
    if input.c.todo_keywords.0.iter().any(|k| k == word.s) {
        Ok((input, (word.token(SyntaxKind::HEADLINE_KEYWORD_TODO), ws)))
    } else if input.c.todo_keywords.1.iter().any(|k| k == word.s) {
        Ok((input, (word.token(SyntaxKind::HEADLINE_KEYWORD_DONE), ws)))
    } else {
        Err(nom::Err::Error(()))
    }
}

pub(super) fn headline_priority_node(input: Input) -> IResult<Input, (GreenElement, Input), ()> {
    if !input.starts_with("[#") {
        return Err(nom::Err::Error(()));
    }
    let Some(value_len) = priority_value_len(&input[2..]) else {
        return Err(nom::Err::Error(()));
    };
    let close = 2 + value_len;
    if input.as_bytes().get(close) != Some(&b']') {
        return Err(nom::Err::Error(()));
    }

    let node = node(
        SyntaxKind::HEADLINE_PRIORITY,
        [
            input.slice(..1).token(SyntaxKind::L_BRACKET),
            input.slice(1..2).token(SyntaxKind::HASH),
            input.slice(2..close).text_token(),
            input.slice(close..close + 1).token(SyntaxKind::R_BRACKET),
        ],
    );
    let input = input.take_from(close + 1);

    let (input, ws) = space0(input)?;

    Ok((input, (node, ws)))
}

fn priority_value_len(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    match bytes.first().copied()? {
        b'A'..=b'Z' => Some(1),
        digit @ b'0'..=b'9' => match bytes.get(1).copied() {
            Some(next) if next.is_ascii_digit() => match (digit, next) {
                (b'1'..=b'5', b'0'..=b'9') | (b'6', b'0'..=b'4') => Some(2),
                _ => None,
            },
            _ => Some(1),
        },
        _ => None,
    }
}

#[test]
fn parse() {
    use crate::{ParseConfig, syntax_ast::Headline, tests::to_ast};

    let to_headline =
        to_ast::<Headline>(|input| headline_node(input, crate::syntax::element::element_nodes));

    insta::assert_debug_snapshot!(
        to_headline("* foo").syntax,
        @r###"
    HEADLINE@0..5
      HEADLINE_STARS@0..1 "*"
      WHITESPACE@1..2 " "
      HEADLINE_TITLE@2..5
        TEXT@2..5 "foo"
    "###
    );

    insta::assert_debug_snapshot!(
        to_headline("* foo\n\n** bar").syntax,
        @r###"
    HEADLINE@0..13
      HEADLINE_STARS@0..1 "*"
      WHITESPACE@1..2 " "
      HEADLINE_TITLE@2..5
        TEXT@2..5 "foo"
      NEW_LINE@5..6 "\n"
      SECTION@6..7
        PARAGRAPH@6..7
          BLANK_LINE@6..7 "\n"
      HEADLINE@7..13
        HEADLINE_STARS@7..9 "**"
        WHITESPACE@9..10 " "
        HEADLINE_TITLE@10..13
          TEXT@10..13 "bar"
    "###
    );

    insta::assert_debug_snapshot!(
        to_headline("* TODO foo\nbar\n** baz\n").syntax,
        @r###"
    HEADLINE@0..22
      HEADLINE_STARS@0..1 "*"
      WHITESPACE@1..2 " "
      HEADLINE_KEYWORD_TODO@2..6 "TODO"
      WHITESPACE@6..7 " "
      HEADLINE_TITLE@7..10
        TEXT@7..10 "foo"
      NEW_LINE@10..11 "\n"
      SECTION@11..15
        PARAGRAPH@11..15
          TEXT@11..15 "bar\n"
      HEADLINE@15..22
        HEADLINE_STARS@15..17 "**"
        WHITESPACE@17..18 " "
        HEADLINE_TITLE@18..21
          TEXT@18..21 "baz"
        NEW_LINE@21..22 "\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_headline("** [#A] foo\n* baz").syntax,
        @r###"
    HEADLINE@0..12
      HEADLINE_STARS@0..2 "**"
      WHITESPACE@2..3 " "
      HEADLINE_PRIORITY@3..7
        L_BRACKET@3..4 "["
        HASH@4..5 "#"
        TEXT@5..6 "A"
        R_BRACKET@6..7 "]"
      WHITESPACE@7..8 " "
      HEADLINE_TITLE@8..11
        TEXT@8..11 "foo"
      NEW_LINE@11..12 "\n"
    "###
    );

    insta::assert_debug_snapshot!(
        to_headline("** [#64] foo\n* baz").syntax,
        @r###"
    HEADLINE@0..13
      HEADLINE_STARS@0..2 "**"
      WHITESPACE@2..3 " "
      HEADLINE_PRIORITY@3..8
        L_BRACKET@3..4 "["
        HASH@4..5 "#"
        TEXT@5..7 "64"
        R_BRACKET@7..8 "]"
      WHITESPACE@8..9 " "
      HEADLINE_TITLE@9..12
        TEXT@9..12 "foo"
      NEW_LINE@12..13 "\n"
    "###
    );

    let config = &ParseConfig::default();

    assert!(headline_node(("_ ", config).into(), crate::syntax::element::element_nodes).is_err());
    assert!(headline_node(("*", config).into(), crate::syntax::element::element_nodes).is_err());
    assert!(
        headline_node(
            (" * ", config).into(),
            crate::syntax::element::element_nodes
        )
        .is_err()
    );
    assert!(headline_node(("**", config).into(), crate::syntax::element::element_nodes).is_err());
    assert!(
        headline_node(
            ("**\n", config).into(),
            crate::syntax::element::element_nodes
        )
        .is_err()
    );
    assert!(
        headline_node(
            ("**\r", config).into(),
            crate::syntax::element::element_nodes
        )
        .is_err()
    );
    assert!(
        headline_node(
            ("**\t", config).into(),
            crate::syntax::element::element_nodes
        )
        .is_err()
    );
}

#[test]
fn issue_15_16() {
    use crate::{syntax_ast::Headline, tests::to_ast};

    let to_headline =
        to_ast::<Headline>(|input| headline_node(input, crate::syntax::element::element_nodes));

    assert!(to_headline("* a ::").tags().count() == 0);
    assert!(to_headline("* a : :").tags().count() == 0);
    assert!(to_headline("* a :(:").tags().count() == 0);
    assert!(to_headline("* a :a: :").tags().count() == 0);
    assert!(to_headline("* a :a :").tags().count() == 0);
    assert!(to_headline("* a a:").tags().count() == 0);
    assert!(to_headline("* a :a").tags().count() == 0);

    let tags = to_headline("* a \t:_:").tags();
    assert_eq!(
        vec!["_".to_string()],
        tags.map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a \t :@:").tags();
    assert_eq!(
        vec!["@".to_string()],
        tags.map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a :#:").tags();
    assert_eq!(
        vec!["#".to_string()],
        tags.map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a\t :%:").tags();
    assert_eq!(
        vec!["%".to_string()],
        tags.map(|x| x.to_string()).collect::<Vec<_>>(),
    );

    let tags = to_headline("* a :余: :破:").tags();
    assert_eq!(
        vec!["余".to_string(), "破".to_string()],
        tags.map(|x| x.to_string()).collect::<Vec<_>>(),
    );
}
