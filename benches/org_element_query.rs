//! Separate parser and composite-query costs on a dense Org task document.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use gerbil_parser_rowan::{parse_generated_events, project_syntax_graph};
use orgize::org_aot::{org_graph_spec, org_language_spec, parse_org_aot};

#[rustfmt::skip]
mod generated_context_events {
    use gerbil_parser_rowan::TreeEvent;
    include!(concat!(env!("OUT_DIR"), "/org_rowan_events.rs"));
}

fn task_source() -> String {
    let mut source = String::with_capacity(200_000);
    source.push_str("#+SEQ_TODO: WAIT | DONE\n* Team\n");
    for _ in 0..2_500 {
        source.push_str("** WAIT Review\n** WAIT Audit\n** WAIT Other\n** DONE Review\n");
    }
    source
}

fn table_source() -> String {
    let mut source = String::with_capacity(220_000);
    source.push_str("* Table\n");
    for _ in 0..10_000 {
        source.push_str("| a | b | c | d |\n");
    }
    source
}

fn list_source() -> String {
    let mut source = String::with_capacity(150_000);
    source.push_str("* Lists\n");
    for _ in 0..2_500 {
        source.push_str("- parent\n  - child\n  - second\n- peer\n");
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

    let events = generated_context_events::parse_org_rowan_events(&source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        &source,
        &events,
    )
    .expect("Scheme AOT benchmark events satisfy Rowan");
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme AOT benchmark events project Elements");
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "headline")
            .count(),
        10_001
    );
    assert_eq!(
        records
            .iter()
            .find(|record| record.kind == "keyword")
            .and_then(|record| record.field("key")),
        Some("SEQ_TODO")
    );

    let mut aot_group = c.benchmark_group("OrgSchemeEventAot");
    aot_group.throughput(Throughput::Elements(10_000));
    aot_group.bench_function("events-rowan/10k-headlines", |b| {
        b.iter(|| {
            let events = generated_context_events::parse_org_rowan_events(black_box(&source));
            black_box(
                parse_generated_events(
                    org_language_spec(),
                    generated_context_events::PARSER_DIGEST,
                    &source,
                    &events,
                )
                .unwrap(),
            )
        })
    });
    aot_group.bench_function("events-rowan-elements/10k-headlines", |b| {
        b.iter(|| {
            let events = generated_context_events::parse_org_rowan_events(black_box(&source));
            let parsed = parse_generated_events(
                org_language_spec(),
                generated_context_events::PARSER_DIGEST,
                &source,
                &events,
            )
            .unwrap();
            black_box(
                project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
                    .unwrap(),
            )
        })
    });
    let unmatched_blocks = "#+begin_src rust\n".repeat(10_000);
    aot_group.bench_function("events-rowan/10k-unclosed-blocks", |b| {
        b.iter(|| {
            let events =
                generated_context_events::parse_org_rowan_events(black_box(&unmatched_blocks));
            black_box(
                parse_generated_events(
                    org_language_spec(),
                    generated_context_events::PARSER_DIGEST,
                    &unmatched_blocks,
                    &events,
                )
                .unwrap(),
            )
        })
    });
    let inline_objects = "~code~ =verbatim=\n".repeat(10_000);
    let inline_records = orgize::org_aot::parse_org_aot(&inline_objects)
        .expect("inline benchmark source builds a Rowan document");
    assert_eq!(
        inline_records
            .records()
            .iter()
            .filter(|record| record.kind == "code" || record.kind == "verbatim")
            .count(),
        20_000
    );
    aot_group.bench_function("events-rowan-elements/10k-inline-objects", |b| {
        b.iter(|| black_box(orgize::org_aot::parse_org_aot(black_box(&inline_objects)).unwrap()))
    });
    let target_objects = "<<target>> <<<radio>>>\n".repeat(10_000);
    let target_records = orgize::org_aot::parse_org_aot(&target_objects)
        .expect("target benchmark source builds a Rowan document");
    assert_eq!(
        target_records
            .records()
            .iter()
            .filter(|record| record.kind == "target" || record.kind == "radio-target")
            .count(),
        20_000
    );
    aot_group.bench_function("events-rowan-elements/10k-target-objects", |b| {
        b.iter(|| black_box(orgize::org_aot::parse_org_aot(black_box(&target_objects)).unwrap()))
    });
    let terminal_objects = "[50%] [2/3] text\\\\\n".repeat(10_000);
    let terminal_records = orgize::org_aot::parse_org_aot(&terminal_objects)
        .expect("terminal-Object benchmark source builds a Rowan document");
    assert_eq!(
        terminal_records
            .records()
            .iter()
            .filter(|record| record.kind == "statistics-cookie" || record.kind == "line-break")
            .count(),
        30_000
    );
    aot_group.bench_function("events-rowan-elements/10k-terminal-objects", |b| {
        b.iter(|| black_box(orgize::org_aot::parse_org_aot(black_box(&terminal_objects)).unwrap()))
    });
    let plain_lines = "plain words here\n".repeat(10_000);
    aot_group.bench_function("events-rowan-elements/10k-plain-lines", |b| {
        b.iter(|| black_box(orgize::org_aot::parse_org_aot(black_box(&plain_lines)).unwrap()))
    });
    aot_group.finish();
}

