#![warn(missing_docs)]
//! The actual Register Allocation with Graph-Coloring is based on [this Paper](https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf)

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

mod context;
mod determine_spill;
mod spill;

/// The Types of Registers that can be allocated
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RegisterType {
    /// A General Purpose Register that will be used for most operations
    GeneralPurpose,
    /// A Floating Point Register used for Floating Point or sometimes SIMD/Vector operations
    FloatingPoint,
}

impl RegisterType {
    /// Checks if the given Type is useable with the current Register Type
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

/// This Trait abstracts away the actual Register Type used as long as it can be converted into a
/// Register Type
pub trait Register {
    /// The corresponding RegisterType for the Register
    fn reg_type(&self) -> RegisterType;
    /// The Size and Alignment of the Register
    fn align_size(&self) -> (usize, usize);
}

/// The Mapping of Variables to Registers
#[derive(Debug, PartialEq, Clone)]
pub struct RegisterMapping<R> {
    inner: HashMap<ir::Variable, R>,
}

impl<R> RegisterMapping<R>
where
    R: Clone + Hash + PartialEq + Eq + Register,
{
    /// Actually performs the Register allocation
    pub fn allocate(func: &ir::FunctionDefinition, registers: &[R]) -> Self {
        let mut spill_count = 0;
        let interference_graph = loop {
            let mut interference_graph = ir::DefaultInterferenceGraph::new();
            let mut too_large_clique = None;
            func.interference_graph(&mut interference_graph, |live, block, index| {
                if too_large_clique.is_some() {
                    return;
                }

                let mut available_registers: HashMap<RegisterType, isize> = HashMap::new();
                for reg in registers {
                    let reg_type = reg.reg_type();

                    let reg_avail = available_registers.entry(reg_type).or_insert(0);
                    *reg_avail += 1;
                }

                for var in live.iter() {
                    let useable_reg = registers
                        .iter()
                        .filter(|r| r.reg_type().useable(&var.ty))
                        .map(|r| r.reg_type())
                        .next()
                        .unwrap();

                    let regs_avail = available_registers.get_mut(&useable_reg).unwrap();
                    *regs_avail -= 1;
                }

                for (reg, available) in available_registers {
                    if available < 0 {
                        too_large_clique = Some((live.clone(), block.clone(), index));
                        return;
                    }
                }
            });

            let (largest_vars, largest_block, largest_stmnt_i) = match too_large_clique {
                Some(l) => l,
                None => break interference_graph,
            };

            let spill_ctx = context::SpillContext::determine(largest_block.clone());

            spill::spill_variable(
                largest_vars,
                largest_block.clone(),
                largest_stmnt_i,
                spill_ctx,
            );

            spill_count += 1;

            if spill_count > 5 {
                dbg!(&largest_block);
                panic!("Spilled more than five times in a single Function");
            }
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

                    todo!("Not enough Registers available")
                }
            };

            coloring.insert(current, used_color.clone());
        }

        RegisterMapping { inner: coloring }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    enum TestRegister {
        General(u8),
        Float(u8),
    }

    impl Register for TestRegister {
        fn reg_type(&self) -> RegisterType {
            match self {
                Self::General(_) => RegisterType::GeneralPurpose,
                Self::Float(_) => RegisterType::FloatingPoint,
            }
        }

        fn align_size(&self) -> (usize, usize) {
            (4, 4)
        }
    }

    #[test]
    fn fits() {
        let input_register = vec![TestRegister::General(0)];
        let input_statements = vec![ir::Statement::Assignment {
            target: ir::Variable::new("test", ir::Type::U8),
            value: ir::Value::Unknown,
        }];

        let input_function = ir::FunctionDefinition {
            name: "test".to_string(),
            block: ir::BasicBlock::new(vec![], input_statements.clone()),
            arguments: vec![],
            return_ty: ir::Type::Void,
        };

        let result = RegisterMapping::allocate(&input_function, &input_register);
    }
}
