#![warn(missing_docs)]
//! The actual Register Allocation with Graph-Coloring is based on [this Paper](https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf)

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use spilling::RegisterConfig;

//mod context;
//mod determine_spill;
//mod spill;
mod spilling;

pub(crate) fn save_statement(var: ir::Variable) -> ir::Statement {
    if var.global() {
        todo!()
    } else {
        ir::Statement::SaveVariable { var }
    }
}

pub(crate) fn load_statement(var: ir::Variable) -> ir::Statement {
    if var.global() {
        todo!()
    } else {
        ir::Statement::Assignment {
            target: var,
            value: ir::Value::Unknown,
        }
    }
}

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
        // TODO
        // Instead of registers.len() we should calculate the correct Number of available
        // registers
        let float_registers = registers
            .iter()
            .filter(|r| matches!(r.reg_type(), RegisterType::FloatingPoint))
            .count();
        let general_registers = registers
            .iter()
            .filter(|r| matches!(r.reg_type(), RegisterType::GeneralPurpose))
            .count();
        spilling::spill(
            func.block.clone(),
            RegisterConfig {
                general_purpose_count: general_registers - 1,
                floating_point_count: float_registers - 1,
            },
        );

        let mut interference_graph = ir::DefaultInterferenceGraph::new();
        func.interference_graph(&mut interference_graph, |_, _, _| {});

        let dominance_tree = func.dominance_tree();

        let mut coloring = HashMap::new();

        for current in dominance_tree.post_order_iter() {
            let neighbours = interference_graph.neighbours(&current);

            let used_colors: HashSet<_> = neighbours
                .iter()
                .cloned()
                .filter_map(|n| coloring.get(n.var()).cloned())
                .collect();

            dbg!(used_colors.len());

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

impl<R> Into<HashMap<ir::Variable, R>> for RegisterMapping<R> {
    fn into(self) -> HashMap<ir::Variable, R> {
        self.inner
    }
}
