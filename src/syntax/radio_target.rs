use nom::{
    IResult,
    bytes::complete::take_while,
    combinator::{map, verify},
};

use super::{
    SyntaxKind,
    combinator::{GreenElement, l_angle3_token, node, r_angle3_token},
    input::Input,
    parser_contract::ObjectNodesParser,
};

pub(crate) fn radio_target_node(
    input: Input,
    minimal_object_nodes: ObjectNodesParser,
) -> IResult<Input, GreenElement, ()> {
    let mut parser = map(
        (
            l_angle3_token,
            verify(
                take_while(|c: char| c != '<' && c != '\n' && c != '>'),
                |s: &Input| {
                    s.as_str().starts_with(|c| c != ' ') && s.as_str().ends_with(|c| c != ' ')
                },
            ),
            r_angle3_token,
        ),
        |(l_angle3, contents, r_angle3)| {
            let mut children = vec![l_angle3];
            children.extend(minimal_object_nodes(contents));
            children.push(r_angle3);
            node(SyntaxKind::RADIO_TARGET, children)
        },
    );
    crate::lossless_parser!(parser, input)
}

#[test]
fn parse() {
    use crate::{ParseConfig, syntax_ast::RadioTarget, tests::to_ast};

    let to_radio_target = to_ast::<RadioTarget>(|input| {
        radio_target_node(input, crate::syntax::object::minimal_object_nodes)
    });

    insta::assert_debug_snapshot!(
        to_radio_target("<<<target>>>").syntax,
        @r###"
    RADIO_TARGET@0..12
      L_ANGLE3@0..3 "<<<"
      TEXT@3..9 "target"
      R_ANGLE3@9..12 ">>>"
    "###
    );

    insta::assert_debug_snapshot!(
        to_radio_target("<<<tar get>>>").syntax,
        @r###"
    RADIO_TARGET@0..13
      L_ANGLE3@0..3 "<<<"
      TEXT@3..10 "tar get"
      R_ANGLE3@10..13 ">>>"
    "###
    );

    insta::assert_debug_snapshot!(
        to_radio_target("<<<\\alpha>>>").syntax,
        @r###"
    RADIO_TARGET@0..12
      L_ANGLE3@0..3 "<<<"
      ENTITY@3..9
        BACKSLASH@3..4 "\\"
        TEXT@4..9 "alpha"
      R_ANGLE3@9..12 ">>>"
    "###
    );

    let config = &ParseConfig::default();

    assert!(
        radio_target_node(
            ("<<<target >>>", config).into(),
            crate::syntax::object::minimal_object_nodes
        )
        .is_err()
    );
    assert!(
        radio_target_node(
            ("<<< target>>>", config).into(),
            crate::syntax::object::minimal_object_nodes
        )
        .is_err()
    );
    assert!(
        radio_target_node(
            ("<<<ta<get>>>", config).into(),
            crate::syntax::object::minimal_object_nodes
        )
        .is_err()
    );
    assert!(
        radio_target_node(
            ("<<<ta>get>>>", config).into(),
            crate::syntax::object::minimal_object_nodes
        )
        .is_err()
    );
    assert!(
        radio_target_node(
            ("<<<ta\nget>>>", config).into(),
            crate::syntax::object::minimal_object_nodes
        )
        .is_err()
    );
    assert!(
        radio_target_node(
            ("<<<target>>", config).into(),
            crate::syntax::object::minimal_object_nodes
        )
        .is_err()
    );
}
