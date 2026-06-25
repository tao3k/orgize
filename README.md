# Orgize

[![Crates.io](https://img.shields.io/crates/v/orgize.svg)](https://crates.io/crates/orgize)
[![Documentation](https://docs.rs/orgize/badge.svg)](https://docs.rs/orgize)
[![Build status](https://img.shields.io/github/actions/workflow/status/tao3k/orgize/ci.yml)](https://github.com/tao3k/orgize/actions/workflows/ci.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

Orgize is a Rust library for parsing Org mode documents. It keeps parsing
non-mutating by default: source blocks, links, agenda metadata, capture plans,
publishing graphs, and runtime-adjacent Org features are projected as
source-backed data instead of being executed.

Live demo: <https://tao3k.github.io/orgize/>

## Parse

```rust
use orgize::{ast::ElementData, Org};

let org = Org::parse("* DONE Title :tag:");
let document = org.document();

assert_eq!(document.sections[0].level, 1);
assert_eq!(document.sections[0].raw_title, "Title ");
assert_eq!(document.sections[0].tags, ["tag"]);
assert!(document.sections[0].children.iter().all(|element| {
    !matches!(element.data, ElementData::Unknown { .. })
}));
```

Use `ParseConfig::parse` when a document needs custom parser settings:

```rust
use orgize::{syntax_ast::Headline, Org, ParseConfig};

let config = ParseConfig {
    todo_keywords: (vec!["TASK".to_string()], vec![]),
    ..Default::default()
};

let org = config.parse("* TASK Title 1");
let headline = org.first_node::<Headline>().unwrap();
assert_eq!(headline.todo_keyword().unwrap(), "TASK");
```

## Syntax Tree

Use `Org::syntax_document()` for the lossless rowan-backed syntax tree:

```rust
use orgize::{rowan::ast::AstNode, syntax_ast::Headline, Org};

let org = Org::parse("* Title");
let syntax_doc = org.syntax_document();
let headline = syntax_doc.syntax().children().find_map(Headline::cast).unwrap();

assert_eq!(headline.title_raw(), "Title");
```

## Traverse

```rust
use orgize::{
    export::{from_fn, Container, Event},
    Org,
};

let mut headline_count = 0;
let mut handler = from_fn(|event| {
    if matches!(event, Event::Enter(Container::Headline(_))) {
        headline_count += 1;
    }
});

Org::parse("* 1\n** 2\n*** 3\n****4").traverse(&mut handler);
assert_eq!(headline_count, 3);
```

## Modify

```rust
use orgize::{syntax_ast::Headline, Org, TextRange};

let mut org = Org::parse("hello\n* world");
let headline = org.first_node::<Headline>().unwrap();

org.replace_range(headline.text_range(), "** WORLD!");
let headline = org.first_node::<Headline>().unwrap();

assert_eq!(headline.level(), 2);
org.replace_range(TextRange::up_to(headline.start()), "");
assert_eq!(org.to_org(), "** WORLD!");
```

## Documentation

The README is the crate entrypoint. Long-lived feature notes, parser surface
maps, release evidence, and architecture records live under `docs/`.

- `docs/index.org`: documentation coordinate index
- `docs/20_parser/20.05_parser_surface_map.org`: parser and semantic projection surface map
- <https://docs.rs/orgize>: public Rust API documentation

## Features

- `chrono`: timestamp integration
- `datafusion-sql`: in-process SQL over the stable `org_elements` table projection
- `indexmap`: indexmap-backed collections where enabled
- `md`: Markdown export support through `comrak`
- `syntax-org-fc`: syntax support for Org-fc-style use cases
