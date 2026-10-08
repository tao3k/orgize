//! Native headline-time facts projected into semantic agenda rows.
use super::agenda_model::{AgendaQuery, AgendaTime};
use super::model::Section;
#[derive(Clone, Copy)]
pub(crate) struct HeadlineTimeSpec {
    pub(crate) start: AgendaTime,
    pub(crate) end: Option<AgendaTime>,
}
pub(crate) fn headline_time<A>(
    section: &Section<A>,
    query: &AgendaQuery,
) -> Option<HeadlineTimeSpec> {
    if !query.search_headline_time {
        return None;
    }
    let row = super::org_native_values::optional("headline-time", &section.raw_title)?;
    assert!(row.len() == 2 || row.len() == 4, "native time arity");
    let time = |fields: &[String]| AgendaTime {
        hour: fields[0].parse().expect("native time hour"),
        minute: fields[1].parse().expect("native time minute"),
    };
    Some(HeadlineTimeSpec {
        start: time(&row[..2]),
        end: (row.len() == 4).then(|| time(&row[2..])),
    })
}
