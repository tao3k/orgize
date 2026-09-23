//! Customer-owned scanner generated from the Scheme Org language pack.

#[path = "../../languages/org/v1/generated/scanner.rs"]
mod generated;

#[test]
fn source_block_suppresses_headline_tokens() {
    let source = "#+BEGIN_SRC rust\r\n* not a headline\r\n#+END_SRC\r\n* Real\n";
    assert_eq!(
        generated::scan(source),
        [
            ("block-begin", 0, 18),
            ("text", 18, 36),
            ("block-end", 36, 47),
            ("headline", 47, 54),
        ]
    );
}

#[test]
fn unicode_and_crlf_have_exact_utf8_coverage() {
    let source = "é\r\n** 标题\n";
    let tokens = generated::scan(source);
    assert_eq!(tokens, [("text", 0, 4), ("headline", 4, 14)]);
    let mut end = 0;
    for (_, start, next) in tokens {
        assert_eq!(start, end);
        assert!(source.is_char_boundary(start));
        assert!(source.is_char_boundary(next));
        end = next;
    }
    assert_eq!(end, source.len());
}

#[test]
fn directive_names_require_a_boundary() {
    assert_eq!(
        generated::scan("#+begin_srcx\n*not headline\n"),
        [("text", 0, 13), ("text", 13, 27)]
    );
}
