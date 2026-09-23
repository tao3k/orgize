#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only Scheme AOT generator; Cargo consumes committed Rust.

(load "languages/org/v1/scanner.ss")

(def (directive name)
  (cdr (assq name +org-scanner-directives+)))

(def (emit-scanner port)
  (display "// Generated from languages/org/v1/scanner.ss. Do not edit.\n" port)
  (display "pub const BLOCK_BEGIN: &str = " port)
  (write (directive 'block-begin) port)
  (display ";\npub const BLOCK_END: &str = " port)
  (write (directive 'block-end) port)
  (display ";\n" port)
  (display #<<RUST

use gerbil_parser_rowan::ScannedToken;

fn directive_line(line: &str, directive: &str) -> bool {
    let Some(prefix) = line.get(..directive.len()) else {
        return false;
    };
    prefix.eq_ignore_ascii_case(directive)
        && line[directive.len()..]
            .chars()
            .next()
            .is_none_or(|next| matches!(next, ' ' | '\t' | '\r' | '\n'))
}

fn headline_line(line: &str) -> bool {
    let level = line.bytes().take_while(|byte| *byte == b'*').count();
    level > 0 && matches!(line.as_bytes().get(level), Some(b' ' | b'\t'))
}

/// Full-source UTF-8 byte coverage for the Org customer language pack.
pub fn scan(source: &str) -> Vec<ScannedToken> {
    let mut result = Vec::new();
    let mut inside_source_block = false;
    let mut start = 0;
    while start < source.len() {
        let end = source[start..]
            .find('\n')
            .map_or(source.len(), |offset| start + offset + 1);
        let line = &source[start..end];
        let block_end = inside_source_block && directive_line(line, BLOCK_END);
        let block_begin = !inside_source_block && directive_line(line, BLOCK_BEGIN);
        let terminal = if block_end {
            "block-end"
        } else if block_begin {
            "block-begin"
        } else if inside_source_block {
            "text"
        } else if headline_line(line) {
            "headline"
        } else {
            "text"
        };
        result.push(ScannedToken {
            terminal,
            start,
            end,
        });
        if block_end {
            inside_source_block = false;
        } else if block_begin {
            inside_source_block = true;
        }
        start = end;
    }
    result
}
RUST
           port)
  (newline port))

(def arguments (command-line))
(unless (>= (length arguments) 3)
  (error "usage: gxi languages/org/v1/generate-scanner.ss OUTPUT"))
(call-with-output-file (car (reverse arguments))
  (lambda (port) (emit-scanner port)))
