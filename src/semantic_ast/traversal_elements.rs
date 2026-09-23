use super::{AstMut, AstRef, Drawer, Element, ElementData, FootnoteDef};

impl<A> Element<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Element<B>
    where
        F: FnMut(&A) -> B,
    {
        Element {
            ann: f(&self.ann),
            affiliated_keywords: self
                .affiliated_keywords
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            data: self.data.map_ann_with(f),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Element<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Element {
            ann: f(&self.ann)?,
            affiliated_keywords: self
                .affiliated_keywords
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
            data: self.data.try_map_ann_with(f)?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Element(self));
        for keyword in &self.affiliated_keywords {
            keyword.visit_with(f);
        }
        self.data.visit_with(f);
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Element(self));
        for keyword in &mut self.affiliated_keywords {
            keyword.visit_mut_with(f);
        }
        self.data.visit_mut_with(f);
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Element(self));
        for keyword in &self.affiliated_keywords {
            acc = keyword.fold_with(acc, f);
        }
        self.data.fold_with(acc, f)
    }
}

impl<A> ElementData<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> ElementData<B>
    where
        F: FnMut(&A) -> B,
    {
        match self {
            ElementData::Paragraph(objects) => {
                ElementData::Paragraph(objects.iter().map(|x| x.map_ann_with(f)).collect())
            }
            ElementData::Keyword(keyword) => ElementData::Keyword(keyword.map_ann_with(f)),
            ElementData::BabelCall(keyword) => ElementData::BabelCall(keyword.map_ann_with(f)),
            ElementData::Clock(clock) => ElementData::Clock(clock.clone()),
            ElementData::Drawer(drawer) => ElementData::Drawer(Drawer {
                name: drawer.name.clone(),
                children: drawer.children.iter().map(|x| x.map_ann_with(f)).collect(),
                raw: drawer.raw.clone(),
            }),
            ElementData::PropertyDrawer(properties) => {
                ElementData::PropertyDrawer(properties.iter().map(|x| x.map_ann_with(f)).collect())
            }
            ElementData::List(list) => ElementData::List(list.map_ann_with(f)),
            ElementData::Table(table) => ElementData::Table(table.map_ann_with(f)),
            ElementData::TableEl { raw } => ElementData::TableEl { raw: raw.clone() },
            ElementData::Block(block) => ElementData::Block(block.map_ann_with(f)),
            ElementData::FootnoteDef(def) => ElementData::FootnoteDef(FootnoteDef {
                label: def.label.clone(),
                children: def.children.iter().map(|x| x.map_ann_with(f)).collect(),
            }),
            ElementData::Inlinetask(inlinetask) => {
                ElementData::Inlinetask(Box::new(inlinetask.map_ann_with(f)))
            }
            ElementData::Comment(value) => ElementData::Comment(value.clone()),
            ElementData::DiarySexp(value) => ElementData::DiarySexp(value.clone()),
            ElementData::FixedWidth(fixed_width) => {
                ElementData::FixedWidth(fixed_width.map_ann_with(f))
            }
            ElementData::Rule => ElementData::Rule,
            ElementData::LatexEnvironment(value) => ElementData::LatexEnvironment(value.clone()),
            ElementData::Unknown { kind, raw } => ElementData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ElementData<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(match self {
            ElementData::Paragraph(objects) => ElementData::Paragraph(
                objects
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            ),
            ElementData::Keyword(keyword) => ElementData::Keyword(keyword.try_map_ann_with(f)?),
            ElementData::BabelCall(keyword) => ElementData::BabelCall(keyword.try_map_ann_with(f)?),
            ElementData::Clock(clock) => ElementData::Clock(clock.clone()),
            ElementData::Drawer(drawer) => ElementData::Drawer(Drawer {
                name: drawer.name.clone(),
                children: drawer
                    .children
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
                raw: drawer.raw.clone(),
            }),
            ElementData::PropertyDrawer(properties) => ElementData::PropertyDrawer(
                properties
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            ),
            ElementData::List(list) => ElementData::List(list.try_map_ann_with(f)?),
            ElementData::Table(table) => ElementData::Table(table.try_map_ann_with(f)?),
            ElementData::TableEl { raw } => ElementData::TableEl { raw: raw.clone() },
            ElementData::Block(block) => ElementData::Block(block.try_map_ann_with(f)?),
            ElementData::FootnoteDef(def) => ElementData::FootnoteDef(FootnoteDef {
                label: def.label.clone(),
                children: def
                    .children
                    .iter()
                    .map(|x| x.try_map_ann_with(f))
                    .collect::<Result<_, _>>()?,
            }),
            ElementData::Inlinetask(inlinetask) => {
                ElementData::Inlinetask(Box::new(inlinetask.try_map_ann_with(f)?))
            }
            ElementData::Comment(value) => ElementData::Comment(value.clone()),
            ElementData::DiarySexp(value) => ElementData::DiarySexp(value.clone()),
            ElementData::FixedWidth(fixed_width) => {
                ElementData::FixedWidth(fixed_width.try_map_ann_with(f)?)
            }
            ElementData::Rule => ElementData::Rule,
            ElementData::LatexEnvironment(value) => ElementData::LatexEnvironment(value.clone()),
            ElementData::Unknown { kind, raw } => ElementData::Unknown {
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
            ElementData::Paragraph(objects) => {
                for object in objects {
                    object.visit_with(f);
                }
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                keyword.visit_with(f);
            }
            ElementData::Drawer(drawer) => {
                for child in &drawer.children {
                    child.visit_with(f);
                }
            }
            ElementData::PropertyDrawer(properties) => {
                for property in properties {
                    property.visit_with(f);
                }
            }
            ElementData::List(list) => {
                list.visit_with(f);
            }
            ElementData::Table(table) => {
                table.visit_with(f);
            }
            ElementData::Block(block) => {
                for line in &block.lines {
                    line.visit_with(f);
                }
                for child in &block.children {
                    child.visit_with(f);
                }
            }
            ElementData::FixedWidth(fixed_width) => fixed_width.visit_with(f),
            ElementData::FootnoteDef(def) => {
                for child in &def.children {
                    child.visit_with(f);
                }
            }
            ElementData::Inlinetask(inlinetask) => {
                inlinetask.visit_with(f);
            }
            _ => {}
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        match self {
            ElementData::Paragraph(objects) => {
                for object in objects {
                    object.visit_mut_with(f);
                }
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                keyword.visit_mut_with(f);
            }
            ElementData::Drawer(drawer) => {
                for child in &mut drawer.children {
                    child.visit_mut_with(f);
                }
            }
            ElementData::PropertyDrawer(properties) => {
                for property in properties {
                    property.visit_mut_with(f);
                }
            }
            ElementData::List(list) => {
                list.visit_mut_with(f);
            }
            ElementData::Table(table) => {
                table.visit_mut_with(f);
            }
            ElementData::Block(block) => {
                for line in &mut block.lines {
                    line.visit_mut_with(f);
                }
                for child in &mut block.children {
                    child.visit_mut_with(f);
                }
            }
            ElementData::FixedWidth(fixed_width) => fixed_width.visit_mut_with(f),
            ElementData::FootnoteDef(def) => {
                for child in &mut def.children {
                    child.visit_mut_with(f);
                }
            }
            ElementData::Inlinetask(inlinetask) => {
                inlinetask.visit_mut_with(f);
            }
            _ => {}
        }
    }

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        match self {
            ElementData::Paragraph(objects) => {
                for object in objects {
                    acc = object.fold_with(acc, f);
                }
            }
            ElementData::Keyword(keyword) | ElementData::BabelCall(keyword) => {
                acc = keyword.fold_with(acc, f);
            }
            ElementData::Drawer(drawer) => {
                for child in &drawer.children {
                    acc = child.fold_with(acc, f);
                }
            }
            ElementData::PropertyDrawer(properties) => {
                for property in properties {
                    acc = property.fold_with(acc, f);
                }
            }
            ElementData::List(list) => {
                acc = list.fold_with(acc, f);
            }
            ElementData::Table(table) => {
                acc = table.fold_with(acc, f);
            }
            ElementData::Block(block) => {
                for line in &block.lines {
                    acc = line.fold_with(acc, f);
                }
                for child in &block.children {
                    acc = child.fold_with(acc, f);
                }
            }
            ElementData::FixedWidth(fixed_width) => {
                acc = fixed_width.fold_with(acc, f);
            }
            ElementData::FootnoteDef(def) => {
                for child in &def.children {
                    acc = child.fold_with(acc, f);
                }
            }
            ElementData::Inlinetask(inlinetask) => {
                acc = inlinetask.fold_with(acc, f);
            }
            _ => {}
        }
        acc
    }
}
