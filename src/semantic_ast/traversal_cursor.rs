use super::{AstMut, AstRef, BareAst, Document};

impl<A> Document<A> {
    pub fn to_bare(&self) -> BareAst {
        self.map_ann(|_| ())
    }

    pub fn map_ann<B, F>(&self, mut f: F) -> Document<B>
    where
        F: FnMut(&A) -> B,
    {
        self.map_ann_with(&mut f)
    }

    pub fn try_map_ann<B, E, F>(&self, mut f: F) -> Result<Document<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        self.try_map_ann_with(&mut f)
    }

    pub fn visit<F>(&self, mut f: F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        self.visit_with(&mut f);
    }

    pub fn visit_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        self.visit_mut_with(&mut f);
    }

    pub fn fold<T, F>(&self, init: T, mut f: F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        self.fold_with(init, &mut f)
    }

    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Document<B>
    where
        F: FnMut(&A) -> B,
    {
        Document {
            ann: f(&self.ann),
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            archive_locations: self
                .archive_locations
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            metadata: self.metadata.iter().map(|x| x.map_ann_with(f)).collect(),
            filetags: self.filetags.clone(),
            tag_definitions: self.tag_definitions.clone(),
            export_settings: self.export_settings.clone(),
            link_abbreviations: self.link_abbreviations.clone(),
            includes: self.includes.iter().map(|x| x.map_ann_with(f)).collect(),
            macro_definitions: self
                .macro_definitions
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            targets: self.targets.iter().map(|x| x.map_ann_with(f)).collect(),
            footnotes: self.footnotes.iter().map(|x| x.map_ann_with(f)).collect(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            sections: self.sections.iter().map(|x| x.map_ann_with(f)).collect(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Document<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Document {
            ann: f(&self.ann)?,
            properties: self
                .properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            archive_locations: self
                .archive_locations
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            metadata: self
                .metadata
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            filetags: self.filetags.clone(),
            tag_definitions: self.tag_definitions.clone(),
            export_settings: self.export_settings.clone(),
            link_abbreviations: self.link_abbreviations.clone(),
            includes: self
                .includes
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            macro_definitions: self
                .macro_definitions
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            targets: self
                .targets
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            footnotes: self
                .footnotes
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            sections: self
                .sections
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            diagnostics: self.diagnostics.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Document(self));
        for property in &self.properties {
            property.visit_with(f);
        }
        for archive_location in &self.archive_locations {
            archive_location.visit_with(f);
        }
        for keyword in &self.metadata {
            keyword.visit_with(f);
        }
        for include in &self.includes {
            include.visit_with(f);
        }
        for definition in &self.macro_definitions {
            definition.visit_with(f);
        }
        for target in &self.targets {
            target.visit_with(f);
        }
        for footnote in &self.footnotes {
            footnote.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
        for section in &self.sections {
            section.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Document(self));
        for property in &mut self.properties {
            property.visit_mut_with(f);
        }
        for archive_location in &mut self.archive_locations {
            archive_location.visit_mut_with(f);
        }
        for keyword in &mut self.metadata {
            keyword.visit_mut_with(f);
        }
        for include in &mut self.includes {
            include.visit_mut_with(f);
        }
        for definition in &mut self.macro_definitions {
            definition.visit_mut_with(f);
        }
        for target in &mut self.targets {
            target.visit_mut_with(f);
        }
        for footnote in &mut self.footnotes {
            footnote.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
        for section in &mut self.sections {
            section.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Document(self));
        for property in &self.properties {
            acc = property.fold_with(acc, f);
        }
        for archive_location in &self.archive_locations {
            acc = archive_location.fold_with(acc, f);
        }
        for keyword in &self.metadata {
            acc = keyword.fold_with(acc, f);
        }
        for include in &self.includes {
            acc = include.fold_with(acc, f);
        }
        for definition in &self.macro_definitions {
            acc = definition.fold_with(acc, f);
        }
        for target in &self.targets {
            acc = target.fold_with(acc, f);
        }
        for footnote in &self.footnotes {
            acc = footnote.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        for section in &self.sections {
            acc = section.fold_with(acc, f);
        }
        acc
    }
}
