//! Call-local admission of Scheme-owned keyword facts, bound to graph IDs.
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct KeywordPlans(HashMap<usize, Vec<Vec<String>>>);

impl KeywordPlans {
    pub(super) fn admit(ids: &[usize], rows: Vec<Vec<String>>) -> Result<Self, String> {
        let mut rows = rows.into_iter();
        let mut plans = HashMap::with_capacity(ids.len());
        for (index, &id) in ids.iter().enumerate() {
            let header = rows.next().ok_or("missing native keyword header")?;
            if header.len() != 3 || header[0] != "keyword" || header[1] != index.to_string() {
                return Err("invalid native keyword header".into());
            }
            let count = header[2]
                .parse::<usize>()
                .map_err(|_| "invalid native keyword count")?;
            if header[2] != count.to_string() || count > rows.len() {
                return Err("invalid native keyword framing".into());
            }
            let facts: Vec<_> = rows.by_ref().take(count).collect();
            if facts.iter().any(|row| {
                row.len() != 2
                    || !matches!(
                        row[0].as_str(),
                        "route" | "first" | "rest" | "word" | "tag" | "H" | "-" | "e"
                    )
            }) || facts.iter().filter(|row| row[0] == "route").count() != 1
                || facts.iter().filter(|row| row[0] == "first").count() != 1
                || facts.iter().filter(|row| row[0] == "rest").count() != 1
                || plans.insert(id, facts).is_some()
            {
                return Err("invalid native keyword facts".into());
            }
        }
        if rows.next().is_some() {
            return Err("trailing native keyword rows".into());
        }
        Ok(Self(plans))
    }

    pub(super) fn get(&self, id: usize) -> Option<&[Vec<String>]> {
        self.0.get(&id).map(Vec::as_slice)
    }
}

#[cfg(test)]
#[path = "../tests/unit/org_aot_keyword_view.rs"]
mod tests;
