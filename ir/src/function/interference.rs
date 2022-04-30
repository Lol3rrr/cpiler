use std::collections::HashMap;

use graphs::directed::{ChainEntry, DirectedChain, DirectedGraph};

use crate::{BasicBlock, InterferenceGraph, Statement, Variable};

#[derive(Debug, PartialEq, Clone)]
struct LiveVars {
    vars: HashMap<Variable, usize>,
}

impl LiveVars {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Adds a new Variable to the Set of Live-Vars
    pub fn add_var(&mut self, var: Variable, uses: usize) {
        if uses == 0 {
            println!("Unused: {:?}", var);
            return;
        }

        #[cfg(debug_assertions)]
        if self.vars.contains_key(&var) {
            panic!(
                "Tried to insert into the Interference Graph again: {:?}",
                var
            );
        }

        let insert_res = self.vars.insert(var, uses);
        assert!(insert_res.is_none());
    }

    /// Decrements the use counter for the given Variable and removes it, if the count reaches 0
    pub fn used_var(&mut self, var: &Variable) -> Result<(), ()> {
        let remaining = self.vars.get_mut(var).ok_or(())?;

        *remaining = remaining.saturating_sub(1);

        if *remaining == 0 {
            self.vars.remove(var);
        }

        Ok(())
    }

    pub fn merge_branched(&mut self, left: Self, right: Self) {
        let union_vars: HashMap<_, _> = self
            .vars
            .iter()
            .filter_map(|(var, count)| {
                let left_delta = count - left.vars.get(var)?;
                let right_delta = count - right.vars.get(var)?;

                let n_count = count.saturating_sub(left_delta).saturating_sub(right_delta);

                Some((var, n_count))
            })
            .map(|(v, c)| (Variable::clone(v), c))
            .filter(|(_, c)| *c > 0)
            .collect();

        let left_exclusive = left
            .vars
            .into_iter()
            .filter(|(v, _)| !self.vars.contains_key(v));

        let right_exclusive = right
            .vars
            .into_iter()
            .filter(|(v, _)| !self.vars.contains_key(v));

        let n_vars = {
            let mut tmp = union_vars;
            tmp.extend(left_exclusive);
            tmp.extend(right_exclusive);
            tmp
        };

        self.vars = n_vars;
    }

    /// Returns an iterator over the currently live Variables
    pub fn iter(&self) -> impl Iterator<Item = &Variable> + '_ {
        self.vars.keys()
    }
}

/// Counts the Number of uses for the Variables found along the entire Chain
fn count_uses(chain: DirectedChain<'_, BasicBlock>) -> HashMap<Variable, usize> {
    let mut result = HashMap::new();

    for block in chain.flatten() {
        for stmnt in block.get_statements() {
            for var in stmnt.used_vars() {
                let entry = result.entry(var);
                let value = entry.or_insert(0);
                *value += 1;
            }
        }
    }

    result
}

/// Constructs the Interference Graph of a single Chain of Blocks
///
/// Params:
/// * outside_uses: A closure that should yield the number of uses of the given Variable in the
/// outer Scope (if it exists)
fn construct_chain<'c, I>(
    graph: &'c DirectedGraph<BasicBlock>,
    chain: &mut DirectedChain<'c, BasicBlock>,
    if_graph: &mut I,
    live_vars: &mut LiveVars,
    outside_uses: &dyn Fn(&Variable) -> usize,
) where
    I: InterferenceGraph,
{
    while let Some(entry) = chain.next_entry() {
        match entry {
            ChainEntry::Node(block) => {
                for stmnt in block.get_statements() {
                    for var in stmnt.used_vars() {
                        if live_vars.used_var(&var).is_err() {
                            dbg!(&var);
                        }
                    }

                    if let Statement::Assignment { target, .. } = stmnt {
                        let uses = outside_uses(&target);

                        if uses > 0 {
                            if_graph.add_node(target.clone());
                            for other in live_vars.iter() {
                                if_graph.add_edge(target.clone(), other.clone());
                            }

                            live_vars.add_var(target.clone(), uses);
                        } else {
                            println!("Variable has no uses: {:?}", target);
                        }
                    }
                }
            }
            ChainEntry::Branched {
                sides: (mut left, right),
                ..
            } => {
                match right {
                    Some(mut right) => {
                        let left_uses = count_uses(left.duplicate());
                        let right_uses = count_uses(right.duplicate());

                        let mut left_vars = live_vars.clone();
                        for (used, uses) in right_uses {
                            for _ in 0..uses {
                                let _ = left_vars.used_var(&used);
                            }
                        }
                        construct_chain(graph, &mut left, if_graph, &mut left_vars, outside_uses);

                        let mut right_vars = live_vars.clone();
                        for (used, uses) in left_uses {
                            for _ in 0..uses {
                                let _ = right_vars.used_var(&used);
                            }
                        }
                        construct_chain(graph, &mut right, if_graph, &mut right_vars, outside_uses);

                        live_vars.merge_branched(left_vars, right_vars);
                    }
                    None => {
                        construct_chain(graph, &mut left, if_graph, live_vars, outside_uses);
                    }
                };
            }
            ChainEntry::Cycle { mut inner, .. } => {
                let mut inner_vars = live_vars.clone();
                construct_chain(graph, &mut inner, if_graph, &mut inner_vars, outside_uses);

                *live_vars = inner_vars;
            }
        };
    }
}

fn total_uses(root_chain: DirectedChain<'_, BasicBlock>) -> HashMap<Variable, usize> {
    let mut result = HashMap::new();

    for stmnt in root_chain.flatten().flat_map(|b| b.get_statements()) {
        for used in stmnt.used_vars() {
            let entry = result.entry(used);
            let value = entry.or_default();
            *value += 1;
        }
    }

    result
}

/// Constructs the Interference Graph for the given Graph
pub fn construct<'g, C>(graph: C, result: &mut impl InterferenceGraph)
where
    C: Into<DirectedChain<'g, BasicBlock>>,
{
    let mut root_chain = graph.into();
    let graph = root_chain.graph();

    let total_uses = total_uses(root_chain.duplicate());

    let mut live_vars = LiveVars::new();
    construct_chain(
        graph,
        &mut root_chain,
        result,
        &mut live_vars,
        &|var| match total_uses.get(var) {
            Some(u) => *u,
            None => {
                println!("Getting total uses for {:?}", var);
                0
            }
        },
    );
}
