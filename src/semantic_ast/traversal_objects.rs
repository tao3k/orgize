use super::{AstMut, AstRef, Citation, CiteReference, Object, ObjectData};

impl<A> Object<A> {
    pub(crate) fn map_ann_with<B, F>(&self, f: &mut F) -> Object<B>
    where
        F: FnMut(&A) -> B,
    {
        Object {
            ann: f(&self.ann),
            data: self.data.map_ann_with(f),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Object<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Object {
            ann: f(&self.ann)?,
            data: self.data.try_map_ann_with(f)?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Object(self));
        self.data.visit_with(f);
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Object(self));
        self.data.visit_mut_with(f);
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let acc = f(init, AstRef::Object(self));
        self.data.fold_with(acc, f)
    }
}

impl<A> ObjectData<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> ObjectData<B>
    where
        F: FnMut(&A) -> B,
    {
        match self {
            ObjectData::Plain(value) => ObjectData::Plain(value.clone()),
            ObjectData::LineBreak => ObjectData::LineBreak,
            ObjectData::Markup { kind, children } => ObjectData::Markup {
                kind: *kind,
                children: children.iter().map(|x| x.map_ann_with(f)).collect(),
            },
            ObjectData::Code(value) => ObjectData::Code(value.clone()),
            ObjectData::Verbatim(value) => ObjectData::Verbatim(value.clone()),
            ObjectData::Timestamp(value) => ObjectData::Timestamp(value.clone()),
            ObjectData::Entity(value) => ObjectData::Entity(value.clone()),
            ObjectData::LatexFragment(value) => ObjectData::LatexFragment(value.clone()),
            ObjectData::ExportSnippet { backend, value } => ObjectData::ExportSnippet {
                backend: backend.clone(),
                value: value.clone(),
            },
            ObjectData::FootnoteRef {
                label,
                resolved_label,
                definition,
            } => ObjectData::FootnoteRef {
                label: label.clone(),
                resolved_label: resolved_label.clone(),
                definition: definition.iter().map(|x| x.map_ann_with(f)).collect(),
            },
            ObjectData::Citation(citation) => ObjectData::Citation(citation.map_ann_with(f)),
            ObjectData::Cloze {
                text,
                raw_text,
                hint,
                id,
                raw,
            } => ObjectData::Cloze {
                text: text.iter().map(|x| x.map_ann_with(f)).collect(),
                raw_text: raw_text.clone(),
                hint: hint.clone(),
                id: id.clone(),
                raw: raw.clone(),
            },
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                raw,
            } => ObjectData::InlineCall {
                name: name.clone(),
                arguments: arguments.clone(),
                header: header.clone(),
                end_header: end_header.clone(),
                raw: raw.clone(),
            },
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                raw,
            } => ObjectData::InlineSrc {
                language: language.clone(),
                parameters: parameters.clone(),
                value: value.clone(),
                raw: raw.clone(),
            },
            ObjectData::Link(link) => ObjectData::Link(Box::new(link.map_ann_with(f))),
            ObjectData::Target(value) => ObjectData::Target(value.clone()),
            ObjectData::RadioTarget(value) => ObjectData::RadioTarget(value.clone()),
            ObjectData::Macro { name, arguments } => ObjectData::Macro {
                name: name.clone(),
                arguments: arguments.clone(),
            },
            ObjectData::StatisticCookie(value) => ObjectData::StatisticCookie(value.clone()),
            ObjectData::Unknown { kind, raw } => ObjectData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ObjectData<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(match self {
            ObjectData::Plain(value) => ObjectData::Plain(value.clone()),
            ObjectData::LineBreak => ObjectData::LineBreak,
            ObjectData::Markup { kind, children } => ObjectData::Markup {
                kind: *kind,
                children: children
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            },
            ObjectData::Code(value) => ObjectData::Code(value.clone()),
            ObjectData::Verbatim(value) => ObjectData::Verbatim(value.clone()),
            ObjectData::Timestamp(value) => ObjectData::Timestamp(value.clone()),
            ObjectData::Entity(value) => ObjectData::Entity(value.clone()),
            ObjectData::LatexFragment(value) => ObjectData::LatexFragment(value.clone()),
            ObjectData::ExportSnippet { backend, value } => ObjectData::ExportSnippet {
                backend: backend.clone(),
                value: value.clone(),
            },
            ObjectData::FootnoteRef {
                label,
                resolved_label,
                definition,
            } => ObjectData::FootnoteRef {
                label: label.clone(),
                resolved_label: resolved_label.clone(),
                definition: definition
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            },
            ObjectData::Citation(citation) => ObjectData::Citation(citation.try_map_ann_with(f)?),
            ObjectData::Cloze {
                text,
                raw_text,
                hint,
                id,
                raw,
            } => ObjectData::Cloze {
                text: text
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
                raw_text: raw_text.clone(),
                hint: hint.clone(),
                id: id.clone(),
                raw: raw.clone(),
            },
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                raw,
            } => ObjectData::InlineCall {
                name: name.clone(),
                arguments: arguments.clone(),
                header: header.clone(),
                end_header: end_header.clone(),
                raw: raw.clone(),
            },
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                raw,
            } => ObjectData::InlineSrc {
                language: language.clone(),
                parameters: parameters.clone(),
                value: value.clone(),
                raw: raw.clone(),
            },
            ObjectData::Link(link) => ObjectData::Link(Box::new(link.try_map_ann_with(f)?)),
            ObjectData::Target(value) => ObjectData::Target(value.clone()),
            ObjectData::RadioTarget(value) => ObjectData::RadioTarget(value.clone()),
            ObjectData::Macro { name, arguments } => ObjectData::Macro {
                name: name.clone(),
                arguments: arguments.clone(),
            },
            ObjectData::StatisticCookie(value) => ObjectData::StatisticCookie(value.clone()),
            ObjectData::Unknown { kind, raw } => ObjectData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        match self {
            ObjectData::Markup { children, .. }
            | ObjectData::FootnoteRef {
                definition: children,
                ..
            } => {
                for child in children {
                    child.visit_with(f);
                }
            }
            ObjectData::Link(link) => link.visit_with(f),
            ObjectData::Citation(citation) => citation.visit_with(f),
            ObjectData::Cloze { text, .. } => {
                for child in text {
                    child.visit_with(f);
                }
            }
            _ => {}
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        match self {
            ObjectData::Markup { children, .. }
            | ObjectData::FootnoteRef {
                definition: children,
                ..
            } => {
                for child in children {
                    child.visit_mut_with(f);
                }
            }
            ObjectData::Link(link) => link.visit_mut_with(f),
            ObjectData::Citation(citation) => citation.visit_mut_with(f),
            ObjectData::Cloze { text, .. } => {
                for child in text {
                    child.visit_mut_with(f);
                }
            }
            _ => {}
        }
    }

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        match self {
            ObjectData::Markup { children, .. }
            | ObjectData::FootnoteRef {
                definition: children,
                ..
            } => {
                for child in children {
                    acc = child.fold_with(acc, f);
                }
            }
            ObjectData::Link(link) => {
                acc = link.fold_with(acc, f);
            }
            ObjectData::Citation(citation) => {
                acc = citation.fold_with(acc, f);
            }
            ObjectData::Cloze { text, .. } => {
                for child in text {
                    acc = child.fold_with(acc, f);
                }
            }
            _ => {}
        }
        acc
    }
}

