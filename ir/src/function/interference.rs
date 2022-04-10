use std::{collections::HashMap, env::VarError, panic};

use graphs::directed::{ChainEntry, DirectedChain, DirectedFlatChain, DirectedGraph};

use crate::{BasicBlock, DefaultInterferenceGraph, InterferenceGraph, Statement, Variable};

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
            println!("Var without uses: {:?}", var);
            return;
        }

        self.vars.insert(var, uses);
    }

    /// Decrements the use counter for the given Variable and removes it, if the count reaches 0
    pub fn used_var(&mut self, var: &Variable) {
        let count = match self.vars.get_mut(var) {
            Some(c) => c,
            None => {
                panic!("Used a Variable that is not known anymore: {:?}", var);
            }
        };

        *count = count.saturating_sub(1);

        if *count == 0 {
            self.vars.remove(var);
        }
    }

    pub fn merge_branched(&mut self, left: Self, right: Self) {
        let union_vars: HashMap<_, _> = self
            .vars
            .iter()
            .filter_map(|(var, count)| {
                let left_delta = count - left.vars.get(var)?;
                let right_delta = count - right.vars.get(var)?;

                let n_count = count - left_delta - right_delta;

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

    pub fn iter(&self) -> impl Iterator<Item = &Variable> + '_ {
        self.vars.keys()
    }
}

fn calc_uses<'g>(
    current_chain: DirectedChain<'g, BasicBlock>,
    var: &Variable,
    outside_uses: &dyn Fn(&Variable) -> usize,
) -> usize {
    let mut chain_result = 0;
    let mut flat_chain = DirectedFlatChain::new(current_chain);

    while let Some(block) = flat_chain.next_entry() {
        chain_result += block
            .get_statements()
            .into_iter()
            .filter(|s| s.used_vars().contains(var))
            .count();
    }

    let outside_result = outside_uses(var);

    chain_result + outside_result
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
    let mut prev_chain = chain.duplicate();
    while let Some(entry) = chain.next_entry() {
        match entry {
            ChainEntry::Node(block) => {
                for stmnt in block.get_statements() {
                    for var in stmnt.used_vars() {
                        live_vars.used_var(&var);
                    }

                    if let Statement::Assignment { target, .. } = stmnt {
                        let uses = calc_uses(prev_chain.duplicate(), &target, outside_uses);

                        if uses > 0 {
                            live_vars.add_var(target.clone(), uses);

                            if_graph.add_node(target.clone());
                            for other in live_vars.iter() {
                                if_graph.add_edge(target.clone(), other.clone());
                            }
                        } else {
                            println!("Variable has no uses: {:?}", target);
                        }
                    }
                }
            }
            ChainEntry::Branched {
                head,
                sides: (mut left, mut right),
            } => {
                let uses_func = |var: &Variable| {
                    let mut check_chain = prev_chain.duplicate();
                    let _ = check_chain.next_entry();

                    calc_uses(check_chain, var, outside_uses)
                };

                let mut left_vars = live_vars.clone();
                construct_chain(graph, &mut left, if_graph, &mut left_vars, &uses_func);

                let mut right_vars = live_vars.clone();
                construct_chain(graph, &mut right, if_graph, &mut right_vars, &uses_func);

                live_vars.merge_branched(left_vars, right_vars);
            }
            ChainEntry::Cycle { head, mut inner } => {
                let uses_func = |var: &Variable| {
                    let mut check_chain = prev_chain.duplicate();
                    let _ = check_chain.next_entry();

                    calc_uses(check_chain, var, outside_uses)
                };

                let mut inner_vars = live_vars.clone();
                construct_chain(graph, &mut inner, if_graph, &mut inner_vars, &uses_func);

                *live_vars = inner_vars;
            }
        };

        prev_chain = chain.duplicate();
    }
}

/// Constructs the Interference Graph for the given Graph
pub fn construct(graph: DirectedGraph<BasicBlock>, result: &mut impl InterferenceGraph) {
    let mut root_chain = graph.chain_iter();
    let mut live_vars = LiveVars {
        vars: HashMap::new(),
    };
    construct_chain(&graph, &mut root_chain, result, &mut live_vars, &|_| 0);
}
