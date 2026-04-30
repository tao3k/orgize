# Orgize

[![Crates.io](https://img.shields.io/crates/v/orgize.svg)](https://crates.io/crates/orgize)
[![Documentation](https://docs.rs/orgize/badge.svg)](https://docs.rs/orgize)
[![Build status](https://img.shields.io/github/actions/workflow/status/tao3k/orgize/ci.yml)](https://github.com/tao3k/orgize/actions/workflows/ci.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

A Rust library for parsing org-mode files.

Live Demo: <https://tao3k.github.io/orgize/>

## Parse

To parse a org-mode string, simply invoking the `Org::parse` function:

```rust
use orgize::{Org, rowan::ast::AstNode};

let org = Org::parse("* DONE Title :tag:");
assert_eq!(
    format!("{:#?}", org.document().syntax()),
    r#"DOCUMENT@0..18
  HEADLINE@0..18
    HEADLINE_STARS@0..1 "*"
    WHITESPACE@1..2 " "
    HEADLINE_KEYWORD_DONE@2..6 "DONE"
    WHITESPACE@6..7 " "
    HEADLINE_TITLE@7..13
      TEXT@7..13 "Title "
    HEADLINE_TAGS@13..18
      COLON@13..14 ":"
      TEXT@14..17 "tag"
      COLON@17..18 ":"
"#);
```

use `ParseConfig::parse` to specific a custom parse config

```rust
use orgize::{Org, ParseConfig, ast::Headline};

let config = ParseConfig {
    // custom todo keywords
    todo_keywords: (vec!["TASK".to_string()], vec![]),
    ..Default::default()
};
let org = config.parse("* TASK Title 1");
let hdl = org.first_node::<Headline>().unwrap();
assert_eq!(hdl.todo_keyword().unwrap(), "TASK");
```

## Traverse

Use `org.traverse(&mut traversal)` to walk through the syntax tree.

```rust
use orgize::{
    export::{from_fn, Container, Event},
    Org,
};

let mut hdl_count = 0;
let mut handler = from_fn(|event| {
    if matches!(event, Event::Enter(Container::Headline(_))) {
        hdl_count += 1;
    }
});
Org::parse("* 1\n** 2\n*** 3\n****4").traverse(&mut handler);
assert_eq!(hdl_count, 3);
```

## Modify

Use `org.replace_range(TextRange::new(start, end), "new_text")` to modify the syntax tree:

```rust
use orgize::{Org, ParseConfig, ast::Headline, TextRange};

let mut org = Org::parse("hello\n* world");

let hdl = org.first_node::<Headline>().unwrap();
org.replace_range(hdl.text_range(), "** WORLD!");

let hdl = org.first_node::<Headline>().unwrap();
assert_eq!(hdl.level(), 2);

org.replace_range(TextRange::up_to(hdl.start()), "");
assert_eq!(org.to_org(), "** WORLD!");
```

## Render to html

Call the `Org::to_html` function to export org element tree to html:

```rust
use orgize::Org;

assert_eq!(
    Org::parse("* title\n*section*").to_html(),
    "<main><h1>title</h1><section><p><b>section</b></p></section></main>"
);
```

Checkout `examples/html-slugify.rs` on how to customizing html export process.

## Features

- **`chrono`**: adds the ability to convert `Timestamp` into `chrono::NaiveDateTime`, disabled by default.

- **`indexmap`**: adds the ability to convert `PropertyDrawer` properties into `IndexMap`, disabled by default.

## API compatibility

`element.syntax()` exposes access to the internal syntax tree, along with some rowan low-level APIs.
This can be useful for intricate tasks.

However, the structure of the internal syntax tree can change between different versions of the library.
Because of this, the result of `element.syntax()` doesn't follow semantic versioning,
which means updates might break your code if it relies on this method.
