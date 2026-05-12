use nom::IResult;

use super::{combinator::GreenElement, input::Input};

pub(crate) type ElementNodeParser = for<'a> fn(Input<'a>) -> IResult<Input<'a>, GreenElement, ()>;
pub(crate) type ElementNodesParser =
    for<'a> fn(Input<'a>) -> Result<Vec<GreenElement>, nom::Err<()>>;
pub(crate) type ObjectNodesParser = for<'a> fn(Input<'a>) -> Vec<GreenElement>;
