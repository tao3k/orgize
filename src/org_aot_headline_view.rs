//! Typed, source-backed headline view over the Scheme-AOT Element graph.
//!
//! This layer does not recognize Org source. It only reads admitted graph
//! records and calls Scheme-authored AOT headline functions.

use gerbil_parser_runtime::TextRange;

use super::OrgAotDocument;

/// A headline in the single Scheme-AOT Org Element graph.
#[derive(Clone, Copy, Debug)]
pub struct OrgHeadline<'a> {
    document: &'a OrgAotDocument,
    id: usize,
}

impl OrgAotDocument {
    /// Iterate typed headlines in source order.
    pub fn headlines(&self) -> impl Iterator<Item = OrgHeadline<'_>> {
        self.records
            .iter()
            .filter(|record| record.kind == "headline")
            .map(|record| OrgHeadline {
                document: self,
                id: record.id,
            })
    }

    /// Get a typed headline by its graph identity.
    #[must_use]
    pub fn headline(&self, id: usize) -> Option<OrgHeadline<'_>> {
        self.records
            .get(id)
            .filter(|record| record.kind == "headline")
            .map(|_| OrgHeadline { document: self, id })
    }
}

impl OrgHeadline<'_> {
    fn record(&self) -> &gerbil_parser_runtime::GraphRecord {
        &self.document.records[self.id]
    }

    /// Stable graph identity for queries and source edits.
    #[must_use]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Full source-backed subtree range.
    #[must_use]
    pub fn range(&self) -> TextRange {
        self.record().range
    }

    /// Number of source headline markers, as projected by Scheme.
    #[must_use]
    pub fn level(&self) -> usize {
        self.record().field("markers").map_or(0, str::len)
    }

    /// Display title after Scheme-owned TODO, priority and tag projection.
    #[must_use]
    pub fn display_title(&self) -> Option<String> {
        self.document.headline_display_title(self.id)
    }

    /// File-local TODO keyword recognized by the Scheme-AOT algorithm.
    #[must_use]
    pub fn todo_keyword(&self) -> Option<String> {
        self.document.headline_todo_keyword(self.id)
    }

    /// File-local TODO state class recognized by the Scheme-AOT algorithm.
    #[must_use]
    pub fn todo_type(&self) -> Option<&'static str> {
        self.document.headline_todo_type(self.id)
    }

    /// Whether Org's case-sensitive COMMENT marker is present.
    #[must_use]
    pub fn is_comment(&self) -> bool {
        self.document.headline_is_comment(self.id) == Some(true)
    }

    /// This headline's own Scheme-projected tags.
    pub fn local_tags(&self) -> impl Iterator<Item = &str> {
        self.record().values("tag")
    }

    /// Tags inherited from ancestor headlines, in outer-to-inner order.
    #[must_use]
    pub fn effective_tags(&self) -> Vec<String> {
        let mut lineage = Vec::new();
        let mut cursor = Some(self.id);
        while let Some(record) = cursor.and_then(|id| self.document.records.get(id)) {
            if record.kind == "headline" {
                lineage.push(record);
            }
            cursor = record.parent_id;
        }
        let mut tags = Vec::new();
        for headline in lineage.into_iter().rev() {
            for tag in headline.values("tag") {
                if !tags.iter().any(|seen| seen == tag) {
                    tags.push(tag.to_owned());
                }
            }
        }
        tags
    }

    /// Nearest enclosing headline, if any.
    #[must_use]
    pub fn parent(&self) -> Option<OrgHeadline<'_>> {
        let mut cursor = self.record().parent_id;
        while let Some(record) = cursor.and_then(|id| self.document.records.get(id)) {
            if record.kind == "headline" {
                return self.document.headline(record.id);
            }
            cursor = record.parent_id;
        }
        None
    }

    /// Planning key/value fields admitted under this headline.
    #[must_use]
    pub fn planning(&self) -> Vec<(&str, &str)> {
        self.record()
            .child_ids
            .iter()
            .filter_map(|id| self.document.records.get(*id))
            .filter(|record| record.kind == "planning")
            .filter_map(|record| Some((record.field("key")?, record.field("value")?)))
            .collect()
    }

    /// Node properties admitted under this headline's property drawers.
    #[must_use]
    pub fn properties(&self) -> Vec<(&str, &str)> {
        self.record()
            .child_ids
            .iter()
            .filter_map(|id| self.document.records.get(*id))
            .filter(|record| record.kind == "property-drawer")
            .flat_map(|drawer| drawer.child_ids.iter())
            .filter_map(|id| self.document.records.get(*id))
            .filter(|record| record.kind == "node-property")
            .filter_map(|record| Some((record.field("key")?, record.field("value")?)))
            .collect()
    }
}
