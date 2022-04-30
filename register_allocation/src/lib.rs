#![warn(missing_docs)]
//! The actual Register Allocation with Graph-Coloring is based on [this Paper](https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf)

// TODO
// The Register allocator should reuse the same Registers for Variables that are combined using Phi nodes

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
};

use spilling::RegisterConfig;
mod spilling;

// mod phi_classes;

mod reconstruct;

mod debug_ctx;
use debug_ctx::DebugContext;

/// The Statement to use for saving the provided Variable
pub(crate) fn save_statement(var: ir::Variable) -> ir::Statement {
    if var.global() {
        ir::Statement::SaveGlobalVariable { var }
    } else {
        ir::Statement::SaveVariable { var }
    }
}

/// The Statement to use for loading the provided Variable
pub(crate) fn load_statement(var: ir::Variable) -> ir::Statement {
    if var.global() {
        let global_name = var.name().to_string();
        ir::Statement::Assignment {
            target: var,
            value: ir::Value::Expression(ir::Expression::ReadGlobalVariable { name: global_name }),
        }
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

/// Provides a Context to the Register Allocator to pass around different Information that may
/// be needed
pub struct AllocationCtx {
    /// The Build-Path for the current compilation
    pub build_path: Option<PathBuf>,
}

/// The Mapping of Variables to Registers
#[derive(Debug, PartialEq, Clone)]
pub struct RegisterMapping<R> {
    /// The internal Mapping from Variable to Register
    inner: HashMap<ir::Variable, R>,
}

impl<R> RegisterMapping<R>
where
    R: Clone + Hash + PartialEq + Eq + Register,
{
    fn color(
        func: &ir::FunctionDefinition,
        registers: &[R],
        ctx: &AllocationCtx,
    ) -> HashMap<ir::Variable, R> {
        // Build an interference Graph for the final Function
        println!("After spilling");
        let mut interference_graph = ir::DefaultInterferenceGraph::new();
        func.interference_graph(&mut interference_graph);

        // Build a dominance Tree for the final Function
        println!("Dominance Tree");
        let dominance_tree = func.dominance_tree();

        let mut coloring = HashMap::new();

        for current in dominance_tree.post_order_iter() {
            //if let Some(reg) = groups.get_group(&current) {
            //    coloring.insert(current, R::clone(reg));
            //    continue;
            //}

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

                    eprintln!("{}", ir::text_rep::generate_text_rep(func));

                    if let Some(dbg_path) = &ctx.build_path {
                        let mut debug_interference_g = ir::DefaultInterferenceGraph::new();
                        func.interference_graph(&mut debug_interference_g);

                        let graph_dot = debug_interference_g.to_dot();
                        let graph_path = dbg_path.join("reg-int.dot");
                        std::fs::write(graph_path, graph_dot).unwrap();
                    }

                    todo!("Not enough Registers available")
                }
            };

            coloring.insert(current.clone(), used_color.clone());
            //groups.set_group(current, used_color.clone());
        }

        coloring
    }

    /// Actually performs the Register allocation
    pub fn allocate(func: &ir::FunctionDefinition, registers: &[R], ctx: AllocationCtx) -> Self {
        println!("Allocating Function: {:?}", func.name);

        // Setup the Debug Context
        let mut debug_context = DebugContext::new();
        debug_context.add_state(func);

        // Determine the Number of Floating-Point and General-Purpose Registers available to the Allocator
        let float_registers = registers
            .iter()
            .filter(|r| matches!(r.reg_type(), RegisterType::FloatingPoint))
            .count();
        let general_registers = registers
            .iter()
            .filter(|r| matches!(r.reg_type(), RegisterType::GeneralPurpose))
            .count();

        // Perform the spilling for the Function to ensure that the number of Registers is not exceeded
        spilling::spill(
            func,
            RegisterConfig {
                general_purpose_count: general_registers.saturating_sub(1),
                floating_point_count: float_registers.saturating_sub(1),
            },
            &mut debug_context,
        );

        // TODO
        // For testing currently
        func.verify();

        eprintln!("{}", ir::text_rep::generate_text_rep(func));

        // Run a single Optimizer-Pass to remove all unused Variables
        let mut opt_config = optimizer::Config::new();
        opt_config.add_pass(optimizer::optimizations::DeadCode::new());
        let func = optimizer::optimize_func(func.clone(), &opt_config);

        // Attempt to write the now cleaned up IR to a File in the Build-Directory
        if let Some(dbg_path) = ctx.build_path.as_ref() {
            let func_ir_text = ir::text_rep::generate_text_rep(&func);
            let func_opt_path = dbg_path.join(format!("opt-{}-ir.ir", func.name));
            std::fs::write(func_opt_path, func_ir_text).expect("Save optimized spilled IR to text");
        }

        /*
        debug_context
            .get_steps()
            .for_each(|s| println!("{:?}\n", s));
            */

        let coloring = Self::color(&func, registers, &ctx);

        RegisterMapping { inner: coloring }
    }
}

impl<R> From<RegisterMapping<R>> for HashMap<ir::Variable, R> {
    fn from(other: RegisterMapping<R>) -> Self {
        other.inner
    }
}
