from orgizepy.functions import headline_functions
from orgizepy.parser import parse_org


def test_raw_org_parse_and_functions():
    document = parse_org("#+TODO: NEXT DONE\n* NEXT Ship SDK\n")
    headlines = [element for element in document.elements if element.kind == "headline"]
    assert len(headlines) == 1
    assert ("title", "NEXT Ship SDK") in headlines[0].fields
    functions = headline_functions(document, headlines[0].id)
    assert functions.todo_keyword == "NEXT"
    assert functions.content_after_todo == "Ship SDK"


def test_empty_and_unicode_org():
    assert parse_org("").elements[0].kind == "org-data"
    document = parse_org("* 设计决定\n")
    assert any(("title", "设计决定") in element.fields for element in document.elements)
