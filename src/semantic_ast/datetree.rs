//! Conservative Org datetree shape recognition.

use super::{AgendaDate, DateTreeEntry, Document, ParsedAnnotation, Section, SectionIndexSource};

impl Document<ParsedAnnotation> {
    /// Projects date-organized capture trees into searchable date paths.
    pub fn datetree_entries(&self) -> Vec<DateTreeEntry> {
        self.sections
            .iter()
            .flat_map(|section| datetree_entries_in_section(section, Vec::new()))
            .collect()
    }
}

fn datetree_entries_in_section(
    section: &Section<ParsedAnnotation>,
    parent_path: Vec<String>,
) -> Vec<DateTreeEntry> {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_path;
    outline_path.push(title);
    let current = datetree_entry(section, &outline_path);
    current
        .into_iter()
        .chain(
            section.subsections.iter().flat_map(|subsection| {
                datetree_entries_in_section(subsection, outline_path.clone())
            }),
        )
        .collect()
}

fn datetree_entry(
    section: &Section<ParsedAnnotation>,
    outline_path: &[String],
) -> Option<DateTreeEntry> {
    if outline_path.len() < 3 {
        return None;
    }
    let day_title = outline_path.last()?;
    let month_title = outline_path.get(outline_path.len() - 2)?;
    let year_title = outline_path.get(outline_path.len() - 3)?;
    let date = parse_day_title(day_title)?;
    if parse_year_title(year_title) != Some(date.year) {
        return None;
    }
    if parse_month_title(month_title) != Some((date.year, date.month)) {
        return None;
    }
    Some(DateTreeEntry {
        source: SectionIndexSource::from_annotation(&section.ann),
        date,
        year_title: year_title.clone(),
        month_title: month_title.clone(),
        day_title: day_title.clone(),
        outline_path: outline_path.to_vec(),
    })
}

fn parse_year_title(title: &str) -> Option<u16> {
    title.get(..4)?.parse().ok()
}

fn parse_month_title(title: &str) -> Option<(u16, u8)> {
    let date = title.get(..7)?;
    let (year, month) = date.split_once('-')?;
    Some((year.parse().ok()?, month.parse().ok()?))
}

fn parse_day_title(title: &str) -> Option<AgendaDate> {
    let date = title.get(..10)?;
    let mut parts = date.split('-');
    Some(AgendaDate::new(
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}
