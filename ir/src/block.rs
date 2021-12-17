use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    comp::CompareGraph,
    dot::{Context, DrawnBlocks},
    PhiEntry, Statement, ToDot, Type, Value, Variable,
};

mod inner;
use general::dot::{self, Graph};
pub use inner::*;

mod weak;
pub use weak::*;

/// A Basic-Block contains a linear series of Statements that will be executed one after another.
///
/// A Block can have any number of predecessors and can have any number of following Blocks that
/// can be jumped to using the appropriate Statements
#[derive(Debug, Clone)]
pub struct BasicBlock(Arc<InnerBlock>);

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        let mut blocks = HashMap::new();

        self.compare(other, &mut blocks, 0)
    }
}

impl CompareGraph for BasicBlock {
    fn compare(
        &self,
        other: &Self,
        blocks: &mut HashMap<*const InnerBlock, usize>,
        current_block: usize,
    ) -> bool {
        let self_ptr = self.as_ptr();
        let other_ptr = other.as_ptr();
        match (blocks.get(&self_ptr), blocks.get(&other_ptr)) {
            (Some(own_index), Some(other_index)) => {
                return own_index == other_index;
            }
            (None, None) => {}
            _ => {
                todo!()
            }
        };
        blocks.insert(self_ptr, current_block);
        blocks.insert(other_ptr, current_block);

        let self_pred_count = {
            let tmp = self.0.predecessor.read().unwrap();
            tmp.len()
        };
        let other_pred_count = {
            let tmp = other.0.predecessor.read().unwrap();
            tmp.len()
        };
        if self_pred_count != other_pred_count {
            return false;
        }

        let self_parts = self.0.parts.read().unwrap();
        let other_parts = other.0.parts.read().unwrap();

        let s_vec: &[Statement] = &self_parts;
        let o_vec: &[Statement] = &other_parts;

        if s_vec.len() != o_vec.len() {
            return false;
        }

        for (own_s, other_s) in s_vec.iter().zip(o_vec.iter()) {
            dbg!(&own_s, &other_s);

            if !own_s.compare(other_s, blocks, current_block + 1) {
                return false;
            }
        }

        true
    }
}

impl From<Arc<InnerBlock>> for BasicBlock {
    fn from(inner: Arc<InnerBlock>) -> Self {
        Self(inner)
    }
}

impl BasicBlock {
    /// Creates a new Block without any predecessors, this should only be used for the Global Block
    /// or similiar situations depending on your source language design
    pub fn initial(parts: Vec<Statement>) -> Self {
        Self::new(Vec::new(), parts)
    }

    /// Creates a new Block with the given predecessors and statements
    pub fn new(predecessors: Vec<WeakBlockPtr>, parts: Vec<Statement>) -> Self {
        Self(Arc::new(InnerBlock {
            predecessor: RwLock::new(predecessors),
            parts: RwLock::new(parts),
        }))
    }

    /// Obtains the Weak-Ptr for this Block
    pub fn weak_ptr(&self) -> WeakBlockPtr {
        let inner = Arc::downgrade(&self.0);
        inner.into()
    }
    /// Gets the Ptr to the Data
    pub fn as_ptr(&self) -> *const InnerBlock {
        Arc::as_ptr(&self.0)
    }

    /// Clones all the Predecessors of the current Block
    pub fn get_predecessors(&self) -> Vec<WeakBlockPtr> {
        let tmp = self.0.predecessor.read().unwrap();
        tmp.clone()
    }

    /// Adds a new Predecessor to this Block
    pub fn add_predecessor(&self, pred: WeakBlockPtr) {
        let mut tmp = self.0.predecessor.write().unwrap();

        if tmp.iter().any(|tmp| tmp.as_ptr() == pred.as_ptr()) {
            return;
        }
        tmp.push(pred);
    }

    /// Removes the Predecessor from this Block
    pub fn remove_predecessor(&self, pred: WeakBlockPtr) {
        let mut tmp = self.0.predecessor.write().unwrap();

        if let Some(index) =
            tmp.iter()
                .enumerate()
                .find_map(|(index, ptr)| if ptr == &pred { Some(index) } else { None })
        {
            tmp.remove(index);
        }
    }

    /// Clones all the Statements currently in the Bloc
    pub fn get_statements(&self) -> Vec<Statement> {
        let tmp = self.0.parts.read().unwrap();
        tmp.clone()
    }

    /// Appends the given Statement to the current List of Statements
    pub fn add_statement(&self, statement: Statement) {
        let mut tmp = self.0.parts.write().unwrap();
        tmp.push(statement);
    }

    /// Replaces the current Statements with the given Statements
    pub fn set_statements(&self, statements: Vec<Statement>) {
        let mut tmp = self.0.parts.write().unwrap();
        *tmp = statements;
    }

    /// Attempts to retrieve the latest defined Instance for a Variable with the given Name in this
    /// Block locally, meaning that it wont start a recursive Search in its predecessors
    fn local_definition(&self, name: &str) -> Option<Variable> {
        let tmp = self.0.parts.read().unwrap();

        tmp.iter().rev().find_map(|stmnt| match stmnt {
            Statement::Assignment { target, .. } if target.name == name => Some(target.clone()),
            _ => None,
        })
    }

