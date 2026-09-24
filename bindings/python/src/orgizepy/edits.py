"""Source-bound candidate edits; persistence and authorization belong to callers."""

from dataclasses import dataclass

from ._orgize import apply_source_edits as _apply_source_edits
from ._orgize import source_digest


@dataclass(frozen=True)
class SourceEdit:
    node_id: str
    start_byte: int
    end_byte: int
    expected_old: str
    replacement: str


def apply_source_edits(
    source: str, projected_source_digest: str, edits: list[SourceEdit]
) -> str:
    """Return a reparsed candidate or raise ValueError without changing a file."""
    return _apply_source_edits(
        source,
        projected_source_digest,
        [
            (
                edit.node_id,
                edit.start_byte,
                edit.end_byte,
                edit.expected_old,
                edit.replacement,
            )
            for edit in edits
        ],
    )


__all__ = ["SourceEdit", "apply_source_edits", "source_digest"]
