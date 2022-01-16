// This is build around this Paper
// https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use ir::Variable;

mod spill;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RegisterType {
    GeneralPurpose,
    FloatingPoint,
}

impl RegisterType {
    pub fn useable(&self, ty: &ir::Type) -> bool {
        match self {
            Self::GeneralPurpose => !matches!(
                ty,
                ir::Type::Float | ir::Type::Double | ir::Type::LongDouble
            ),
            Self::FloatingPoint => matches!(
                ty,
                ir::Type::Float | ir::Type::Double | ir::Type::LongDouble
            ),
        }
    }
}

pub trait Register {
    fn reg_type(&self) -> RegisterType;
    fn align_size(&self) -> (usize, usize);
}

fn determine_spill_var(
    vars: HashSet<Variable>,
    start_block: ir::BasicBlock,
    start_index: usize,
) -> ir::Variable {
    let mut unknown = vars;

    let mut offset = 0;
    let mut statements: Vec<_> = start_block
        .get_statements()
        .into_iter()
        .skip(start_index)
        .collect();
    let mut block = start_block;
    let mut offsets = Vec::new();
    for (index, stmnt) in statements.iter().enumerate() {
        let distance = index + offset;

        let used = stmnt.used_vars();

        let used_unknowns: Vec<_> = used.into_iter().filter(|v| unknown.contains(v)).collect();

        for u_v in used_unknowns {
            unknown.remove(&u_v);
            offsets.push((u_v, distance));
        }
    }

    offset += statements.len();

    while !unknown.is_empty() {
        dbg!(&unknown);
        todo!()
    }

    let first = offsets
        .into_iter()
        .max_by(|(_, a_d), (_, b_d)| a_d.cmp(b_d))
        .unwrap();

    first.0
}

pub fn allocate_registers<R>(func: &ir::FunctionDefinition, registers: &[R]) -> HashMap<Variable, R>
where
    R: Clone + Debug + Hash + PartialEq + Eq + Register,
{
    let interference_graph = loop {
        let mut interference_graph = ir::DefaultInterferenceGraph::new();
        let mut too_large_clique = None;
        func.interference_graph(&mut interference_graph, |live, block, index| {
            if too_large_clique.is_some() {
                return;
            }

            let mut used: HashSet<R> = HashSet::new();
            let mut available: Vec<_> = registers.iter().collect();

            for var in live.iter() {
                let avail_colors: Vec<_> = registers
                    .iter()
                    .filter(|r| r.reg_type().useable(&var.ty))
                    .filter(|r| !used.contains(r))
                    .collect();

                match avail_colors.first() {
                    Some(f) => {
                        used.insert((*f).clone());
                        available.remove(0);
                    }
                    None => {
                        too_large_clique = Some((live.clone(), block.clone(), index));
                        dbg!("Too large");
                        return;
                    }
                };
            }

            let mut reg_type_free = HashSet::new();
            for tmp in registers.iter().filter(|r| !used.contains(r)) {
                reg_type_free.insert(tmp.reg_type());
            }

            if reg_type_free.len() != 2 {
                too_large_clique = Some((live.clone(), block.clone(), index));
            }
        });

        let (largest_vars, largest_block, largest_stmnt_i) = match too_large_clique {
            Some(l) => l,
            None => break interference_graph,
        };

        let spill_var = determine_spill_var(largest_vars, largest_block.clone(), largest_stmnt_i);

        spill::spill_variable(spill_var, largest_block, largest_stmnt_i);
    };

    let dominance_tree = func.dominance_tree();

    let mut coloring = HashMap::new();

    for current in dominance_tree.post_order_iter() {
        let neighbours = interference_graph.neighbours(&current);

        let used_colors: HashSet<_> = neighbours
            .iter()
            .cloned()
            .filter_map(|n| coloring.get(n.var()).cloned())
            .collect();

        let mut avail_colors = registers
            .iter()
            .filter(|r| r.reg_type().useable(&current.ty))
            .filter(|r| !used_colors.contains(*r));

        let used_color = match avail_colors.next() {
            Some(c) => c,
            None => {
                dbg!(&current, &neighbours);

                std::fs::write("./failed_registers.dot", interference_graph.to_dot()).unwrap();

                todo!()
            }
        };

        coloring.insert(current, used_color.clone());
    }

    coloring
}