    /// Finds the Definition for the given Variable Name in this block any of its Predecessors
    /// using a recursive look-up.
    /// Returns the Variable where it was defined, if there is only one definition or creates a new
    /// temporary Variable that combines the different definitions using a Phi-Node
    pub fn definition<T>(&self, name: &str, tmp_numb: &T) -> Option<Variable>
    where
        T: Fn() -> usize,
    {
        if let Some(var) = self.local_definition(name) {
            return Some(var);
        }

        let preds = self.0.predecessor.read().unwrap();

        if preds.len() == 0 {
            None
        } else if preds.len() == 1 {
            let single_pred = preds.get(0).unwrap();
            match single_pred.upgrade() {
                Some(pred) => pred.definition(name, tmp_numb),
                None => None,
            }
        } else {
            let tmp_var_numb = tmp_numb();
            let tmp_var = Variable::tmp(tmp_var_numb, Type::Void);
            let phi_stmnt = Statement::Assignment {
                target: tmp_var,
                value: Value::Phi { sources: vec![] },
            };

            {
                let mut tmp = self.0.parts.write().unwrap();
                tmp.push(phi_stmnt);
            }

            let mut sources = Vec::with_capacity(preds.len());
            for raw_pred in preds.iter() {
                let c_pred = raw_pred.clone();
                let pred = match raw_pred.upgrade() {
                    Some(p) => p,
                    None => continue,
                };

                if let Some(var) = pred.definition(name, tmp_numb) {
                    sources.push(PhiEntry { var, block: c_pred });
                }
            }

            if sources.is_empty() {
                {
                    let mut tmp = self.0.parts.write().unwrap();
                    tmp.pop();
                }

                return None;
            }

            let ty = sources.get(0).unwrap().var.ty.clone();

            let final_var = Variable::tmp(tmp_var_numb, ty);
            let tmp_stmnt = Statement::Assignment {
                target: final_var.clone(),
                value: Value::Phi { sources },
            };

            {
                let mut tmp = self.0.parts.write().unwrap();
                let last = tmp.last_mut().unwrap();

                *last = tmp_stmnt;
            }

            Some(final_var)
        }
    }

    /// Loads all the direct Successors of this Block
    pub fn successors(&self) -> HashMap<*const InnerBlock, Self> {
        let mut result = HashMap::new();

        let parts = self.0.parts.read().unwrap();
        for tmp in parts.iter() {
            match tmp {
                Statement::Jump(target) => {
                    let target_ptr = target.as_ptr();

                    result.insert(target_ptr, target.clone());
                }
                Statement::JumpTrue(_, target) => {
                    let target_ptr = target.as_ptr();

                    result.insert(target_ptr, target.clone());
                }
                _ => {}
            };
        }

        result
    }
}

impl ToDot for BasicBlock {
    fn to_dot(&self, lines: &mut dyn dot::Graph, drawn: &mut DrawnBlocks, ctx: &Context) -> String {
        let self_ptr = Arc::as_ptr(&self.0);
        let block_name = format!("block_{}", self_ptr as usize);
        if drawn.contains(&block_name) {
            return block_name;
        }
        drawn.add_block(&block_name);

        {
            let succs = self.successors();
            for succ in succs.values() {
                succ.to_dot(lines, drawn, ctx);
            }
        }

        let mut block_graph = dot::SubGraph::new(&block_name).cluster();
        let label_content = format!("{} - Block Start", block_name);
        block_graph.add_node(dot::Node::new(&block_name).add_label("label", label_content));

        {
            let parts = self.0.parts.read().unwrap();
            let mut parts_context = Context::new();
            parts_context.set("block_ptr", self_ptr as usize);
            parts_context.set("block_src", block_name.clone());

            for (numb, part) in parts.iter().enumerate() {
                parts_context.set("block_number", numb);
                let n_src = part.to_dot(&mut block_graph, drawn, &parts_context);
                parts_context.set("block_src", n_src);
            }
        }
        lines.add_subgraph(block_graph);

        {
            let preds = self.0.predecessor.read().unwrap();
            for pred in preds.iter() {
                let pred_name = format!("block_{}", pred.as_ptr() as usize);
                lines.add_edge(dot::Edge::new(&block_name, pred_name).add_label("style", "dashed"));
            }
        }

        block_name
    }

    fn name(&self, _: &Context) -> String {
        let self_ptr = Arc::as_ptr(&self.0);
        format!("block_{}", self_ptr as usize)
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

        let result = block.definition("test", &|| 0);

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

        let result = block.definition("test", &|| 0);

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

        let result = block.definition("test", &|| 0);

        assert_eq!(expected, result);
    }

    #[test]
    fn definition_in_single_predecessor() {
        let predecessor = BasicBlock::initial(vec![Statement::Assignment {
            target: Variable::new_test("test", 0, Type::I8),
            value: Value::Constant(Constant::I8(1)),
        }]);

        let pred = predecessor.weak_ptr();
        let block = BasicBlock::new(vec![pred], vec![]);

        let expected = Some(Variable::new_test("test", 0, Type::I8));

        let result = block.definition("test", &|| 0);

        assert_eq!(expected, result);
    }

    #[test]
    fn definition_in_multiple_predecessors() {
        let predecessor_1 = BasicBlock::initial(vec![Statement::Assignment {
            target: Variable::new_test("test", 0, Type::I8),
            value: Value::Constant(Constant::I8(1)),
        }]);
        let pred_1 = predecessor_1.weak_ptr();

        let predecessor_2 = BasicBlock::initial(vec![Statement::Assignment {
            target: Variable::new_test("test", 1, Type::I8),
            value: Value::Constant(Constant::I8(2)),
        }]);
        let pred_2 = predecessor_2.weak_ptr();

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

        let result = block.definition("test", &|| 0);
        let result_block_stmnts = block.0.parts.read().unwrap().clone();

        let mut tmp_map = HashMap::new();
        for (expected_stmnt, result_stmnt) in expected_block_stmnts
            .into_iter()
            .zip(result_block_stmnts.into_iter())
        {
            assert!(expected_stmnt.compare(&result_stmnt, &mut tmp_map, 0));
        }
        assert_eq!(expected, result);
    }
}
