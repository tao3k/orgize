from concurrent.futures import ThreadPoolExecutor

import pytest

from orgizepy.contract import ContractRow, evaluate_contract
from orgizepy.parser import parse_org
from orgizepy import _orgize


def test_scheme_contract_repeated_calls():
    rows = [
        ContractRow(0, -1, "org-data"),
        ContractRow(1, 0, "headline", "title", "Evidence"),
    ]
    accepted = evaluate_contract(
        rows, scope_id=0, kind="headline", field_name="title",
        field_value="Evidence", expectation="exactly", expected_count=1,
    )
    assert accepted.matched_count == 1 and accepted.passed
    rejected = evaluate_contract(
        rows, scope_id=0, kind="headline", field_name="title",
        field_value="Missing", expectation="exactly", expected_count=1,
    )
    assert rejected.matched_count == 0 and not rejected.passed


def test_parser_and_contract_share_owner_from_worker_threads():
    def mixed_call(index):
        document = parse_org(f"* Evidence {index}\n")
        assert any(element.kind == "headline" for element in document.elements)
        result = evaluate_contract(
            [ContractRow(0, -1, "org-data"), ContractRow(1, 0, "headline")],
            scope_id=0, kind="headline", expectation="exactly", expected_count=1,
        )
        assert result.passed and result.matched_count == 1
        assert parse_org("").elements[0].kind == "org-data"

    with ThreadPoolExecutor(max_workers=4) as pool:
        list(pool.map(mixed_call, range(12)))


def test_contract_rejects_nul():
    with pytest.raises(ValueError, match="NUL"):
        evaluate_contract([], scope_id=0, kind="headline\x00")


def test_rust_c_abi_rejects_null_rows_and_initializes_failure_result():
    # Standalone C-consumer regression only; Python's public API uses PyO3.
    import ctypes as c

    class Result(c.Structure):
        _fields_ = [("status", c.c_int32), ("matched_count", c.c_uint32),
                    ("passed", c.c_int32)]

    library = c.CDLL(_orgize.__file__)
    evaluate = library.orgize_shared_contract_evaluate
    evaluate.argtypes = [c.c_void_p, c.c_uint32, c.c_int64,
                        c.c_char_p, c.c_char_p, c.c_char_p,
                        c.c_uint32, c.c_uint32, c.POINTER(Result)]
    evaluate.restype = c.c_int32
    assert library.orgize_shared_abi_revision() == 2
    # The session fixture already performed exclusive Python startup.
    assert library.orgize_shared_runtime_initialize() == 0
    output = Result()
    output.status = 42
    output.matched_count = 99
    assert evaluate(
        None, 1, 0, b"headline", b"", b"", 0, 1, c.byref(output),
    ) == -1
    assert output.status == -1 and output.matched_count == 0 and output.passed == 0
    assert evaluate(
        None, 0, 0, b"headline", b"", b"", 0, 1, None,
    ) == -1


@pytest.mark.parametrize("expectation,count", [
    ("unknown", 1), ("exactly", -1), ("at_least", 2**32), ("exactly", 2**100),
])
def test_contract_rejects_invalid_expectations(expectation, count):
    with pytest.raises(ValueError, match="expectation"):
        evaluate_contract([], scope_id=0, kind="headline",
                          expectation=expectation, expected_count=count)


def test_contract_rejects_nul_in_rows():
    with pytest.raises(ValueError, match="NUL"):
        evaluate_contract([ContractRow(0, -1, "org-data", "x", "\x00")],
                          scope_id=0, kind="headline")


@pytest.mark.parametrize("expectation,count,passed", [
    ("exactly", 1, True), ("at_least", 2, False), ("at_most", 0, False),
    ("at_most", 1, True),
])
def test_contract_count_modes_use_shared_native_owner(expectation, count, passed):
    result = evaluate_contract(
        [ContractRow(0, -1, "org-data"), ContractRow(1, 0, "headline")],
        scope_id=0, kind="headline", expectation=expectation, expected_count=count,
    )
    assert result.matched_count == 1 and result.passed is passed
