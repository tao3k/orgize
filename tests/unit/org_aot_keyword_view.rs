use super::KeywordPlans;

fn row(fields: &[&str]) -> Vec<String> {
    fields.iter().map(|field| (*field).to_owned()).collect()
}

#[test]
fn keyword_rows_reject_cross_record_and_malformed_framing() {
    let valid = vec![
        row(&["keyword", "0", "3"]),
        row(&["route", ""]),
        row(&["first", ""]),
        row(&["rest", ""]),
    ];
    let plans = KeywordPlans::admit(&[7], valid.clone()).unwrap();
    assert_eq!(plans.get(7).unwrap().len(), 3);
    assert!(plans.get(0).is_none());
    assert!(KeywordPlans::admit(&[], vec![]).unwrap().get(7).is_none());
    for replacement in [
        row(&["keyword", "1", "3"]),
        row(&["keyword", "00", "3"]),
        row(&["keyword", "0", "03"]),
        row(&["keyword", "0", "4"]),
        row(&["keyword", "0", "18446744073709551616"]),
    ] {
        let mut malformed = valid.clone();
        malformed[0] = replacement;
        assert!(KeywordPlans::admit(&[7], malformed).is_err());
    }
    assert!(KeywordPlans::admit(&[7], vec![]).is_err());
    assert!(KeywordPlans::admit(&[], valid.clone()).is_err());
    let mut duplicate = valid.clone();
    duplicate[3] = row(&["route", "TITLE"]);
    assert!(KeywordPlans::admit(&[7], duplicate).is_err());
    let mut arity = valid;
    arity[3] = row(&["rest"]);
    assert!(KeywordPlans::admit(&[7], arity).is_err());
}
