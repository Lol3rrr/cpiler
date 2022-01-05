// This is build around this Paper
// https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use ir::Variable;

pub enum RegisterType {
    GeneralPurpose,
    FloatingPoint,
}

impl RegisterType {
    pub fn useable(&self, ty: &ir::Type) -> bool {
        match self {
            Self::GeneralPurpose => match ty {
                ir::Type::Float | ir::Type::Double | ir::Type::LongDouble => false,
                _ => true,
            },
            Self::FloatingPoint => match ty {
                ir::Type::Float | ir::Type::Double | ir::Type::LongDouble => true,
                _ => false,
            },
        }
    }
}

pub trait Register {
    fn reg_type(&self) -> RegisterType;
}

pub fn allocate_registers<R>(func: &ir::FunctionDefinition, registers: &[R]) -> HashMap<Variable, R>
where
    R: Clone + Debug + Hash + PartialEq + Eq + Register,
{
    let mut interference_graph = ir::DefaultInterferenceGraph::new();
    func.interference_graph(&mut interference_graph);

    let dominance_tree = func.dominance_tree();

    let mut coloring = HashMap::new();

    for current in dominance_tree.post_order_iter() {
        let neighbours = interference_graph.neighbours(&current);

        let used_colors: HashSet<_> = neighbours
            .into_iter()
            .map(|n| coloring.get(n.var()).cloned())
            .filter_map(|n| n)
            .collect();

        let avail_colors: Vec<_> = registers
            .iter()
            .filter(|r| r.reg_type().useable(&current.ty))
            .filter(|r| !used_colors.contains(*r))
            .collect();

        let used_color = avail_colors.into_iter().next().unwrap();

        coloring.insert(current, used_color.clone());
    }

    coloring
}
