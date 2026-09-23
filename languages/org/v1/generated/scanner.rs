// Generated from languages/org/v1/scanner.ss. Do not edit.
pub const BLOCK_BEGIN: &str = "#+begin_src";
pub const BLOCK_END: &str = "#+end_src";
pub const SCANNER_DIGEST: &str =
    "sha256:4b5caa88e74e921339157be6f23b7b5600ed65e0cf4ee73dac1e206152fb3970";

use gerbil_parser_rowan::ScannedToken;

fn directive_line(line: &str, directive: &str, closing: bool) -> bool {
    let line = line.trim_start_matches([' ', '\t']);
    let Some(prefix) = line.get(..directive.len()) else {
        return false;
    };
    if !prefix.eq_ignore_ascii_case(directive) {
        return false;
    }
    let tail = &line[directive.len()..];
    if closing {
        tail.bytes()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
    } else {
        tail.chars()
            .next()
            .is_none_or(|next| matches!(next, ' ' | '\t' | '\r' | '\n'))
    }
}

fn headline_line(line: &str) -> bool {
    let level = line.bytes().take_while(|byte| *byte == b'*').count();
    level > 0 && matches!(line.as_bytes().get(level), Some(b' '))
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
        let block_end = inside_source_block && directive_line(line, BLOCK_END, true);
        let block_begin = !inside_source_block && directive_line(line, BLOCK_BEGIN, false);
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
