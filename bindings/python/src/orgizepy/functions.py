"""Scheme-AOT headline functions exposed independently of parsing."""

from dataclasses import dataclass

from .parser import ParsedOrg


@dataclass(frozen=True)
class HeadlineFunctions:
    todo_type: str | None
    todo_keyword: str | None
    content_after_todo: str | None


def headline_functions(document: ParsedOrg, element_id: int) -> HeadlineFunctions:
    """Read the generated Org headline functions for one parsed Element."""
    return HeadlineFunctions(
        document.headline_todo_type(element_id),
        document.headline_todo_keyword(element_id),
        document.headline_content_after_todo(element_id),
    )
