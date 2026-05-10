//! Annotation mapping and traversal for the semantic AST.

use super::{
    AstMut, AstRef, BareAst, Block, Citation, CiteReference, Document, Drawer, Element,
    ElementData, FootnoteDef, IncludeDirective, Keyword, Link, List, ListItem, MacroDefinition,
    Object, ObjectData, Property, Section, Table, TableCell, TableRow,
};

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

    fn map_ann_with<B, F>(&self, f: &mut F) -> Document<B>
    where
        F: FnMut(&A) -> B,
    {
        Document {
            ann: f(&self.ann),
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            includes: self.includes.iter().map(|x| x.map_ann_with(f)).collect(),
            macro_definitions: self
                .macro_definitions
                .iter()
                .map(|x| x.map_ann_with(f))
                .collect(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            sections: self.sections.iter().map(|x| x.map_ann_with(f)).collect(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Document<B>, E>
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

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Document(self));
        for property in &self.properties {
            property.visit_with(f);
        }
        for include in &self.includes {
            include.visit_with(f);
        }
        for definition in &self.macro_definitions {
            definition.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
        for section in &self.sections {
            section.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Document(self));
        for property in &mut self.properties {
            property.visit_mut_with(f);
        }
        for include in &mut self.includes {
            include.visit_mut_with(f);
        }
        for definition in &mut self.macro_definitions {
            definition.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
        for section in &mut self.sections {
            section.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Document(self));
        for property in &self.properties {
            acc = property.fold_with(acc, f);
        }
        for include in &self.includes {
            acc = include.fold_with(acc, f);
        }
        for definition in &self.macro_definitions {
            acc = definition.fold_with(acc, f);
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

impl<A> IncludeDirective<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> IncludeDirective<B>
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

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<IncludeDirective<B>, E>
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

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::IncludeDirective(self));
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::IncludeDirective(self));
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::IncludeDirective(self))
    }
}

impl<A> MacroDefinition<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> MacroDefinition<B>
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

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<MacroDefinition<B>, E>
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

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::MacroDefinition(self));
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::MacroDefinition(self));
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::MacroDefinition(self))
    }
}

impl<A> Section<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Section<B>
    where
        F: FnMut(&A) -> B,
    {
        Section {
            ann: f(&self.ann),
            level: self.level,
            properties: self.properties.iter().map(|x| x.map_ann_with(f)).collect(),
            todo: self.todo.clone(),
            is_comment: self.is_comment,
            priority: self.priority.clone(),
            title: self.title.iter().map(|x| x.map_ann_with(f)).collect(),
            raw_title: self.raw_title.clone(),
            anchor: self.anchor.clone(),
            tags: self.tags.clone(),
            planning: self.planning.clone(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
            subsections: self.subsections.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Section<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Section {
            ann: f(&self.ann)?,
            level: self.level,
            properties: self
                .properties
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
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

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Section(self));
        for property in &self.properties {
            property.visit_with(f);
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

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Section(self));
        for property in &mut self.properties {
            property.visit_mut_with(f);
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

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::Section(self));
        for property in &self.properties {
            acc = property.fold_with(acc, f);
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

impl<A> Property<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Property<B>
    where
        F: FnMut(&A) -> B,
    {
        Property {
            ann: f(&self.ann),
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Property<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Property {
            ann: f(&self.ann)?,
            key: self.key.clone(),
            value: self.value.clone(),
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Property(self));
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Property(self));
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::Property(self))
    }
}

impl<A> Keyword<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Keyword<B>
    where
        F: FnMut(&A) -> B,
    {
        Keyword {
            ann: f(&self.ann),
            key: self.key.clone(),
            optional: self.optional.clone(),
            value: self.value.clone(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Keyword<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Keyword {
            ann: f(&self.ann)?,
            key: self.key.clone(),
            optional: self.optional.clone(),
            value: self.value.clone(),
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Keyword(self));
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Keyword(self));
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        f(init, AstRef::Keyword(self))
    }
}

impl<A> Element<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Element<B>
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

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Element<B>, E>
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

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Element(self));
        for keyword in &self.affiliated_keywords {
            keyword.visit_with(f);
        }
        self.data.visit_with(f);
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Element(self));
        for keyword in &mut self.affiliated_keywords {
            keyword.visit_mut_with(f);
        }
        self.data.visit_mut_with(f);
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
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
    fn map_ann_with<B, F>(&self, f: &mut F) -> ElementData<B>
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
            ElementData::Comment(value) => ElementData::Comment(value.clone()),
            ElementData::FixedWidth(value) => ElementData::FixedWidth(value.clone()),
            ElementData::Rule => ElementData::Rule,
            ElementData::LatexEnvironment(value) => ElementData::LatexEnvironment(value.clone()),
            ElementData::Unknown { kind, raw } => ElementData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ElementData<B>, E>
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
            ElementData::Comment(value) => ElementData::Comment(value.clone()),
            ElementData::FixedWidth(value) => ElementData::FixedWidth(value.clone()),
            ElementData::Rule => ElementData::Rule,
            ElementData::LatexEnvironment(value) => ElementData::LatexEnvironment(value.clone()),
            ElementData::Unknown { kind, raw } => ElementData::Unknown {
                kind: kind.clone(),
                raw: raw.clone(),
            },
        })
    }

    fn visit_with<F>(&self, f: &mut F)
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
                for child in &block.children {
                    child.visit_with(f);
                }
            }
            ElementData::FootnoteDef(def) => {
                for child in &def.children {
                    child.visit_with(f);
                }
            }
            _ => {}
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
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
                for child in &mut block.children {
                    child.visit_mut_with(f);
                }
            }
            ElementData::FootnoteDef(def) => {
                for child in &mut def.children {
                    child.visit_mut_with(f);
                }
            }
            _ => {}
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
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
                for child in &block.children {
                    acc = child.fold_with(acc, f);
                }
            }
            ElementData::FootnoteDef(def) => {
                for child in &def.children {
                    acc = child.fold_with(acc, f);
                }
            }
            _ => {}
        }
        acc
    }
}

