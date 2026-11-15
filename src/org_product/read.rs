//! Admit the linked Scheme program’s document product at the host boundary.

use super::view::SyntaxRow;
use super::{
    GraphFieldValue, GraphNodeRule, GraphProjectionSpec, GraphRecord, KindCategory, LanguageSpec,
    SyntaxKind, SyntaxNode, TextRange, TextSize,
};

#[derive(Debug)]
pub(crate) struct DocumentProduct {
    pub syntax: Option<SyntaxNode>,
    pub records: Vec<GraphRecord>,
    pub subtree_ends: Vec<usize>,
    pub ledger_title: Option<[String; 3]>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    source: &'a str,
}

impl Reader<'_> {
    fn word(&mut self) -> Result<u32, String> {
        let (word, rest) = self
            .bytes
            .split_at_checked(4)
            .ok_or("truncated Org document column")?;
        self.bytes = rest;
        Ok(u32::from_le_bytes(word.try_into().unwrap()))
    }

    fn text(&mut self) -> Result<String, String> {
        let size = self.word()? as usize;
        let (text, rest) = self
            .bytes
            .split_at_checked(size)
            .ok_or("truncated Ledger text")?;
        self.bytes = rest;
        Ok(std::str::from_utf8(text)
            .map_err(|_| "invalid Ledger UTF-8")?
            .to_owned())
    }

    fn count(&mut self, row_width: usize) -> Result<usize, String> {
        let count = self.word()? as usize;
        if count > self.bytes.len() / row_width {
            return Err("truncated Org document rows".into());
        }
        Ok(count)
    }

    fn parent(&mut self, id: usize) -> Result<Option<usize>, String> {
        match self.word()? {
            u32::MAX if id == 0 => Ok(None),
            parent if (parent as usize) < id => Ok(Some(parent as usize)),
            _ => Err("invalid Org document parent".into()),
        }
    }

    fn range(&mut self) -> Result<TextRange, String> {
        let start = self.word()?;
        let end = self.word()?;
        if self.source.get(start as usize..end as usize).is_none() {
            return Err("invalid Org document source range".into());
        }
        Ok(TextRange::new(start.into(), end.into()))
    }

    fn subtree_end(&mut self, id: usize, size: usize) -> Result<usize, String> {
        let end = self.word()? as usize;
        if end <= id || end > size || (id == 0 && end != size) {
            return Err("invalid Org subtree interval".into());
        }
        Ok(end)
    }

    fn navigation(&mut self, count: usize, catalog: &LanguageSpec) -> Result<SyntaxNode, String> {
        let mut rows = Vec::<SyntaxRow>::with_capacity(count);
        let mut position = TextSize(0);
        let mut active: Option<usize> = None;
        for id in 0..count {
            while let Some(owner) = active {
                let row: &SyntaxRow = &rows[owner];
                if row.subtree_end != id {
                    break;
                }
                if row.range.end() != position {
                    return Err("invalid Org syntax close range".into());
                }
                active = row.parent;
            }
            let kind = u16::try_from(self.word()?).map_err(|_| "invalid Org syntax kind")?;
            let spec = catalog
                .kinds
                .get(usize::from(kind))
                .ok_or("unknown Org syntax kind")?;
            let token = spec.category == KindCategory::Token;
            let parent = self.parent(id)?;
            let range = self.range()?;
            let subtree_end = self.subtree_end(id, count)?;
            if parent != active
                || range.start() != position
                || (id == 0 && (kind != catalog.root_kind || token))
                || parent.is_some_and(|owner| {
                    let row: &SyntaxRow = &rows[owner];
                    subtree_end > row.subtree_end || range.end() > row.range.end()
                })
                || (token && (subtree_end != id + 1 || range.end() == position))
            {
                return Err("invalid Org navigation row".into());
            }
            if token {
                position = range.end();
            } else {
                active = Some(id);
            }
            rows.push(SyntaxRow {
                kind: SyntaxKind(kind),
                token,
                range,
                parent,
                subtree_end,
            });
        }
        if usize::from(position) != self.source.len() {
            return Err("Org syntax lacks complete source coverage".into());
        }
        Ok(SyntaxNode::from_rows(self.source, rows))
    }

    fn field(&mut self, rule: &GraphNodeRule) -> Result<GraphFieldValue, String> {
        let ordinal = self.word()? as usize;
        let name = rule
            .fields
            .get(ordinal)
            .ok_or("unknown Org Element field")?
            .name;
        let count = self.count(8)?;
        if count == 0 {
            return Err("missing Org Element field range".into());
        }
        let first = self.range()?;
        let mut value = first.slice(self.source).to_owned();
        let (mut start, mut end) = (first.start(), first.end());
        for _ in 1..count {
            let range = self.range()?;
            value.push_str(range.slice(self.source));
            start = start.min(range.start());
            end = end.max(range.end());
        }
        Ok(GraphFieldValue {
            name,
            value,
            range: TextRange::new(start, end),
        })
    }
}

