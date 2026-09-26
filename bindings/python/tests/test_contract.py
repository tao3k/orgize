from concurrent.futures import ThreadPoolExecutor

import pytest

from orgizepy.contract import ContractRow, OrgizeContractError, evaluate_contract


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


def test_scheme_contract_rejects_worker_thread():
    with ThreadPoolExecutor(max_workers=1) as pool:
        future = pool.submit(evaluate_contract, [], scope_id=0, kind="headline")
        with pytest.raises(OrgizeContractError, match="main thread"):
            future.result()
