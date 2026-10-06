//! ```bash
//! cargo run --example html-slugify '* hello world!'
//! ```

use orgize::Org;
use slugify::slugify;
use std::env::args;

fn main() {
    // SAFETY: standalone entrypoint, before workers or children and host I/O.
    unsafe { orgize::initialize_native_runtime() }.expect("native startup");
    let args: Vec<_> = args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <org-mode-string>", args[0]);
    } else {
        let html = Org::parse(&args[1])
            .try_to_html_with_headline_anchor(|title| slugify!(title))
            .expect("Scheme AOT HTML projection");
        println!("{html}");
    }
}
