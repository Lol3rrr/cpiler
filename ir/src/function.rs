use std::{collections::HashSet, fmt::Debug};

use crate::{BasicBlock, DominanceTree, InterferenceGraph, ToDot, Type, Variable};

mod debug;
use debug::DebugBlocks;

mod interference;

/// A definition of a Function
#[derive(Clone, PartialEq)]
pub struct FunctionDefinition {
    /// The Name of the Function
    pub name: String,
    /// The Arguments of the Function in the Order they will be received in
    pub arguments: Vec<(String, Type)>,
    /// The initial BasicBlock of the Function
    pub block: BasicBlock,
    /// The Return Type of the Function
    pub return_ty: Type,
}

impl Debug for FunctionDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f_struct = f.debug_struct("FunctionDefinition");

        let dbg_blocks = DebugBlocks {
            start: self.block.clone(),
        };

        f_struct.field("arguments", &self.arguments);
        f_struct.field("return_ty", &self.return_ty);
        f_struct.field("blocks", &dbg_blocks);

        f_struct.finish()?;

        Ok(())
    }
}

impl ToDot for FunctionDefinition {
    fn to_dot(
        &self,
        lines: &mut dyn graphviz::Graph,
        drawn: &mut crate::dot::DrawnBlocks,
        ctx: &crate::dot::Context,
    ) -> String {
        let dot_name = format!("func_{}", self.name);
        let mut function_graph = graphviz::SubGraph::new(&dot_name)
            .cluster()
            .arg("label", format!("Function-{}", self.name));

        let block_name = self.block.to_dot(&mut function_graph, drawn, ctx);
        lines.add_subgraph(function_graph);

        lines.add_edge(graphviz::Edge::new(&dot_name, block_name));

        dot_name
    }

    fn name(&self, _: &crate::dot::Context) -> String {
        format!("func_{}", self.name)
    }
}

impl FunctionDefinition {
    /// This is used generate the Interference Graph for a given Function
    pub fn interference_graph<T, F>(&self, graph: &mut T, mut update: F)
    where
        T: InterferenceGraph,
        F: FnMut(&HashSet<Variable>, &BasicBlock, usize),
    {
        let g = self.to_directed_graph();
        interference::construct(g, graph);
        return;

        self.block
            .interference_graph(graph, &mut HashSet::new(), &mut HashSet::new(), &mut update);
    }

    /// Generates the Dominance Tree for this Function
    pub fn dominance_tree(&self) -> DominanceTree {
        self.block.dominance_tree(&mut HashSet::new())
    }

    /// Converts the Function to a Directed Graph for easier Processing and handling
    pub fn to_directed_graph(&self) -> graphs::directed::DirectedGraph<BasicBlock> {
        let mut graph = graphs::directed::DirectedGraph::new();

        for block in self.block.block_iter() {
            graph.add_node(block);
        }

        graph
    }
}
