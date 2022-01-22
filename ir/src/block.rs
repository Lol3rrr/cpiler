use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    comp::CompareGraph,
    dot::{Context, DrawnBlocks},
    DominanceTree, InterferenceGraph, NodeId, PhiEntry, Statement, ToDot, Type, Value, Variable,
    VariableMetadata,
};

mod inner;
use graphviz::Graph;
pub use inner::*;

mod weak;
pub use weak::*;

mod iter;
pub use iter::BlockIter;

mod linear;
pub use linear::LinearIter;

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

    /// Returns an Iterator over all the Blocks that can be reached from this Block as the starting
    /// Point, including this Block itself
    pub fn block_iter(&self) -> BlockIter {
        BlockIter::new(self.0.clone())
    }
    pub fn linear_iter(&self) -> LinearIter {
        LinearIter::new(self.0.clone())
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

    // TODO
    // This currently causes a StackOverflow in certain Cases so this should probably be turned
    // into an iterative approach however im not quite sure on how to do that
    /// Finds the Definition for the given Variable Name in this block any of its Predecessors
    /// using a recursive look-up.
    /// Returns the Variable where it was defined, if there is only one definition or creates a new
    /// temporary Variable that combines the different definitions using a Phi-Node
    pub fn definition<T>(&self, name: &str, tmp_numb: &T) -> Option<Variable>
    where
        T: Fn() -> usize,
    {
        if !self.has_definition(name) {
            return None;
        }

        if let Some(var) = self.local_definition(name) {
            return Some(var);
        }

        let preds = self.0.predecessor.read().unwrap().clone();

        if preds.is_empty() {
            return None;
        }

        if preds.len() == 1 {
            let single_pred = preds.get(0).unwrap();
            match single_pred.upgrade() {
                Some(pred) => return pred.definition(name, tmp_numb),
                None => return None,
            };
        }

        let tmp_var = Variable::new(name, Type::Void).set_meta(VariableMetadata::Temporary);
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

        let var = sources.get(0).unwrap().var.clone();
        if sources.iter().all(|s| s.var == var) && sources.len() > 1 {
            let mut tmp = self.0.parts.write().unwrap();
            tmp.pop();

            return Some(var);
        }

        let final_var = var.next_gen();
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

    /// This will update all the Phi Nodes in this Block to fill in any missing ways a Variable
    /// might be defined by its predecessors and also filters out and dead Variables, where the
    /// Block has been removed
    pub fn refresh_phis(&self) {
        let mut statements = self.get_statements();

        let preds = self.get_predecessors();

        for stmnt in statements.iter_mut() {
            let target = match stmnt {
                Statement::Assignment {
                    target,
                    value: Value::Phi { .. },
                } => target,
                _ => continue,
            };

            let mut sources = Vec::with_capacity(preds.len());
            for raw_pred in preds.iter() {
                let c_pred = raw_pred.clone();
                let pred = match raw_pred.upgrade() {
                    Some(p) => p,
                    None => continue,
                };

                if let Some(var) = pred.definition(&target.name, &|| panic!()) {
                    sources.push(PhiEntry { var, block: c_pred });
                }
            }

            *stmnt = Statement::Assignment {
                target: target.clone(),
                value: Value::Phi { sources },
            };
        }

        self.set_statements(statements);
    }

    /// Checks if there exists a Variable Definition/Assignment for the given Name
    pub fn has_definition(&self, name: &str) -> bool {
        let mut preds = vec![self.weak_ptr()];
        let mut visited = HashSet::new();

        while let Some(raw_pred) = preds.pop() {
            let pred_ptr = raw_pred.as_ptr();
            if visited.contains(&pred_ptr) {
                continue;
            }

            let pred = match raw_pred.upgrade() {
                Some(p) => p,
                None => continue,
            };

            if pred.local_definition(name).is_some() {
                return true;
            }
            visited.insert(pred_ptr);

            preds.extend(pred.0.predecessor.read().unwrap().clone());
        }

        false
    }

    /// Loads all the direct Successors of this Block
    pub fn successors(&self) -> BTreeMap<*const InnerBlock, Self> {
        let mut result = BTreeMap::new();

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

    /// Gets the Number of uses for all the Variables used in the current Block
    pub fn block_used_vars(&self) -> HashMap<Variable, usize> {
        let mut result = HashMap::new();

        let stmnts = self.0.parts.read().unwrap();
        for tmp in stmnts.iter() {
            tmp.used_vars().into_iter().for_each(|u_v| {
                match result.get_mut(&u_v) {
                    Some(u_count) => {
                        *u_count += 1;
                    }
                    None => {
                        result.insert(u_v, 1);
                    }
                };
            });
        }

        result
    }

    /// Gets the Number of uses for all the Variables used in the successors of this Block
    pub fn following_uses(&self) -> HashMap<Variable, usize> {
        let mut base = HashMap::new();

        for succ in self.block_iter().skip(1) {
            let succ_uses = succ.block_used_vars();

            for (u_v, u_c) in succ_uses.into_iter() {
                match base.get_mut(&u_v) {
                    Some(b_c) => {
                        *b_c += u_c;
                    }
                    None => {
                        base.insert(u_v, u_c);
                    }
                };
            }
        }

        base
    }

    pub(crate) fn earliest_common_block(&self, other: &Self) -> Option<Self> {
        let own_iter = self.block_iter();
        let other_iter = other.block_iter();

        let own_succs: HashMap<*const InnerBlock, BasicBlock> =
            own_iter.map(|b| (b.as_ptr(), b)).collect();

        for other_succ in other_iter {
            let other_ptr = other_succ.as_ptr();

            if own_succs.contains_key(&other_ptr) {
                return Some(other_succ);
            }
        }

        None
    }

    pub(crate) fn interference_graph<T, F>(
        &self,
        graph: &mut T,
        live_vars: &mut HashSet<Variable>,
        visited: &mut HashSet<*const InnerBlock>,
        update: &mut F,
    ) where
        T: InterferenceGraph,
        F: FnMut(&HashSet<Variable>, &BasicBlock, usize),
    {
        visited.insert(self.as_ptr());

        let mut block_uses = self.block_used_vars();
        let following_uses = self.following_uses();
        let statements = self.get_statements();

        for (index, stmnt) in statements.into_iter().enumerate() {
            match &stmnt {
                Statement::Assignment { target, .. } => {
                    let target_node = NodeId::new(target.clone());
                    graph.add_node(target_node.clone());

                    for live in live_vars.iter() {
                        graph.add_edge(target_node.clone(), NodeId::new(live.clone()));
                    }

                    live_vars.insert(target.clone());
                }
                _ => {}
            };

            update(live_vars, self, index);

            let tmp_used = stmnt.used_vars();
            for used in tmp_used {
                match block_uses.get_mut(&used) {
                    Some(b_u) => {
                        *b_u -= 1;

                        if *b_u == 0 {
                            block_uses.remove(&used);

                            if !following_uses.contains_key(&used) {
                                live_vars.remove(&used);
                            }
                        }
                    }
                    None => {
                        dbg!(&used);
                    }
                };
            }
        }

        let succs: HashMap<_, _> = self
            .successors()
            .into_iter()
            .filter(|(p, _)| !visited.contains(p))
            .collect();
        if succs.is_empty() {
            return;
        }

        if succs.len() == 1 {
            let (_, single_succ) = succs.into_iter().next().unwrap();
            single_succ.interference_graph(graph, live_vars, visited, update);
            return;
        }

        assert!(succs.len() == 2);
        let (left, right) = {
            let mut tmp = succs.into_iter().map(|(_, b)| b);
            (tmp.next().unwrap(), tmp.next().unwrap())
        };

        let end_block = match left.earliest_common_block(&right) {
            Some(b) => b,
            None => {
                dbg!(&left, &right);

                for left_succ in left.block_iter() {
                    dbg!(left_succ.as_ptr());
                }
                for right_succ in right.block_iter() {
                    dbg!(right_succ.as_ptr());
                }

                todo!()
            }
        };

        visited.insert(end_block.as_ptr());
        let mut left_live = live_vars.clone();
        left.interference_graph(graph, &mut left_live, visited, update);

        let mut right_live = live_vars.clone();
        right.interference_graph(graph, &mut right_live, visited, update);

        let mut n_live = {
            let mut tmp = left_live;
            tmp.extend(right_live);
            tmp
        };
        end_block.interference_graph(graph, &mut n_live, visited, update);
    }

    /// This will generate the Dominance Tree for this Block and all its immediate Successors that
    /// have not already been visited
    pub fn dominance_tree(&self, visited: &mut HashSet<*const InnerBlock>) -> DominanceTree {
        visited.insert(self.as_ptr());

        let mut result = DominanceTree::new();

        let statements = self.get_statements();
        for stmnt in statements {
            if let Statement::Assignment { target, .. } = stmnt {
                result.append(target);
            }
        }

        let succs = self.successors();

        if succs.is_empty() {
            return result;
        }

        if succs.len() == 1 {
            let (_, succ) = succs.into_iter().next().unwrap();
            if visited.contains(&succ.as_ptr()) {
                return result;
            }

            let succ_tree = succ.dominance_tree(visited);

            result.append_tree(succ_tree);

            return result;
        }

        assert!(succs.len() == 2);

        let current_node = result.current_node();

        let mut succ_iter = succs.into_iter();
        let (_, left_block) = succ_iter.next().unwrap();
        let (_, right_block) = succ_iter.next().unwrap();

        let common_block = left_block.earliest_common_block(&right_block).unwrap();
        visited.insert(common_block.as_ptr());

        if !visited.contains(&left_block.as_ptr()) {
            let left_tree = left_block.dominance_tree(visited);
            result.append_tree_to_node(&current_node, left_tree);
        }
        if !visited.contains(&right_block.as_ptr()) {
            let right_tree = right_block.dominance_tree(visited);
            result.append_tree_to_node(&current_node, right_tree);
        }
        visited.remove(&common_block.as_ptr());

        let common_tree = common_block.dominance_tree(visited);
        result.append_tree_to_node(&current_node, common_tree);

        result
    }
}

impl ToDot for BasicBlock {
    fn to_dot(
        &self,
        lines: &mut dyn graphviz::Graph,
        drawn: &mut DrawnBlocks,
        ctx: &Context,
    ) -> String {
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

        let mut block_graph = graphviz::SubGraph::new(&block_name).cluster();
        let label_content = format!("{} - Block Start", block_name);
        block_graph.add_node(graphviz::Node::new(&block_name).add_label("label", label_content));

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
                lines.add_edge(
                    graphviz::Edge::new(&block_name, pred_name).add_label("style", "dashed"),
                );
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
        let var_test0 = Variable::new("test", Type::I8);
        let var_test1 = var_test0.next_gen();

        let predecessor_1 = BasicBlock::initial(vec![Statement::Assignment {
            target: var_test0.clone(),
            value: Value::Constant(Constant::I8(1)),
        }]);
        let pred_1 = predecessor_1.weak_ptr();

        let predecessor_2 = BasicBlock::initial(vec![Statement::Assignment {
            target: var_test1.clone(),
            value: Value::Constant(Constant::I8(2)),
        }]);
        let pred_2 = predecessor_2.weak_ptr();

        let block = BasicBlock::new(vec![pred_1.clone(), pred_2.clone()], vec![]);

        let expected = Some(Variable::new_test("test", 2, Type::I8));
        let expected_block_stmnts = vec![Statement::Assignment {
            target: Variable::new_test("test", 2, Type::I8),
            value: Value::Phi {
                sources: vec![
                    PhiEntry {
                        block: pred_1,
                        var: var_test0,
                    },
                    PhiEntry {
                        block: pred_2,
                        var: var_test1,
                    },
                ],
            },
        }];

        let result = block.definition("test", &|| 0);
        let result_block_stmnts = block.0.parts.read().unwrap().clone();
        dbg!(&result_block_stmnts);

        let mut tmp_map = HashMap::new();
        for (expected_stmnt, result_stmnt) in expected_block_stmnts
            .into_iter()
            .zip(result_block_stmnts.into_iter())
        {
            assert!(expected_stmnt.compare(&result_stmnt, &mut tmp_map, 0));
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn definition_common_pred_with_multiple_preds_between() {
        let test_var = Variable::new("test", Type::I8);
        let predecessor_1 = BasicBlock::initial(vec![Statement::Assignment {
            target: test_var.clone(),
            value: Value::Constant(Constant::I8(1)),
        }]);

        let predecessor_2 = BasicBlock::new(vec![predecessor_1.weak_ptr()], vec![]);
        let predecessor_3 = BasicBlock::new(vec![predecessor_1.weak_ptr()], vec![]);
        predecessor_1.add_statement(Statement::Jump(predecessor_2.clone()));
        predecessor_1.add_statement(Statement::Jump(predecessor_3.clone()));

        let block = BasicBlock::new(
            vec![predecessor_2.weak_ptr(), predecessor_3.weak_ptr()],
            vec![],
        );
        predecessor_2.add_statement(Statement::Jump(block.clone()));
        predecessor_3.add_statement(Statement::Jump(block.clone()));

        let expected = Some(test_var.clone());
        let expected_block_stmnts: Vec<Statement> = vec![];

        let result = block.definition("test", &|| 0);
        dbg!(&result);
        let result_block_stmnts = block.0.parts.read().unwrap().clone();
        dbg!(&result_block_stmnts);

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