impl<A> List<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> List<B>
    where
        F: FnMut(&A) -> B,
    {
        List {
            list_type: self.list_type,
            items: self
                .items
                .iter()
                .map(|item| ListItem {
                    ann: f(&item.ann),
                    bullet: item.bullet.clone(),
                    counter: item.counter.clone(),
                    checkbox: item.checkbox,
                    tag: item.tag.iter().map(|x| x.map_ann_with(f)).collect(),
                    children: item.children.iter().map(|x| x.map_ann_with(f)).collect(),
                })
                .collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<List<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(List {
            list_type: self.list_type,
            items: self
                .items
                .iter()
                .map(|item| {
                    Ok(ListItem {
                        ann: f(&item.ann)?,
                        bullet: item.bullet.clone(),
                        counter: item.counter.clone(),
                        checkbox: item.checkbox,
                        tag: item
                            .tag
                            .iter()
                            .map(|x| x.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                        children: item
                            .children
                            .iter()
                            .map(|x| x.try_map_ann_with(f))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for item in &self.items {
            item.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for item in &mut self.items {
            item.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for item in &self.items {
            acc = item.fold_with(acc, f);
        }
        acc
    }
}

impl<A> ListItem<A> {
    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::ListItem(self));
        for object in &self.tag {
            object.visit_with(f);
        }
        for child in &self.children {
            child.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::ListItem(self));
        for object in &mut self.tag {
            object.visit_mut_with(f);
        }
        for child in &mut self.children {
            child.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::ListItem(self));
        for object in &self.tag {
            acc = object.fold_with(acc, f);
        }
        for child in &self.children {
            acc = child.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Table<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Table<B>
    where
        F: FnMut(&A) -> B,
    {
        Table {
            rows: self
                .rows
                .iter()
                .map(|row| TableRow {
                    ann: f(&row.ann),
                    is_rule: row.is_rule,
                    cells: row
                        .cells
                        .iter()
                        .map(|cell| TableCell {
                            ann: f(&cell.ann),
                            objects: cell.objects.iter().map(|x| x.map_ann_with(f)).collect(),
                        })
                        .collect(),
                })
                .collect(),
            column_alignments: self.column_alignments.clone(),
            formulas: self
                .formulas
                .iter()
                .map(|formula| formula.map_ann_with(f))
                .collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Table<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Table {
            rows: self
                .rows
                .iter()
                .map(|row| {
                    Ok(TableRow {
                        ann: f(&row.ann)?,
                        is_rule: row.is_rule,
                        cells: row
                            .cells
                            .iter()
                            .map(|cell| {
                                Ok(TableCell {
                                    ann: f(&cell.ann)?,
                                    objects: cell
                                        .objects
                                        .iter()
                                        .map(|x| x.try_map_ann_with(f))
                                        .collect::<Result<_, _>>()?,
                                })
                            })
                            .collect::<Result<_, E>>()?,
                    })
                })
                .collect::<Result<_, E>>()?,
            column_alignments: self.column_alignments.clone(),
            formulas: self
                .formulas
                .iter()
                .map(|formula| formula.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for row in &self.rows {
            row.visit_with(f);
        }
        for formula in &self.formulas {
            formula.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for row in &mut self.rows {
            row.visit_mut_with(f);
        }
        for formula in &mut self.formulas {
            formula.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        for row in &self.rows {
            acc = row.fold_with(acc, f);
        }
        for formula in &self.formulas {
            acc = formula.fold_with(acc, f);
        }
        acc
    }
}

impl<A> TableRow<A> {
    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TableRow(self));
        for cell in &self.cells {
            cell.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TableRow(self));
        for cell in &mut self.cells {
            cell.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::TableRow(self));
        for cell in &self.cells {
            acc = cell.fold_with(acc, f);
        }
        acc
    }
}

impl<A> TableCell<A> {
    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TableCell(self));
        for object in &self.objects {
            object.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TableCell(self));
        for object in &mut self.objects {
            object.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let mut acc = f(init, AstRef::TableCell(self));
        for object in &self.objects {
            acc = object.fold_with(acc, f);
        }
        acc
    }
}

impl<A> Block<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Block<B>
    where
        F: FnMut(&A) -> B,
    {
        Block {
            kind: self.kind.clone(),
            name: self.name.clone(),
            language: self.language.clone(),
            switches: self.switches.clone(),
            line_numbering: self.line_numbering.clone(),
            preserve_indentation: self.preserve_indentation,
            code_refs: self.code_refs.clone(),
            parameters: self.parameters.clone(),
            header_args: self.header_args.clone(),
            value: self.value.clone(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Block<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Block {
            kind: self.kind.clone(),
            name: self.name.clone(),
            language: self.language.clone(),
            switches: self.switches.clone(),
            line_numbering: self.line_numbering.clone(),
            preserve_indentation: self.preserve_indentation,
            code_refs: self.code_refs.clone(),
            parameters: self.parameters.clone(),
            header_args: self.header_args.clone(),
            value: self.value.clone(),
            children: self
                .children
                .iter()
                .map(|x| x.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
        })
    }
}

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

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Object<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Object {
            ann: f(&self.ann)?,
            data: self.data.try_map_ann_with(f)?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::Object(self));
        self.data.visit_with(f);
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::Object(self));
        self.data.visit_mut_with(f);
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
    where
        F: FnMut(T, AstRef<'_, A>) -> T,
    {
        let acc = f(init, AstRef::Object(self));
        self.data.fold_with(acc, f)
    }
}

impl<A> ObjectData<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> ObjectData<B>
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
            ObjectData::FootnoteRef { label, definition } => ObjectData::FootnoteRef {
                label: label.clone(),
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
            ObjectData::Link(link) => ObjectData::Link(link.map_ann_with(f)),
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

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ObjectData<B>, E>
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
            ObjectData::FootnoteRef { label, definition } => ObjectData::FootnoteRef {
                label: label.clone(),
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
            ObjectData::Link(link) => ObjectData::Link(link.try_map_ann_with(f)?),
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

    fn visit_with<F>(&self, f: &mut F)
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

    fn visit_mut_with<F>(&mut self, f: &mut F)
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

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
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

impl<A> Link<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Link<B>
    where
        F: FnMut(&A) -> B,
    {
        Link {
            path: self.path.clone(),
            target: self.target.clone(),
            description: self.description.iter().map(|x| x.map_ann_with(f)).collect(),
            raw_description: self.raw_description.clone(),
            has_description: self.has_description,
            is_image: self.is_image,
            caption: self.caption.as_ref().map(|caption| caption.map_ann_with(f)),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Link<B>, E>
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
            raw_description: self.raw_description.clone(),
            has_description: self.has_description,
            is_image: self.is_image,
            caption: self
                .caption
                .as_ref()
                .map(|caption| caption.try_map_ann_with(f))
                .transpose()?,
        })
    }

    fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        if let Some(caption) = &self.caption {
            caption.visit_with(f);
        }
        for object in &self.description {
            object.visit_with(f);
        }
    }

    fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        if let Some(caption) = &mut self.caption {
            caption.visit_mut_with(f);
        }
        for object in &mut self.description {
            object.visit_mut_with(f);
        }
    }

    fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
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
        acc
    }
}

impl<A> Citation<A> {
    fn map_ann_with<B, F>(&self, f: &mut F) -> Citation<B>
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
                    id: x.id.clone(),
                    prefix: x.prefix.iter().map(|o| o.map_ann_with(f)).collect(),
                    suffix: x.suffix.iter().map(|o| o.map_ann_with(f)).collect(),
                })
                .collect(),
        }
    }

    fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Citation<B>, E>
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

    fn visit_with<F>(&self, f: &mut F)
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

    fn visit_mut_with<F>(&mut self, f: &mut F)
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

    fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
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