fn bench_org_table_rows(c: &mut Criterion) {
    let source = table_source();
    let structural = parse_org_aot(&source).expect("structural table benchmark parses");
    let events = generated_context_events::parse_org_rowan_events(&source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        &source,
        &events,
    )
    .expect("Scheme table benchmark events satisfy Rowan");
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme table benchmark projects Elements");
    for (kind, expected) in [("table", 1), ("table-row", 10_000), ("table-cell", 40_000)] {
        assert_eq!(
            structural
                .records()
                .iter()
                .filter(|record| record.kind == kind)
                .count(),
            expected
        );
        assert_eq!(
            records.iter().filter(|record| record.kind == kind).count(),
            expected
        );
    }

    let mut group = c.benchmark_group("OrgSchemeEventAotTable");
    group.throughput(Throughput::Elements(10_000));
    group.bench_function("structural-elements/10k-rows", |b| {
        b.iter(|| black_box(parse_org_aot(black_box(&source)).unwrap()))
    });
    group.bench_function("events-rowan-elements/10k-rows", |b| {
        b.iter(|| {
            let events = generated_context_events::parse_org_rowan_events(black_box(&source));
            let parsed = parse_generated_events(
                org_language_spec(),
                generated_context_events::PARSER_DIGEST,
                &source,
                &events,
            )
            .unwrap();
            black_box(
                project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
                    .unwrap(),
            )
        })
    });
    group.finish();
}

fn bench_org_nested_lists(c: &mut Criterion) {
    let source = list_source();
    let structural = parse_org_aot(&source).expect("structural list benchmark parses");
    let events = generated_context_events::parse_org_rowan_events(&source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        &source,
        &events,
    )
    .expect("Scheme list benchmark events satisfy Rowan");
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme list benchmark projects Elements");
    for (kind, expected) in [("plain-list", 2_501), ("item", 10_000)] {
        assert_eq!(
            structural
                .records()
                .iter()
                .filter(|record| record.kind == kind)
                .count(),
            expected
        );
        assert_eq!(
            records.iter().filter(|record| record.kind == kind).count(),
            expected
        );
    }

    let mut group = c.benchmark_group("OrgSchemeEventAotList");
    group.throughput(Throughput::Elements(10_000));
    group.bench_function("structural-elements/10k-items", |b| {
        b.iter(|| black_box(parse_org_aot(black_box(&source)).unwrap()))
    });
    group.bench_function("events-rowan-elements/10k-items", |b| {
        b.iter(|| {
            let events = generated_context_events::parse_org_rowan_events(black_box(&source));
            let parsed = parse_generated_events(
                org_language_spec(),
                generated_context_events::PARSER_DIGEST,
                &source,
                &events,
            )
            .unwrap();
            black_box(
                project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
                    .unwrap(),
            )
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_org_element_query,
    bench_org_table_rows,
    bench_org_nested_lists
);
criterion_main!(benches);
