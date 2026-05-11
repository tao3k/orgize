use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use orgize::{config::RadioLinkProjection, Org, ParseConfig};

const INPUT: &[(&str, &str)] = &[
    ("doc.org", include_str!("./fixtures/doc.org")),
    ("org-faq.org", include_str!("./fixtures/org-faq.org")),
    ("org-hacks.org", include_str!("./fixtures/org-hacks.org")),
    (
        "org-release-notes.org",
        include_str!("./fixtures/org-release-notes.org"),
    ),
    ("org-syntax.org", include_str!("./fixtures/org-syntax.org")),
    (
        "plain-links.org",
        include_str!("./fixtures/plain-links.org"),
    ),
    (
        "radio-links.org",
        include_str!("./fixtures/radio-links.org"),
    ),
    (
        "block-line-numbers.org",
        include_str!("./fixtures/block-line-numbers.org"),
    ),
    (
        "block-code-refs.org",
        include_str!("./fixtures/block-code-refs.org"),
    ),
    (
        "block-header-args.org",
        include_str!("./fixtures/block-header-args.org"),
    ),
    (
        "table-column-metadata.org",
        include_str!("./fixtures/table-column-metadata.org"),
    ),
    (
        "file-todo-keywords.org",
        include_str!("./fixtures/file-todo-keywords.org"),
    ),
    (
        "preprocessing-directives.org",
        include_str!("./fixtures/preprocessing-directives.org"),
    ),
    (
        "internal-links.org",
        include_str!("./fixtures/internal-links.org"),
    ),
    (
        "quote-heavy.org",
        include_str!("./fixtures/quote-heavy.org"),
    ),
];

pub fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::parse");

    for &(id, org) in INPUT {
        group.throughput(Throughput::Bytes(org.len() as u64));
        group.bench_with_input(id, org, |b, i| {
            b.iter(|| black_box(Org::parse(black_box(i))))
        });
    }

    group.finish();
}

pub fn bench_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document");

    for &(id, org) in INPUT {
        let parsed = Org::parse(org);

        group.throughput(Throughput::Bytes(org.len() as u64));
        group.bench_with_input(id, &parsed, |b, i| b.iter(|| black_box(i.document())));
    }

    group.finish();
}

pub fn bench_to_html(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::to_html");

    for &(id, org) in INPUT {
        let parsed = Org::parse(org);

        group.throughput(Throughput::Bytes(org.len() as u64));
        group.bench_with_input(id, &parsed, |b, i| b.iter(|| black_box(i.to_html())));
    }

    group.finish();
}

pub fn bench_macro_expansions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::macro_expansions");

    for &(id, org) in INPUT {
        let document = Org::parse(org).document();

        group.throughput(Throughput::Bytes(org.len() as u64));
        group.bench_with_input(id, &document, |b, i| {
            b.iter(|| black_box(i.macro_expansions()))
        });
    }

    let dense_macros = dense_macro_expansion_fixture();
    let dense_document = Org::parse(&dense_macros).document();
    group.throughput(Throughput::Bytes(dense_macros.len() as u64));
    group.bench_with_input("dense-macro-expansions.org", &dense_document, |b, i| {
        b.iter(|| black_box(i.macro_expansions()))
    });

    group.finish();
}

pub fn bench_semantic_radio_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document/radio-link-projection");
    let org = r#"<<<*Radio Target*>>> links *Radio Target* here.
Paragraph with *Radio Target* and \alpha but not Radio Targets.
"#;
    let config = ParseConfig {
        radio_link_projection: RadioLinkProjection::Semantic,
        ..Default::default()
    };
    let parsed = config.parse(org);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_with_input("semantic-object-spans.org", &parsed, |b, i| {
        b.iter(|| black_box(i.document()))
    });

    group.finish();
}

pub fn bench_inlinetask_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document/inlinetasks");
    let org = r#"Intro.

*************** TODO [#A] *Inline* task :bench:
SCHEDULED: <2026-05-10 Sun>
:PROPERTIES:
:CUSTOM_ID: bench-inline
:END:
Body with [[https://example.com][link]].
*************** END

* Outline
Body.
"#;
    let parsed = Org::parse(org);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_with_input("closed-inlinetask.org", &parsed, |b, i| {
        b.iter(|| black_box(i.document()))
    });

    group.finish();
}

pub fn bench_dense_target_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document/dense-target-projection");
    let org = dense_target_projection_fixture();
    let parsed = Org::parse(&org);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_with_input("many-targets-and-radio-links.org", &parsed, |b, i| {
        b.iter(|| black_box(i.document()))
    });

    group.finish();
}

pub fn bench_dense_annotation_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document/dense-annotation-projection");
    let org = dense_annotation_projection_fixture();
    let parsed = Org::parse(&org);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_with_input("many-annotated-ascii-objects.org", &parsed, |b, i| {
        b.iter(|| black_box(i.document()))
    });

    group.finish();
}

fn dense_target_projection_fixture() -> String {
    let mut org = String::new();

    for idx in 0..128 {
        org.push_str(&format!("<<<Radio Target {idx}>>> Radio Target {idx}\n"));
    }

    org.push('\n');

    for idx in 0..128 {
        org.push_str(&format!(
            "* Heading {idx}\n:PROPERTIES:\n:CUSTOM_ID: heading-{idx}\n:END:\n[[*Heading {idx}][headline]] [[#heading-{idx}][custom]] [[Radio Target {idx}][radio]] Radio Target {idx}\n"
        ));
    }

    org
}

fn dense_annotation_projection_fixture() -> String {
    let mut org = String::new();

    for idx in 0..256 {
        org.push_str(&format!(
            "Line {idx:03} with *bold {idx}* /italic {idx}/ _under {idx}_ +strike {idx}+ ~code {idx}~ =verb {idx}= and [[https://example.com/{idx}][link {idx}]].\n"
        ));
    }

    org
}

fn dense_macro_expansion_fixture() -> String {
    let mut org = String::new();

    for idx in 0..32 {
        org.push_str(&format!("#+MACRO: m{idx} $0 :: $1 :: $0\n"));
    }

    org.push('\n');

    for idx in 0..256 {
        org.push_str(&format!(
            "{{{{{{m{}(alpha {}, beta {})}}}}}} ",
            idx % 32,
            idx,
            idx
        ));
        if idx % 8 == 7 {
            org.push('\n');
        }
    }

    org
}

criterion_group!(
    benches,
    bench_parse,
    bench_document,
    bench_to_html,
    bench_macro_expansions,
    bench_semantic_radio_links,
    bench_inlinetask_document,
    bench_dense_target_projection,
    bench_dense_annotation_projection
);
criterion_main!(benches);
