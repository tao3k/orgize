use super::{AstMut, AstRef, BlockLine, Link, SemanticFixedWidth};

impl<A> SemanticFixedWidth<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> SemanticFixedWidth<B>
    where
        F: FnMut(&A) -> B,
    {
        SemanticFixedWidth {
            value: self.value.clone(),
            lines: self.lines.iter().map(|line| line.map_ann_with(f)).collect(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<SemanticFixedWidth<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(SemanticFixedWidth {
            value: self.value.clone(),
            lines: self
                .lines
                .iter()
                .map(|line| line.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for line in &self.lines {
            line.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for line in &mut self.lines {
            line.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for line in &self.lines {
            acc = line.fold_with(acc, f);
        }
        acc
    }
}

impl<A> BlockLine<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> BlockLine<B>
    where
        F: FnMut(&A) -> B,
    {
        BlockLine {
            ann: f(&self.ann),
            number: self.number,
            source: self.source.clone(),
            value: self.value.clone(),
            normalized_value: self.normalized_value.clone(),
            value_without_code_ref: self.value_without_code_ref.clone(),
            normalized_value_without_code_ref: self.normalized_value_without_code_ref.clone(),
            removed_indent: self.removed_indent,
            line_ending: self.line_ending.clone(),
            code_ref: self.code_ref.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<BlockLine<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(BlockLine {
            ann: f(&self.ann)?,
            number: self.number,
            source: self.source.clone(),
            value: self.value.clone(),
            normalized_value: self.normalized_value.clone(),
            value_without_code_ref: self.value_without_code_ref.clone(),
            normalized_value_without_code_ref: self.normalized_value_without_code_ref.clone(),
            removed_indent: self.removed_indent,
            line_ending: self.line_ending.clone(),
            code_ref: self.code_ref.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::BlockLine(self));
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::BlockLine(self));
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::BlockLine(self))
    }
}

impl<A> Link<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Link<B>
    where
        F: FnMut(&A) -> B,
    {
        Link {
            path: self.path.clone(),
            target: self.target.clone(),
            description: self.description.iter().map(|x| x.map_ann_with(f)).collect(),
            default_description: self
                .default_description
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            raw_description: self.raw_description.clone(),
            description_state: self.description_state,
            media_kind: self.media_kind,
            caption: self.caption.as_ref().map(|caption| caption.map_ann_with(f)),
            search: self.search.clone(),
            attachment: self.attachment.clone(),
            file: self.file.clone(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Link<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Link {
            path: self.path.clone(),
            target: self.target.clone(),
            description: self
                .description
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            default_description: self
                .default_description
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            raw_description: self.raw_description.clone(),
            description_state: self.description_state,
            media_kind: self.media_kind,
            caption: self
                .caption
                .as_ref()
                .map(|caption| caption.try_map_ann_with(f))
                .transpose()?,
            search: self.search.clone(),
            attachment: self.attachment.clone(),
            file: self.file.clone(),
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        if let Some(caption) = &self.caption {
            caption.visit_with(f);
        }
        for object in &self.description {
            object.visit_with(f);
        }
        for object in &self.default_description {
            object.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        if let Some(caption) = &mut self.caption {
            caption.visit_mut_with(f);
        }
        for object in &mut self.description {
            object.visit_mut_with(f);
        }
        for object in &mut self.default_description {
            object.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = init;
        if let Some(caption) = &self.caption {
            acc = caption.fold_with(acc, f);
        }
        for object in &self.description {
            acc = object.fold_with(acc, f);
        }
        for object in &self.default_description {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}