impl DocumentProduct {
    pub(crate) fn decode(
        bytes: &[u8],
        source: &str,
        language: &LanguageSpec,
        graph: &GraphProjectionSpec,
        ledger: bool,
    ) -> Result<Self, String> {
        let bytes = bytes
            .strip_prefix(b"ODV1")
            .and_then(|bytes| bytes.strip_prefix(language.grammar_digest.as_bytes()))
            .and_then(|bytes| bytes.strip_prefix(graph.projection_digest.as_bytes()))
            .ok_or("Org document product identity mismatch")?;
        let mut reader = Reader { bytes, source };
        let syntax_count = reader.count(20)?;
        let element_count = reader.count(24)?;
        if (syntax_count == 0) != ledger || element_count == 0 {
            return Err("invalid Org document product shape".into());
        }
        let syntax = if ledger {
            None
        } else {
            Some(reader.navigation(syntax_count, language)?)
        };
        let mut records = Vec::<GraphRecord>::with_capacity(element_count);
        let mut subtree_ends = Vec::with_capacity(element_count);
        let mut active: Option<usize> = None;
        for id in 0..element_count {
            while let Some(owner) = active {
                if subtree_ends[owner] != id {
                    break;
                }
                active = records[owner].parent_id;
            }
            let ordinal = reader.word()? as usize;
            let rule = graph.rules.get(ordinal).ok_or("unknown Org Element rule")?;
            let parent_id = reader.parent(id)?;
            let range = reader.range()?;
            let subtree_end = reader.subtree_end(id, element_count)?;
            if parent_id != active
                || (id == 0
                    && (rule.syntax_kind != language.root_kind || range.slice(source) != source))
                || parent_id.is_some_and(|owner| subtree_end > subtree_ends[owner])
            {
                return Err("invalid Org Element ancestry".into());
            }
            let count = reader.count(16)?;
            let fields = (0..count)
                .map(|_| reader.field(rule))
                .collect::<Result<_, _>>()?;
            records.push(GraphRecord {
                id,
                parent_id,
                child_ids: Vec::new(),
                syntax_kind: rule.syntax_kind,
                category: rule.category,
                kind: rule.kind,
                range,
                fields,
            });
            if let Some(parent) = parent_id {
                records[parent].child_ids.push(id);
            }
            subtree_ends.push(subtree_end);
            active = Some(id);
        }
        let ledger_title = if ledger {
            match reader.word()? {
                0 => None,
                3 => Some([reader.text()?, reader.text()?, reader.text()?]),
                _ => return Err("invalid Ledger headline row".into()),
            }
        } else {
            None
        };
        if !reader.bytes.is_empty() {
            return Err("trailing Org document columns".into());
        }
        Ok(Self {
            syntax,
            records,
            subtree_ends,
            ledger_title,
        })
    }
}
