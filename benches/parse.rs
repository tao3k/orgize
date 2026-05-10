use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use orgize::Org;

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

criterion_group!(benches, bench_parse, bench_document, bench_to_html);
criterion_main!(benches);
