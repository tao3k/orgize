//! Task blocker evidence over parsed Org sections.

use super::{
    Document, ParsedAnnotation, Property, Section, SectionIndexSource, TaskBlockerKind,
    TaskBlockerParent, TaskBlockerRecord, TaskBlockerTask, TodoState,
};
use std::collections::HashMap;

impl Document<ParsedAnnotation> {
    /// Projects source-grounded task blocker edges for agent planning consumers.
    ///
    /// This currently models Org's local `ORDERED` property on a parent entry:
    /// direct open TODO children after an earlier open TODO sibling receive a
    /// previous-sibling blocker edge. It does not mutate TODO state or invent
    /// custom dependency syntax.
    pub fn task_blocker_records(&self) -> Vec<TaskBlockerRecord> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_task_blockers(section, Vec::new(), &mut records);
        }
        records.sort_by_key(|record| {
            (
                record.blocked.source.range_start,
                record.blocker.source.range_start,
            )
        });
        records
    }
}

fn collect_task_blockers(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    records: &mut Vec<TaskBlockerRecord>,
) {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path;
    outline_path.push(title.clone());

    collect_ordered_child_blockers(section, &outline_path, records);

    for child in &section.subsections {
        collect_task_blockers(child, outline_path.clone(), records);
    }
}

fn collect_ordered_child_blockers(
    parent: &Section<ParsedAnnotation>,
    parent_outline_path: &[String],
    records: &mut Vec<TaskBlockerRecord>,
) {
    let Some(ordered_property) = local_ordered_property(parent) else {
        return;
    };

    let parent_record = TaskBlockerParent {
        source: SectionIndexSource::from_annotation(&parent.ann),
        ordered_property_source: SectionIndexSource::from_annotation(&ordered_property.ann),
        outline_path: parent_outline_path.to_vec(),
        level: parent.level,
        title: parent.raw_title.trim_end().to_string(),
    };

    let mut previous_open_sibling: Option<TaskBlockerTask> = None;
    for child in &parent.subsections {
        if !is_open_todo_section(child) {
            continue;
        }

        let child_task = task_from_section(child, parent_outline_path);
        if let Some(blocker) = previous_open_sibling.clone() {
            records.push(TaskBlockerRecord {
                kind: TaskBlockerKind::OrderedPreviousSibling,
                blocked: child_task.clone(),
                blocker: blocker.clone(),
                parent: parent_record.clone(),
                message: format!(
                    "local ORDERED property on parent '{}' requires '{}' after previous open sibling '{}'",
                    parent_record.title, child_task.title, blocker.title
                ),
            });
        }

        previous_open_sibling = Some(child_task);
    }
}

fn task_from_section(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: &[String],
) -> TaskBlockerTask {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path.to_vec();
    outline_path.push(title.clone());
    TaskBlockerTask {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path,
        level: section.level,
        title,
        todo: section.todo.clone(),
    }
}

fn is_open_todo_section(section: &Section<ParsedAnnotation>) -> bool {
    section
        .todo
        .as_ref()
        .is_some_and(|todo| todo.state == TodoState::Todo)
}

fn local_ordered_property(
    section: &Section<ParsedAnnotation>,
) -> Option<&Property<ParsedAnnotation>> {
    section.properties.iter().find(|property| {
        property.key.eq_ignore_ascii_case("ORDERED") && is_truthy_property_value(&property.value)
    })
}

fn is_truthy_property_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "t" | "true" | "yes" | "1"
    )
}

pub(crate) fn blockers_by_blocked_source(
    blockers: Vec<TaskBlockerRecord>,
) -> HashMap<u32, Vec<TaskBlockerRecord>> {
    let mut by_source: HashMap<u32, Vec<TaskBlockerRecord>> = HashMap::new();
    for blocker in blockers {
        by_source
            .entry(blocker.blocked.source.range_start)
            .or_default()
            .push(blocker);
    }
    by_source
}

pub(crate) fn blockers_for_source(
    blockers_by_source: &HashMap<u32, Vec<TaskBlockerRecord>>,
    source: &SectionIndexSource,
) -> Vec<TaskBlockerRecord> {
    blockers_by_source
        .get(&source.range_start)
        .cloned()
        .unwrap_or_default()
}
