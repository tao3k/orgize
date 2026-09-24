//! Separate parser and composite-query costs on a dense Org task document.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use orgize::org_aot::parse_org_aot;

fn task_source() -> String {
    let mut source = String::with_capacity(200_000);
    source.push_str("#+SEQ_TODO: WAIT | DONE\n* Team\n");
    for _ in 0..2_500 {
        source.push_str("** WAIT Review\n** WAIT Audit\n** WAIT Other\n** DONE Review\n");
    }
    source
}

fn bench_org_element_query(c: &mut Criterion) {
    let source = task_source();
    let document = parse_org_aot(&source).expect("benchmark document is valid Org");
    let scope = document
        .records()
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("Team"))
        .expect("Team headline")
        .id;
    assert_eq!(
        document
            .query_named("tasks.review-or-audit", scope)
            .unwrap()
            .len(),
        5_000
    );

    let mut group = c.benchmark_group("OrgAotDocument");
    group.throughput(Throughput::Elements(10_000));
    group.bench_function("parse/10k-headlines", |b| {
        b.iter(|| black_box(parse_org_aot(black_box(&source)).unwrap()))
    });
    group.bench_function("query/composite/10k-headlines", |b| {
        b.iter(|| {
            black_box(
                document
                    .query_named(black_box("tasks.review-or-audit"), black_box(scope))
                    .unwrap(),
            )
        })
    });
    group.finish();
}

criterion_group!(benches, bench_org_element_query);
criterion_main!(benches);
