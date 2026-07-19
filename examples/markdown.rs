//! ```bash
//! cargo run --example markdown test.org
//! ```

use orgize::Org;
use std::{env::args, fs};

fn main() {
    let args: Vec<_> = args().collect();

    if args.len() < 2 {
        panic!("Usage: {} <org-mode-file>", args[0]);
    }

    let content = fs::read_to_string(&args[1]).unwrap();

    fs::write(format!("{}.md", args[1]), Org::parse(content).to_markdown()).unwrap();

    println!("Wrote to {}.md", args[1]);
}
