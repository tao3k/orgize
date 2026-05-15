use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use orgize::{
    ast::{AgendaDate, AgendaQuery, ExportProjectionOptions},
    config::RadioLinkProjection,
    Org, ParseConfig,
};

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

pub fn bench_to_markdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::to_markdown");

    for &(id, org) in INPUT {
        let parsed = Org::parse(org);

        group.throughput(Throughput::Bytes(org.len() as u64));
        group.bench_with_input(id, &parsed, |b, i| b.iter(|| black_box(i.to_markdown())));
    }

    group.finish();
}

pub fn bench_to_latex(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::to_latex");

    for &(id, org) in INPUT {
        let parsed = Org::parse(org);

        group.throughput(Throughput::Bytes(org.len() as u64));
        group.bench_with_input(id, &parsed, |b, i| b.iter(|| black_box(i.to_latex())));
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

pub fn bench_dense_semantic_radio_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document/dense-semantic-radio-projection");
    let org = dense_semantic_radio_projection_fixture();
    let config = ParseConfig {
        radio_link_projection: RadioLinkProjection::Semantic,
        ..Default::default()
    };
    let parsed = config.parse(&org);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_with_input("many-parsed-object-radio-links.org", &parsed, |b, i| {
        b.iter(|| black_box(i.document()))
    });

    group.finish();
}

pub fn bench_dense_m15_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("Org::document/dense-m15-side-tables");
    let org = dense_m15_projection_fixture();
    let parsed = Org::parse(&org);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_with_input("many-m15-settings-links-footnotes.org", &parsed, |b, i| {
        b.iter(|| black_box(i.document()))
    });

    group.finish();
}

pub fn bench_dense_m15_export_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Document::project_for_export/dense-m15");
    let org = dense_m15_projection_fixture();
    let document = Org::parse(&org).document();
    let options = ExportProjectionOptions {
        prune: true,
        special_strings: true,
        headline_level_shift: 1,
        select_tags: vec!["publish".into()],
        exclude_tags: vec!["noexport".into()],
        ..Default::default()
    };

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_function("many-m15-settings-links-footnotes.org", |b| {
        b.iter(|| black_box(document.project_for_export(black_box(&options))))
    });

    group.finish();
}

pub fn bench_dense_agenda_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Document::agenda_entries/dense-agenda");
    let org = dense_agenda_projection_fixture();
    let document = Org::parse(&org).document();
    let query = AgendaQuery::new(AgendaDate::new(2026, 5, 1), AgendaDate::new(2026, 5, 31))
        .include_done(true)
        .include_closed(true)
        .include_archived(true);

    group.throughput(Throughput::Bytes(org.len() as u64));
    group.bench_function("many-agenda-planning-timestamps.org", |b| {
        b.iter(|| black_box(document.agenda_entries(black_box(&query))))
    });

    group.finish();
}

fn dense_target_projection_fixture() -> String {
    let mut org = String::new();

    for idx in 0usize..128 {
        org.push_str(&format!("<<<Radio Target {idx}>>> Radio Target {idx}\n"));
    }

    org.push('\n');

    for idx in 0usize..128 {
        org.push_str(&format!(
            "* Heading {idx}\n:PROPERTIES:\n:CUSTOM_ID: heading-{idx}\n:END:\n[[*Heading {idx}][headline]] [[#heading-{idx}][custom]] [[Radio Target {idx}][radio]] Radio Target {idx}\n"
        ));
    }

    org
}