impl<A> Citation<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Citation<B>
    where
        F: FnMut(&A) -> B,
    {
        Citation {
            style: self.style.clone(),
            variant: self.variant.clone(),
            prefix: self.prefix.iter().map(|x| x.map_ann_with(f)).collect(),
            suffix: self.suffix.iter().map(|x| x.map_ann_with(f)).collect(),
            references: self
                .references
                .iter()
                .map(|x| CiteReference {
                    ann: f(&x.ann),
                    id: x.id.clone(),
                    prefix: x.prefix.iter().map(|o| o.map_ann_with(f)).collect(),
                    suffix: x.suffix.iter().map(|o| o.map_ann_with(f)).collect(),
                })
                .collect(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Citation<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Citation {
            style: self.style.clone(),
            variant: self.variant.clone(),
            prefix: self
                .prefix
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            suffix: self
                .suffix
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            references: self
                .references
                .iter()
                .map(|x| {
                    Ok(CiteReference {
                        ann: f(&x.ann)?,
                        id: x.id.clone(),
                        prefix: x
                            .prefix
                            .iter()
                            .map(|o| o.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                        suffix: x
                            .suffix
                            .iter()
                            .map(|o| o.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for object in &self.prefix {
            object.visit_with(f);
        }
        for reference in &self.references {
            for object in &reference.prefix {
                object.visit_with(f);
            }
            for object in &reference.suffix {
                object.visit_with(f);
            }
        }
        for object in &self.suffix {
            object.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for object in &mut self.prefix {
            object.visit_mut_with(f);
        }
        for reference in &mut self.references {
            for object in &mut reference.prefix {
                object.visit_mut_with(f);
            }
            for object in &mut reference.suffix {
                object.visit_mut_with(f);
            }
        }
        for object in &mut self.suffix {
            object.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for object in &self.prefix {
            acc = object.fold_with(acc, f);
        }
        for reference in &self.references {
            for object in &reference.prefix {
                acc = object.fold_with(acc, f);
            }
            for object in &reference.suffix {
                acc = object.fold_with(acc, f);
            }
        }
        for object in &self.suffix {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}
