"""Run from outside the source tree against an installed wheel."""

from importlib.metadata import distribution
import json

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
package = distribution("orgizepy")
package_files = {str(path) for path in package.files or ()}
for notice in (
    "LICENSE.orgizepy",
    "LICENSES/Apache-2.0.txt",
    "LICENSES/LGPL-2.1-or-later.txt",
    "LICENSES/Zlib.txt",
    "THIRD_PARTY_NOTICES.md",
):
    assert any(path.endswith(f"/licenses/{notice}") for path in package_files), notice
sbom_path = next(
    path for path in package.files or ()
    if str(path).endswith("/sboms/orgizepy.cyclonedx.json")
)
sbom = json.loads(package.locate_file(sbom_path).read_text(encoding="utf-8"))
assert all(component.get("licenses") for component in sbom["components"])
print("orgizepy-wheel-ok")
