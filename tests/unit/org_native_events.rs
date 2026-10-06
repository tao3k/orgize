use super::decode;
use crate::org_aot::org_language_spec;
use gerbil_parser_rowan::{KindCategory, TreeEvent};

fn header() -> Vec<u8> {
    format!("OEV1{}", org_language_spec().grammar_digest).into_bytes()
}

fn kind(category: KindCategory) -> u16 {
    u16::try_from(
        org_language_spec()
            .kinds
            .iter()
            .position(|spec| spec.category == category)
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn rejects_missing_truncated_or_foreign_identity() {
    let valid = header();
    for end in 0..valid.len() {
        assert!(decode(&valid[..end], org_language_spec()).is_err());
    }
    let mut foreign = valid;
    *foreign.last_mut().unwrap() ^= 1;
    assert!(decode(&foreign, org_language_spec()).is_err());
}

#[test]
fn rejects_every_truncated_record_width() {
    for (tag, width) in [(1, 2), (2, 18)] {
        for length in 0..width {
            let mut tape = header();
            tape.push(tag);
            tape.extend(std::iter::repeat_n(0, length));
            assert!(decode(&tape, org_language_spec()).is_err());
        }
    }
}

#[test]
fn rejects_unknown_tags_kinds_and_cross_category_records() {
    for tag in 3..=u8::MAX {
        let mut tape = header();
        tape.push(tag);
        assert!(decode(&tape, org_language_spec()).is_err());
    }
    for (tag, id) in [
        (1, u16::MAX),
        (2, u16::MAX),
        (1, kind(KindCategory::Token)),
        (2, kind(KindCategory::Node)),
    ] {
        let mut tape = header();
        tape.push(tag);
        tape.extend_from_slice(&id.to_le_bytes());
        if tag == 2 {
            tape.extend_from_slice(&[0; 16]);
        }
        assert!(decode(&tape, org_language_spec()).is_err());
    }
}

#[test]
fn projects_little_endian_records_without_reinterpreting_source() {
    let node = kind(KindCategory::Node);
    let token = kind(KindCategory::Token);
    let mut tape = header();
    tape.push(1);
    tape.extend_from_slice(&node.to_le_bytes());
    tape.push(2);
    tape.extend_from_slice(&token.to_le_bytes());
    tape.extend_from_slice(&258u64.to_le_bytes());
    tape.extend_from_slice(&513u64.to_le_bytes());
    tape.push(0);
    let events = decode(&tape, org_language_spec()).unwrap();
    assert_eq!(
        events,
        [
            TreeEvent::StartNode(node),
            TreeEvent::Token {
                kind: token,
                start: 258,
                end: 513
            },
            TreeEvent::FinishNode
        ]
    );
}

#[test]
fn rowan_admission_rejects_invalid_source_ranges_and_nesting() {
    let grammar = org_language_spec();
    for suffix in [vec![0], {
        let mut record = vec![1];
        record.extend_from_slice(&kind(KindCategory::Node).to_le_bytes());
        record.push(2);
        record.extend_from_slice(&kind(KindCategory::Token).to_le_bytes());
        record.extend_from_slice(&1u64.to_le_bytes());
        record.extend_from_slice(&0u64.to_le_bytes());
        record.push(0);
        record
    }] {
        let mut tape = header();
        tape.extend(suffix);
        let events = decode(&tape, grammar).unwrap();
        assert!(
            gerbil_parser_rowan::parse_generated_events(
                grammar,
                crate::org_aot::org_event_parser_digest(),
                "",
                &events
            )
            .is_err()
        );
    }
}
