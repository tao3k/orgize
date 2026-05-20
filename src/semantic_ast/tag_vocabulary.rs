//! Tag vocabulary helpers backed by document-level `#+TAGS:` definitions.

use super::TagDefinition;

/// Case-insensitive tag matcher that understands org-mode group tag expansion.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TagMatcher<'a> {
    definitions: &'a [TagDefinition],
}

impl<'a> TagMatcher<'a> {
    pub(crate) fn new(definitions: &'a [TagDefinition]) -> Self {
        Self { definitions }
    }

    pub(crate) fn has_tag(&self, tags: &[String], needle: &str) -> bool {
        self.matched_tag(tags, needle).is_some()
    }

    pub(crate) fn matched_tag<'b>(&self, tags: &'b [String], needle: &str) -> Option<&'b String> {
        tags.iter().find(|tag| self.tag_matches(tag, needle))
    }

    fn tag_matches(&self, actual: &str, needle: &str) -> bool {
        actual.eq_ignore_ascii_case(needle) || self.group_contains(needle, actual, &mut Vec::new())
    }

    fn group_contains(&self, group: &str, actual: &str, visited: &mut Vec<String>) -> bool {
        if visited
            .iter()
            .any(|visited_group| visited_group.eq_ignore_ascii_case(group))
        {
            return false;
        }
        visited.push(group.to_string());

        self.definitions
            .iter()
            .filter_map(|definition| {
                definition
                    .group
                    .as_ref()
                    .and_then(|definition_group| definition_group.name.as_deref())
                    .filter(|parent| parent.eq_ignore_ascii_case(group))
                    .map(|_| definition.name.as_str())
            })
            .any(|member| {
                member.eq_ignore_ascii_case(actual) || self.group_contains(member, actual, visited)
            })
    }
}
