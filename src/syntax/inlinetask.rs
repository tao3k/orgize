use nom::{IResult, Parser, character::complete::space0, combinator::opt};

use super::{
    SyntaxKind,
    combinator::{GreenElement, NodeBuilder, line_starts_iter, node, trim_line_end},
    drawer::property_drawer_node,
    headline::{
        headline_keyword_token, headline_priority_node, headline_stars, headline_tags_node,
    },
    input::Input,
    object::standard_object_nodes,
    parser_contract::ElementNodesParser,
    planning::planning_node,
};

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip(input), fields(input = input.s))
)]
pub(crate) fn inlinetask_node(
    input: Input,
    element_nodes: ElementNodesParser,
) -> IResult<Input, GreenElement, ()> {
    crate::lossless_parser!(|input| inlinetask_node_base(input, element_nodes), input)
}

fn inlinetask_node_base(
    input: Input,
    element_nodes: ElementNodesParser,
) -> IResult<Input, GreenElement, ()> {
    let (input, stars) = headline_stars(input)?;
    if stars.len() < input.c.effective_inlinetask_min_level() {
        return Err(nom::Err::Error(()));
    }

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
        return Ok((input, b.finish(SyntaxKind::INLINETASK)));
    }

    let (input, planning) = opt(planning_node).parse(input)?;
    b.push_opt(planning);

    if input.is_empty() {
        return Ok((input, b.finish(SyntaxKind::INLINETASK)));
    }

    let (input, property_drawer) = opt(property_drawer_node).parse(input)?;
    b.push_opt(property_drawer);

    let Some(end_offset) = inlinetask_end_offset(input) else {
        return Ok((input, b.finish(SyntaxKind::INLINETASK)));
    };

    let (end_input, body) = input.take_split(end_offset);
    if !body.is_empty() {
        b.push(node(SyntaxKind::SECTION, element_nodes(body)?));
    }

    let (input, end) = inlinetask_end_node(end_input)?;
    b.push(end);

    Ok((input, b.finish(SyntaxKind::INLINETASK)))
}

fn inlinetask_end_offset(input: Input) -> Option<usize> {
    line_starts_iter(input.as_str()).find(|&offset| is_inlinetask_end(input.slice(offset..)))
}

fn is_inlinetask_end(input: Input) -> bool {
    inlinetask_end_node(input).is_ok()
}

fn inlinetask_end_node(input: Input) -> IResult<Input, GreenElement, ()> {
    let (input, stars) = headline_stars(input)?;
    if stars.len() < input.c.effective_inlinetask_min_level() {
        return Err(nom::Err::Error(()));
    }

    let mut b = NodeBuilder::new();
    b.token(SyntaxKind::HEADLINE_STARS, stars);

    let (input, ws) = space0(input)?;
    b.ws(ws);

    let (input, (title, ws_, nl)) = trim_line_end(input)?;
    if title.as_str() != "END" {
        return Err(nom::Err::Error(()));
    }

    b.push(node(SyntaxKind::HEADLINE_TITLE, [title.text_token()]));
    b.ws(ws_);
    b.nl(nl);

    Ok((input, b.finish(SyntaxKind::INLINETASK_END)))
}
