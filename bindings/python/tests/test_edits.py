import pytest

from orgizepy.edits import SourceEdit, apply_source_edits, source_digest


SOURCE = (
    "* Design\n:PROPERTIES:\n:ID: design-1\n:END:\nOld guarantee\n"
    "* Notes\n:PROPERTIES:\n:ID: notes-1\n:END:\nKeep me\n"
)


def edit(source: str, node_id: str, old: str, replacement: str) -> SourceEdit:
    start = source.encode().index(old.encode())
    return SourceEdit(node_id, start, start + len(old.encode()), old, replacement)


def test_source_bound_candidate_preserves_other_nodes():
    candidate = apply_source_edits(
        SOURCE,
        source_digest(SOURCE),
        [edit(SOURCE, "design-1", "Old guarantee", "New guarantee")],
    )
    assert candidate == SOURCE.replace("Old guarantee", "New guarantee")
    assert ":ID: notes-1\n:END:\nKeep me\n" in candidate
    assert SOURCE.endswith("Keep me\n")


@pytest.mark.parametrize(
    ("digest", "node_id", "old", "expected"),
    [
        ("sha256:stale", "design-1", "Old guarantee", "StaleSource"),
        (None, "notes-1", "Old guarantee", "WrongNode"),
        (None, "design-1", "Keep me", "WrongNode"),
    ],
)
def test_rejects_stale_or_wrong_owner(digest, node_id, old, expected):
    with pytest.raises(ValueError, match=expected):
        apply_source_edits(
            SOURCE,
            digest or source_digest(SOURCE),
            [edit(SOURCE, node_id, old, "replacement")],
        )


def test_rejects_changed_content_and_utf8_split():
    original = edit(SOURCE, "design-1", "Old guarantee", "New guarantee")
    with pytest.raises(ValueError, match="ChangedContent"):
        apply_source_edits(
            SOURCE,
            source_digest(SOURCE),
            [
                SourceEdit(
                    original.node_id,
                    original.start_byte,
                    original.end_byte,
                    "changed",
                    original.replacement,
                )
            ],
        )
    unicode_source = "* 设计\n:PROPERTIES:\n:ID: design-1\n:END:\n说明\n"
    split = unicode_source.encode().index("说明".encode()) + 1
    with pytest.raises(ValueError, match="InvalidRange"):
        apply_source_edits(
            unicode_source,
            source_digest(unicode_source),
            [SourceEdit("design-1", split, split + 1, "", "x")],
        )


def test_rejects_empty_edits():
    with pytest.raises(ValueError, match="EmptyEdits"):
        apply_source_edits(SOURCE, source_digest(SOURCE), [])


def test_rejects_duplicate_owners_and_overlapping_spans():
    first = edit(SOURCE, "design-1", "Old guarantee", "New guarantee")
    duplicate = edit(SOURCE, "design-1", "Design", "Architecture")
    with pytest.raises(ValueError, match="DuplicateNodeId"):
        apply_source_edits(SOURCE, source_digest(SOURCE), [first, duplicate])
    overlap = SourceEdit(
        "notes-1",
        first.start_byte + 1,
        first.end_byte,
        "ld guarantee",
        "replacement",
    )
    with pytest.raises(ValueError, match="Overlap"):
        apply_source_edits(SOURCE, source_digest(SOURCE), [first, overlap])
