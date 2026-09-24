"""Typed surface of the maturin/PyO3 extension."""

class Element:
    id: int
    parent_id: int | None
    child_ids: list[int]
    kind: str
    category: str
    start: int
    end: int
    fields: list[tuple[str, str]]

class ParsedOrg:
    @property
    def elements(self) -> list[Element]: ...
    def headline_todo_type(self, record_id: int) -> str | None: ...
    def headline_todo_keyword(self, record_id: int) -> str | None: ...
    def headline_content_after_todo(self, record_id: int) -> str | None: ...

def parse_org(source: str) -> ParsedOrg: ...