fn dense_semantic_radio_projection_fixture() -> String {
    let mut org = String::new();

    for idx in 0..64 {
        org.push_str(&format!(
            "<<<*Signal {idx}*>>> <<<\\alpha{idx}>>> <<<~Code {idx}~>>>\n"
        ));
    }

    org.push('\n');

    for idx in 0..256 {
        let target = idx % 64;
        org.push_str(&format!(
            "Row {idx:03}: *Signal {target}* appears beside \\alpha{target} and ~Code {target}~, with /noise {idx}/ and [[https://example.com/{idx}][link {idx}]].\n"
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

fn dense_m15_projection_fixture() -> String {
    let mut org = String::from(
        r#"#+TITLE: *M15* benchmark
#+AUTHOR: Parser Bot
#+FILETAGS: :global:bench:
#+OPTIONS: H:3 -:t e:nil
#+SELECT_TAGS: publish
#+EXCLUDE_TAGS: noexport archived
#+LINK: gh https://github.com/%s
#+LINK: query https://example.test?q=%h

"#,
    );

    for idx in 0usize..128 {
        let target = idx.saturating_sub(1);
        let tags = match idx % 4 {
            0 => ":publish:",
            1 => ":noexport:",
            2 => ":ARCHIVE:",
            _ => ":bench:",
        };
        let keyword = if idx % 16 == 0 { "COMMENT " } else { "" };
        org.push_str(&format!(
            "* {keyword}Heading {idx} {tags}\n:PROPERTIES:\n:CUSTOM_ID: h-{idx}\n:ID: org-id-{idx}\n:END:\n#+ATTR_HTML: :class item :data-index \"{idx}\"\nParagraph -- more... with [[#h-{target}]], [[id:org-id-{target}::*Heading {target}]], [[gh:tao3k/orgize]], and [[query:a/b {idx}]].\nInline [fn::anonymous *inline* {idx}] plus [fn:named-{idx}:named inline] and [fn:named-{idx}].\nCitation [cite:see [nested {idx}] @doe{idx} p. [42]; cf. @roe{idx}].\n<<target-{idx}>> [[target-{idx}]]\n\n"
        ));
    }

    org
}

fn dense_agenda_projection_fixture() -> String {
    let mut org = String::from(
        "#+FILETAGS: :agenda:bench:\n#+CATEGORY: bench-agenda\n#+TODO: TODO NEXT | DONE CANCELED\n\n",
    );

    for idx in 0usize..256 {
        let todo = match idx % 8 {
            0 => "DONE",
            1 => "NEXT",
            _ => "TODO",
        };
        let tag = match idx % 5 {
            0 => ":work:",
            1 => ":ops:",
            2 => ":ARCHIVE:",
            3 => ":range:",
            _ => ":bench:",
        };
        let scheduled_day = idx % 28 + 1;
        let deadline_day = (idx + 3) % 28 + 1;
        let range_end_day = (scheduled_day + 1).min(28);
        let headline_time = if idx % 11 == 0 { " 8:30-1pm" } else { "" };

        org.push_str(&format!(
            "* {todo} Agenda item {idx}{headline_time} {tag}\n"
        ));
        if idx % 12 == 0 {
            org.push_str(":PROPERTIES:\n:CATEGORY: bench-work\n:END:\n");
        }
        if idx % 6 == 0 {
            org.push_str(&format!(
                "SCHEDULED: <2026-05-{scheduled_day:02} Fri 09:00>--<2026-05-{range_end_day:02} Sat 10:00 +1w> DEADLINE: <2026-05-{deadline_day:02} Mon -2d>\n"
            ));
        } else if idx % 7 == 0 {
            org.push_str(&format!(
                "SCHEDULED: <2026-05-{scheduled_day:02} Fri 09:00 -2d> DEADLINE: <2026-05-{deadline_day:02} Mon -2d>\n"
            ));
        } else {
            org.push_str(&format!(
                "SCHEDULED: <2026-05-{scheduled_day:02} Fri 09:00 +1w> DEADLINE: <2026-05-{deadline_day:02} Mon -2d>\n"
            ));
        }
        if idx % 10 == 0 {
            org.push_str(&format!("CLOSED: [2026-05-{scheduled_day:02} Fri]\n"));
        }
        if idx % 9 == 0 {
            org.push_str(&format!(
                "Body with active event <2026-05-{scheduled_day:02} Fri 14:00-15:00> and inactive note [2026-05-{deadline_day:02} Mon].\n\n"
            ));
        } else {
            org.push_str("Body with [[https://example.com][link]] and *markup*.\n\n");
        }
    }

    org
}

criterion_group!(
    benches,
    bench_parse,
    bench_document,
    bench_to_html,
    bench_to_markdown,
    bench_to_latex,
    bench_macro_expansions,
    bench_semantic_radio_links,
    bench_inlinetask_document,
    bench_dense_target_projection,
    bench_dense_annotation_projection,
    bench_dense_semantic_radio_projection,
    bench_dense_m15_document,
    bench_dense_m15_export_projection,
    bench_dense_agenda_projection
);
criterion_main!(benches);
