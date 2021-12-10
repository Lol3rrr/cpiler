use std::{
    collections::HashSet,
    sync::{Arc, RwLock, Weak},
};

use crate::{PhiEntry, Statement, Type, Value, Variable};

/// A Basic-Block contains a linear series of Statements that will be executed one after another.
///
/// A Block can have any number of predecessors and can have any number of following Blocks that
/// can be jumped to using the appropriate Statements
#[derive(Debug)]
pub struct BasicBlock {
    /// The List of Predecessors from which you can jump to this Block
    pub predecessor: RwLock<Vec<Weak<BasicBlock>>>,
    /// The actual Statements in this Block
    pub parts: RwLock<Vec<Statement>>,
}

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        let self_pred_count = {
            let tmp = self.predecessor.read().unwrap();
            tmp.len()
        };
        let other_pred_count = {
            let tmp = other.predecessor.read().unwrap();
            tmp.len()
        };
        if self_pred_count != other_pred_count {
            return false;
        }

        let self_parts = self.parts.read().unwrap();
        let other_parts = other.parts.read().unwrap();

        let s_vec: &[Statement] = &self_parts;
        let o_vec: &[Statement] = &other_parts;
        s_vec.eq(o_vec)
    }
}

impl BasicBlock {
    /// Creates a new Block without any predecessors, this should only be used for the Global Block
    /// or similiar situations depending on your source language design
    pub fn initial(parts: Vec<Statement>) -> Arc<Self> {
        let instance = Self {
            predecessor: RwLock::new(Vec::new()),
            parts: RwLock::new(parts),
        };

        Arc::new(instance)
    }

    /// Creates a new Block with the given predecessors and statements
    pub fn new(predecessors: Vec<Weak<BasicBlock>>, parts: Vec<Statement>) -> Arc<Self> {
        Arc::new(Self {
            predecessor: RwLock::new(predecessors),
            parts: RwLock::new(parts),
        })
    }

    /// Obtains the Weak-Ptr for this Block
    pub fn weak_ptr(self: &Arc<Self>) -> Weak<Self> {
        Arc::downgrade(self)
    }

    pub fn add_predecessor(&self, pred: Weak<Self>) {
        let mut tmp = self.predecessor.write().unwrap();
        tmp.push(pred);
    }

    /// Appends the given Statement to the current List of Statements
    pub fn add_statement(&self, statement: Statement) {
        let mut tmp = self.parts.write().unwrap();
        tmp.push(statement);
    }

    /// Attempts to retrieve the latest defined Instance for a Variable with the given Name in this
    /// Block locally, meaning that it wont start a recursive Search in its predecessors
    fn local_definition(&self, name: &str) -> Option<Variable> {
        let tmp = self.parts.read().unwrap();

        tmp.iter().rev().find_map(|stmnt| match stmnt {
            Statement::Assignment { target, .. } if target.name == name => Some(target.clone()),
            _ => None,
        })
    }

    pub fn definition(&self, name: &str) -> Option<Variable> {
        if let Some(var) = self.local_definition(name) {
            return Some(var);
        }

        let preds = self.predecessor.read().unwrap();

        if preds.len() == 0 {
            return None;
        } else if preds.len() == 1 {
            let single_pred = preds.get(0).unwrap();
            match single_pred.upgrade() {
                Some(pred) => return pred.definition(name),
                None => return None,
            }
        } else {
            let tmp_var_name = self.get_next_tmp_name();
            let tmp_var = Variable::new(&tmp_var_name, Type::Void);
            let phi_stmnt = Statement::Assignment {
                target: tmp_var,
                value: Value::Phi { sources: vec![] },
            };

            {
                let mut tmp = self.parts.write().unwrap();
                tmp.push(phi_stmnt);
            }

            let mut sources = Vec::with_capacity(preds.len());
            for raw_pred in preds.iter() {
                let c_pred = raw_pred.clone();
                let pred = match raw_pred.upgrade() {
                    Some(p) => p,
                    None => continue,
                };

                match pred.definition(name) {
                    Some(var) => {
                        sources.push(PhiEntry { var, block: c_pred });
                    }
                    None => {}
                };
            }

            if sources.is_empty() {
                {
                    let mut tmp = self.parts.write().unwrap();
                    tmp.pop();
                }

                return None;
            }

            let ty = sources.get(0).unwrap().var.ty.clone();

            let final_var = Variable::new(tmp_var_name, ty);
            let tmp_stmnt = Statement::Assignment {
                target: final_var.clone(),
                value: Value::Phi { sources },
            };

            {
                let mut tmp = self.parts.write().unwrap();
                let last = tmp.last_mut().unwrap();

                *last = tmp_stmnt;
            }

            Some(final_var)
        }
    }

