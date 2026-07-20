use super::{
    ArchiveLocation, AstMut, AstRef, AttachmentDirectory, FootnoteDefinition, FootnoteEntry,
    IncludeDirective, Inlinetask, InlinetaskEnd, Keyword, MacroDefinition, Property, Section,
    TargetDefinition,
};

impl<A> IncludeDirective<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> IncludeDirective<B>
    where
        F: FnMut(&A) -> B,
    {
        IncludeDirective {
            ann: f(&self.ann),
            path: self.path.clone(),
            raw_path: self.raw_path.clone(),
            arguments: self.arguments.clone(),
            options: self.options.clone(),
            raw_value: self.raw_value.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<IncludeDirective<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(IncludeDirective {
            ann: f(&self.ann)?,
            path: self.path.clone(),
            raw_path: self.raw_path.clone(),
            arguments: self.arguments.clone(),
            options: self.options.clone(),
            raw_value: self.raw_value.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::IncludeDirective(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::IncludeDirective(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::IncludeDirective(self))
    }
}

impl<A> ArchiveLocation<A> {
    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::ArchiveLocation(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::ArchiveLocation(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::ArchiveLocation(self))
    }
}

impl<A> AttachmentDirectory<A> {
    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::AttachmentDirectory(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::AttachmentDirectory(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::AttachmentDirectory(self))
    }
}

impl<A> MacroDefinition<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> MacroDefinition<B>
    where
        F: FnMut(&A) -> B,
    {
        MacroDefinition {
            ann: f(&self.ann),
            name: self.name.clone(),
            template: self.template.clone(),
            raw_value: self.raw_value.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<MacroDefinition<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(MacroDefinition {
            ann: f(&self.ann)?,
            name: self.name.clone(),
            template: self.template.clone(),
            raw_value: self.raw_value.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::MacroDefinition(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::MacroDefinition(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::MacroDefinition(self))
    }
}

impl<A> TargetDefinition<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> TargetDefinition<B>
    where
        F: FnMut(&A) -> B,
    {
        TargetDefinition {
            ann: f(&self.ann),
            kind: self.kind,
            key: self.key.clone(),
            value: self.value.clone(),
            raw: self.raw.clone(),
            alias: self.alias.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<TargetDefinition<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(TargetDefinition {
            ann: f(&self.ann)?,
            kind: self.kind,
            key: self.key.clone(),
            value: self.value.clone(),
            raw: self.raw.clone(),
            alias: self
                .alias
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TargetDefinition(self));
        for object in &self.alias {
            object.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TargetDefinition(self));
        for object in &mut self.alias {
            object.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::TargetDefinition(self));
        for object in &self.alias {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}

impl<A> FootnoteEntry<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> FootnoteEntry<B>
    where
        F: FnMut(&A) -> B,
    {
        FootnoteEntry {
            ann: f(&self.ann),
            label: self.label.clone(),
            definition: self.definition.map_ann_with(f),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<FootnoteEntry<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(FootnoteEntry {
            ann: f(&self.ann)?,
            label: self.label.clone(),
            definition: self.definition.try_map_ann_with(f)?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::FootnoteEntry(self));
        self.definition.visit_with(f);
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::FootnoteEntry(self));
        self.definition.visit_mut_with(f);
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        self.definition
            .fold_with(f(init, AstRef::FootnoteEntry(self)), f)
    }
}

impl<A> FootnoteDefinition<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> FootnoteDefinition<B>
    where
        F: FnMut(&A) -> B,
    {
        match self {
            FootnoteDefinition::Standalone(elements) => {
                FootnoteDefinition::Standalone(elements.iter().map(|x| x.map_ann_with(f)).collect())
            }
            FootnoteDefinition::Inline(objects) => {
                FootnoteDefinition::Inline(objects.iter().map(|x| x.map_ann_with(f)).collect())
            }
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<FootnoteDefinition<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(match self {
            FootnoteDefinition::Standalone(elements) => FootnoteDefinition::Standalone(
                elements
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            ),
            FootnoteDefinition::Inline(objects) => FootnoteDefinition::Inline(
                objects
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        match self {
            FootnoteDefinition::Standalone(elements) => {
                for element in elements {
                    element.visit_with(f);
                }
            }
            FootnoteDefinition::Inline(objects) => {
                for object in objects {
                    object.visit_with(f);
                }
            }
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        match self {
            FootnoteDefinition::Standalone(elements) => {
                for element in elements {
                    element.visit_mut_with(f);
                }
            }
            FootnoteDefinition::Inline(objects) => {
                for object in objects {
                    object.visit_mut_with(f);
                }
            }
        }
    }

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        match self {
            FootnoteDefinition::Standalone(elements) => {
                for element in elements {
                    acc = element.fold_with(acc, f);
                }
            }
            FootnoteDefinition::Inline(objects) => {
                for object in objects {
                    acc = object.fold_with(acc, f);
                }
            }
        }
        acc
    }
}

impl<A> Section<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Section<B>
    where
        F: FnMut(&A) -> B,
    {
        Section {
            ann: f(&self.ann),
            body_ann: self.body_ann.as_ref().map(&mut *f),
            level: self.level,
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            effective_properties: self
                .effective_properties
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            archive: self.archive.map_ann_with(f),
            attachment: self.attachment.map_ann_with(f),
            todo: self.todo.clone(),
            is_comment: self.is_comment,
            priority: self.priority.clone(),
            title: self.title.iter().map(|x| x.map_ann_with(f)).collect(),
            raw_title: self.raw_title.clone(),
            anchor: self.anchor.clone(),
            tags: self.tags.clone(),
            effective_tags: self.effective_tags.clone(),
            planning: self.planning.clone(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            subsections: self.subsections.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Section<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Section {
            ann: f(&self.ann)?,
            body_ann: self.body_ann.as_ref().map(&mut *f).transpose()?,
            level: self.level,
            properties: self
                .properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            effective_properties: self
                .effective_properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            archive: self.archive.try_map_ann_with(f)?,
            attachment: self.attachment.try_map_ann_with(f)?,
            todo: self.todo.clone(),
            is_comment: self.is_comment,
            priority: self.priority.clone(),
            title: self
                .title
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            raw_title: self.raw_title.clone(),
            anchor: self.anchor.clone(),
            tags: self.tags.clone(),
            effective_tags: self.effective_tags.clone(),
            planning: self.planning.clone(),
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            subsections: self
                .subsections
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Section(self));
        for property in &self.properties {
            property.visit_with(f);
        }
        if let Some(directory) = &self.attachment.directory {
            directory.visit_with(f);
        }
        for object in &self.title {
            object.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
        for section in &self.subsections {
            section.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Section(self));
        for property in &mut self.properties {
            property.visit_mut_with(f);
        }
        if let Some(directory) = &mut self.attachment.directory {
            directory.visit_mut_with(f);
        }
        for object in &mut self.title {
            object.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
        for section in &mut self.subsections {
            section.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Section(self));
        for property in &self.properties {
            acc = property.fold_with(acc, f);
        }
        if let Some(directory) = &self.attachment.directory {
            acc = directory.fold_with(acc, f);
        }
        for object in &self.title {
            acc = object.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        for section in &self.subsections {
            acc = section.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Inlinetask<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Inlinetask<B>
    where
        F: FnMut(&A) -> B,
    {
        Inlinetask {
            level: self.level,
            todo: self.todo.clone(),
            priority: self.priority.clone(),
            title: self.title.iter().map(|x| x.map_ann_with(f)).collect(),
            raw_title: self.raw_title.clone(),
            tags: self.tags.clone(),
            planning: self.planning.clone(),
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            end: self.end.as_ref().map(|x| x.map_ann_with(f)),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Inlinetask<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Inlinetask {
            level: self.level,
            todo: self.todo.clone(),
            priority: self.priority.clone(),
            title: self
                .title
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            raw_title: self.raw_title.clone(),
            tags: self.tags.clone(),
            planning: self.planning.clone(),
            properties: self
                .properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            end: self
                .end
                .as_ref()
                .map(|x| x.try_map_ann_with(f))
                .transpose()?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Inlinetask(self));
        for object in &self.title {
            object.visit_with(f);
        }
        for property in &self.properties {
            property.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
        if let Some(end) = &self.end {
            end.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Inlinetask(self));
        for object in &mut self.title {
            object.visit_mut_with(f);
        }
        for property in &mut self.properties {
            property.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
        if let Some(end) = &mut self.end {
            end.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Inlinetask(self));
        for object in &self.title {
            acc = object.fold_with(acc, f);
        }
        for property in &self.properties {
            acc = property.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        if let Some(end) = &self.end {
            acc = end.fold_with(acc, f);
        }
        acc
    }
}

impl<A> InlinetaskEnd<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> InlinetaskEnd<B>
    where
        F: FnMut(&A) -> B,
    {
        InlinetaskEnd {
            ann: f(&self.ann),
            level: self.level,
            raw: self.raw.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<InlinetaskEnd<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(InlinetaskEnd {
            ann: f(&self.ann)?,
            level: self.level,
            raw: self.raw.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::InlinetaskEnd(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::InlinetaskEnd(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::InlinetaskEnd(self))
    }
}

impl<A> Property<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Property<B>
    where
        F: FnMut(&A) -> B,
    {
        Property {
            ann: f(&self.ann),
            key: self.key.clone(),
            value: self.value.clone(),
            duration: self.duration.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Property<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Property {
            ann: f(&self.ann)?,
            key: self.key.clone(),
            value: self.value.clone(),
            duration: self.duration.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Property(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Property(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::Property(self))
    }
}

impl<A> Keyword<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Keyword<B>
    where
        F: FnMut(&A) -> B,
    {
        Keyword {
            ann: f(&self.ann),
            key: self.key.clone(),
            optional: self.optional.clone(),
            value: self.value.clone(),
            parsed: self.parsed.iter().map(|x| x.map_ann_with(f)).collect(),
            attributes: self.attributes.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Keyword<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Keyword {
            ann: f(&self.ann)?,
            key: self.key.clone(),
            optional: self.optional.clone(),
            value: self.value.clone(),
            parsed: self
                .parsed
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            attributes: self.attributes.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Keyword(self));
        for object in &self.parsed {
            object.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Keyword(self));
        for object in &mut self.parsed {
            object.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Keyword(self));
        for object in &self.parsed {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}
