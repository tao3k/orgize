"""Scheme-owned Org Contract ABI, separate from parsing and functions."""

from dataclasses import dataclass
import atexit
from importlib.resources import files
from itertools import chain
import os
from pathlib import Path
import sys
from threading import RLock, current_thread, main_thread
from typing import Literal, Sequence

from cffi import FFI


class OrgizeContractError(RuntimeError):
    """The standalone Scheme Contract runtime rejected an operation."""


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


_ffi = FFI()
_ffi.cdef("""
typedef struct {
  int64_t id;
  int64_t parent_id;
  const char *kind;
  const char *field_name;
  const char *field_value;
} orgize_element_row;
typedef struct {
  int32_t status;
  uint32_t matched_count;
  int32_t passed;
} orgize_contract_result;
uint32_t orgize_abi_revision(void);
int32_t orgize_runtime_init(void);
void orgize_runtime_shutdown(void);
int32_t orgize_contract_evaluate(const orgize_element_row *, uint32_t,
    int64_t, const char *, const char *, const char *, uint32_t, uint32_t,
    orgize_contract_result *);
""")
_lock = RLock()
_library = None
_started = False


def _load_library():
    global _library
    if _library is None:
        suffix = "dylib" if sys.platform == "darwin" else "so"
        configured = os.environ.get("ORGIZE_CONTRACT_LIBRARY")
        path = Path(configured) if configured else Path(str(files("orgizepy").joinpath("lib", f"liborgize.{suffix}")))
        if not path.is_file():
            raise OrgizeContractError(f"Orgize Contract library not found: {path}")
        _library = _ffi.dlopen(str(path), _ffi.RTLD_NOW | _ffi.RTLD_LOCAL)
    return _library


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
    """Evaluate one Contract over explicit, admitted Element rows.

    A parsed document is not silently coerced into Contract rows: each ABI row
    admits at most one projected field, while a parsed Element can have many.
    """
    modes = {"exactly": 0, "at_least": 1, "at_most": 2}
    if expectation not in modes or expected_count < 0 or expected_count > 2**32 - 1:
        raise ValueError("invalid Contract expectation")
    if len(rows) > 2**32 - 1:
        raise ValueError("too many Contract rows")
    if current_thread() is not main_thread():
        raise OrgizeContractError("Scheme Contract calls require the main thread")
    strings = chain(
        (kind, field_name, field_value),
        chain.from_iterable((row.kind, row.field_name, row.field_value) for row in rows),
    )
    for value in strings:
        if "\x00" in value:
            raise ValueError("Contract strings cannot contain NUL bytes")
    global _started
    with _lock:
        library = _load_library()
        if not _started:
            if library.orgize_runtime_init() != 0:
                raise OrgizeContractError("Orgize Contract runtime initialization failed")
            if library.orgize_abi_revision() != 1:
                library.orgize_runtime_shutdown()
                raise OrgizeContractError("Unsupported Orgize Contract ABI revision")
            _started = True
            atexit.register(library.orgize_runtime_shutdown)
        allocated = []

        def encoded(value: str):
            pointer = _ffi.new("char[]", value.encode("utf-8"))
            allocated.append(pointer)
            return pointer

        native_rows = _ffi.new("orgize_element_row[]", len(rows))
        for index, row in enumerate(rows):
            native_rows[index].id = row.id
            native_rows[index].parent_id = row.parent_id
            native_rows[index].kind = encoded(row.kind)
            native_rows[index].field_name = encoded(row.field_name)
            native_rows[index].field_value = encoded(row.field_value)
        result = _ffi.new("orgize_contract_result *")
        status = library.orgize_contract_evaluate(
            native_rows, len(rows), scope_id,
            encoded(kind), encoded(field_name), encoded(field_value),
            modes[expectation], expected_count, result,
        )
        if status != 0 or result.status != 0:
            raise OrgizeContractError("Orgize Contract evaluation failed")
        return ContractResult(result.matched_count, bool(result.passed))
