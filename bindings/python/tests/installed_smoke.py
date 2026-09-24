"""Run from outside the source tree against an installed wheel."""

from orgizepy.contract import ContractRow, evaluate_contract
from orgizepy.functions import headline_functions
from orgizepy.parser import parse_org


document = parse_org("* Evidence\n")
headline = next(element for element in document.elements if element.kind == "headline")
assert headline_functions(document, headline.id).content_after_todo == "Evidence"
result = evaluate_contract(
    [ContractRow(0, -1, "org-data"), ContractRow(1, 0, "headline", "title", "Evidence")],
    scope_id=0,
    kind="headline",
    field_name="title",
    field_value="Evidence",
    expectation="exactly",
    expected_count=1,
)
assert result.passed and result.matched_count == 1
print("orgizepy-wheel-ok")