    // TODO
    // Right now this does not look for previous tempoaries in its predecessors which can cause
    // the existance of multiple temporaries with the same name
    /// Loads the Name for the next Temporary Variable in this Block, which is particularly useful
    /// when breaking up nested Expressions or the like
    pub fn get_next_tmp_name(&self) -> String {
        let tmp = self.parts.read().unwrap();

        let latest = tmp.iter().rev().find_map(|stmnt| match stmnt {
            Statement::Assignment { target, .. } if target.name.starts_with("__t_") => {
                Some(target.name.clone())
            }
            _ => None,
        });

        match latest {
            Some(raw_name) => {
                let raw_numb = raw_name.strip_prefix("__t_").unwrap();
                let last_numb: usize = raw_numb.parse().unwrap();

                let n_numb = last_numb + 1;
                format!("__t_{}", n_numb)
            }
            None => "__t_0".to_string(),
        }
    }

    /// Generates the .dot Graphviz stuff
    pub fn to_dot(
        self: &Arc<Self>,
        lines: &mut Vec<String>,
        drawn: &mut HashSet<*const BasicBlock>,
    ) -> String {
        let self_ptr = Arc::as_ptr(&self);
        let block_name = format!("block_{}", self_ptr as usize);
        if drawn.contains(&self_ptr) {
            return block_name;
        }
        drawn.insert(self_ptr);

        lines.push(format!(
            "{} [label = \"{} - Block Start\"]",
            block_name, block_name
        ));

        {
            let mut src = block_name.clone();

            let parts = self.parts.read().unwrap();
            for (numb, part) in parts.iter().enumerate() {
                src = part.to_dot(lines, drawn, self_ptr, numb, &src);
            }
        }

        {
            let preds = self.predecessor.read().unwrap();
            for pred in preds.iter() {
                let pred_name = format!("block_{}", pred.as_ptr() as usize);
                let pred_line = format!("{} -> {} [style=dashed]", block_name, pred_name);
                lines.push(pred_line);
            }
        }

        block_name
    }
}

#[cfg(test)]
mod tests {
    use crate::{Constant, PhiEntry, Type, Value};

    use super::*;

    #[test]
    fn get_last_definition_empty() {
        let parts = vec![];
        let block = BasicBlock::initial(parts);

        let expected = None;

        let result = block.definition("test");

        assert_eq!(expected, result);
    }
    #[test]
    fn get_last_definition_single() {
        let parts = vec![Statement::Assignment {
            target: Variable::new_test("test", 0, Type::I8),
            value: Value::Constant(Constant::I8(0)),
        }];
        let block = BasicBlock::initial(parts);

        let expected = Some(Variable::new_test("test", 0, Type::I8));

        let result = block.definition("test");

        assert_eq!(expected, result);
    }
    #[test]
    fn get_last_definition_multiple() {
        let parts = vec![
            Statement::Assignment {
                target: Variable::new_test("test", 0, Type::I8),
                value: Value::Constant(Constant::I8(0)),
            },
            Statement::Assignment {
                target: Variable::new_test("test", 1, Type::I8),
                value: Value::Constant(Constant::I8(1)),
            },
        ];
        let block = BasicBlock::initial(parts);

        let expected = Some(Variable::new_test("test", 1, Type::I8));

        let result = block.definition("test");

        assert_eq!(expected, result);
    }

    #[test]
    fn definition_in_single_predecessor() {
        let predecessor = BasicBlock::initial(vec![Statement::Assignment {
            target: Variable::new_test("test", 0, Type::I8),
            value: Value::Constant(Constant::I8(1)),
        }]);

        let pred = Arc::downgrade(&predecessor);
        let block = BasicBlock::new(vec![pred], vec![]);

        let expected = Some(Variable::new_test("test", 0, Type::I8));

        let result = block.definition("test");

        assert_eq!(expected, result);
    }

    #[test]
    fn definition_in_multiple_predecessors() {
        let predecessor_1 = BasicBlock::initial(vec![Statement::Assignment {
            target: Variable::new_test("test", 0, Type::I8),
            value: Value::Constant(Constant::I8(1)),
        }]);
        let pred_1 = Arc::downgrade(&predecessor_1);

        let predecessor_2 = BasicBlock::initial(vec![Statement::Assignment {
            target: Variable::new_test("test", 1, Type::I8),
            value: Value::Constant(Constant::I8(2)),
        }]);
        let pred_2 = Arc::downgrade(&predecessor_2);

        let block = BasicBlock::new(vec![pred_1.clone(), pred_2.clone()], vec![]);

        let expected = Some(Variable::new_test("__t_0", 0, Type::I8));
        let expected_block_stmnts = vec![Statement::Assignment {
            target: Variable::new_test("__t_0", 0, Type::I8),
            value: Value::Phi {
                sources: vec![
                    PhiEntry {
                        block: pred_1,
                        var: Variable::new_test("test", 0, Type::I8),
                    },
                    PhiEntry {
                        block: pred_2,
                        var: Variable::new_test("test", 1, Type::I8),
                    },
                ],
            },
        }];

        let result = block.definition("test");
        let result_block_stmnts = block.parts.read().unwrap().clone();

        assert_eq!(expected_block_stmnts, result_block_stmnts);
        assert_eq!(expected, result);
    }
}
