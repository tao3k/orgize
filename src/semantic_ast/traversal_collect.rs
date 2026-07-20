use super::{AstMut, AstRef, Block, List, ListItem, Table, TableCell, TableFormula, TableRow};

impl<A> List<A> {
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> List<B>
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

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<List<B>, E>
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

    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        for item in &self.items {
            item.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        for item in &mut self.items {
            item.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
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
    pub(super) fn visit_with<F>(&self, f: &mut F)
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

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
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

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
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
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Table<B>
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
            parsed_formulas: self
                .parsed_formulas
                .iter()
                .map(|formula| TableFormula {
                    ann: f(&formula.ann),
                    raw: formula.raw.clone(),
                    assignments: formula.assignments.clone(),
                })
                .collect(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Table<B>, E>
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
            parsed_formulas: self
                .parsed_formulas
                .iter()
                .map(|formula| {
                    Ok(TableFormula {
                        ann: f(&formula.ann)?,
                        raw: formula.raw.clone(),
                        assignments: formula.assignments.clone(),
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    pub(super) fn visit_with<F>(&self, f: &mut F)
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

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
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

    pub(super) fn fold_with<T, F>(&self, mut acc: T, f: &mut F) -> T
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
    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TableRow(self));
        for cell in &self.cells {
            cell.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TableRow(self));
        for cell in &mut self.cells {
            cell.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
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
    pub(super) fn visit_with<F>(&self, f: &mut F)
    where
        F: FnMut(AstRef<'_, A>),
    {
        f(AstRef::TableCell(self));
        for object in &self.objects {
            object.visit_with(f);
        }
    }

    pub(super) fn visit_mut_with<F>(&mut self, f: &mut F)
    where
        F: FnMut(AstMut<'_, A>),
    {
        f(AstMut::TableCell(self));
        for object in &mut self.objects {
            object.visit_mut_with(f);
        }
    }

    pub(super) fn fold_with<T, F>(&self, init: T, f: &mut F) -> T
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
    pub(super) fn map_ann_with<B, F>(&self, f: &mut F) -> Block<B>
    where
        F: FnMut(&A) -> B,
    {
        Block {
            kind: self.kind.clone(),
            name: self.name.clone(),
            language: self.language.clone(),
            switches: self.switches.clone(),
            switch_options: self.switch_options.clone(),
            line_numbering: self.line_numbering.clone(),
            preserve_indentation: self.preserve_indentation,
            lines: self.lines.iter().map(|line| line.map_ann_with(f)).collect(),
            code_refs: self.code_refs.clone(),
            parameters: self.parameters.clone(),
            header_args: self.header_args.clone(),
            value: self.value.clone(),
            children: self.children.iter().map(|x| x.map_ann_with(f)).collect(),
        }
    }

    pub(super) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<Block<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(Block {
            kind: self.kind.clone(),
            name: self.name.clone(),
            language: self.language.clone(),
            switches: self.switches.clone(),
            switch_options: self.switch_options.clone(),
            line_numbering: self.line_numbering.clone(),
            preserve_indentation: self.preserve_indentation,
            lines: self
                .lines
                .iter()
                .map(|line| line.try_map_ann_with(f))
                .collect::<Result<_, _>>()?,
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
