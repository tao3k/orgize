"""Thin Python projection of the shared Rust/Scheme Contract owner."""

from dataclasses import dataclass
from typing import Literal, Sequence

from ._orgize import _evaluate_contract


class OrgizeContractError(RuntimeError):
    """The shared native Contract runtime rejected an operation."""


@dataclass(frozen=True)
class ContractRow:
    id: int
    parent_id: int
    kind: str
    field_name: str = ""
    field_value: str = ""


@dataclass(frozen=True)
class ContractResult:
    matched_count: int
    passed: bool


def evaluate_contract(
    rows: Sequence[ContractRow],
    *,
    scope_id: int,
    kind: str,
    field_name: str = "",
    field_value: str = "",
    expectation: Literal["exactly", "at_least", "at_most"] = "at_least",
    expected_count: int = 1,
) -> ContractResult:
    """Evaluate explicit admitted rows; parsed Elements are not flattened."""
    try:
        matched, passed = _evaluate_contract(
            [(r.id, r.parent_id, r.kind, r.field_name, r.field_value) for r in rows],
            scope_id, kind, field_name, field_value, expectation, expected_count,
        )
    except RuntimeError as error:
        raise OrgizeContractError(str(error)) from error
    return ContractResult(matched, passed)
