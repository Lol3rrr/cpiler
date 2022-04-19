//! Contains all the general Code for handling the Stack of functions

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
};

use register_allocation::Register;

/// Determines the space needed on the Stack for the given Iterator of Allocations
fn stack_space<ISI, IS>(allocations: ISI, base: usize, forced_alignment: usize) -> usize
where
    ISI: IntoIterator<IntoIter = IS, Item = (usize, usize)>,
    IS: Iterator<Item = (usize, usize)>,
{
    let mut base = base;

    for (align, size) in allocations.into_iter() {
        if base % align != 0 {
            base += align - (base % align);
        }

        base += size;
    }

    if base % forced_alignment == 0 {
        base
    } else {
        base + (forced_alignment - (base % forced_alignment))
    }
}

/// Determines the Variables that need to be Allocate on the Space/need space on the Stack for
/// spilling or the like
fn vars_used<TAS>(
    graph: &graphs::directed::DirectedGraph<ir::BasicBlock>,
    size_align: TAS,
) -> BTreeMap<String, (usize, usize)>
where
    TAS: Fn(&ir::Type) -> (usize, usize),
{
    let mut result = BTreeMap::new();

    for block in graph.chain_iter().flatten() {
        let statements = block.get_statements();

        for stmnt in statements {
            let used_vars: Box<dyn Iterator<Item = ir::Variable>> = match stmnt {
                ir::Statement::SaveVariable { var } => Box::new(std::iter::once(var)),
                ir::Statement::Assignment {
                    value: ir::Value::Unknown,
                    target,
                } => Box::new(std::iter::once(target)),
                ir::Statement::Assignment {
                    target,
                    value: ir::Value::Phi { .. },
                } => Box::new(std::iter::once(target)),
                _ => Box::new(std::iter::empty()),
            };
            for var in used_vars {
                result.insert(var.name, size_align(&var.ty));
            }
        }
    }

    result
}

/// Determine the number of Allocations needed
fn allocations(
    graph: &graphs::directed::DirectedGraph<ir::BasicBlock>,
) -> BTreeMap<ir::Variable, (usize, usize)> {
    // The Resulting Map of Allocations
    let mut result = BTreeMap::new();

    // Iterate over the Blocks in the Graph
    for block in graph.chain_iter().flatten() {
        let statements = block.get_statements();

        for stmnt in statements {
            if let ir::Statement::Assignment {
                target,
                value: ir::Value::Expression(ir::Expression::StackAlloc { size, alignment }),
            } = stmnt
            {
                result.insert(target, (alignment, size));
            }
        }
    }

    result
}

/// Describes the Stack that should be allocated
#[derive(Debug)]
pub struct StackAllocation<I> {
    /// The Instructions to setup the Stack, moving the Stack Pointer and all that
    pub setup_instr: Vec<I>,
    /// The Instructions that should be run before returning from the function to make sure that
    /// the Stack gets reset properly
    pub pre_return_instr: Vec<I>,
    /// The Offsets for all the Variables by name
    pub var_offsets: HashMap<String, isize>,
    /// The Offsets for certain Structures like arrays
    pub allocations: HashMap<ir::Variable, isize>,
}

/// The Configuration to use for the Stack-Allocation procedure
pub struct AllocateConfig<A, DA, SS, LS, TA, I, R>
where
    A: FnOnce(usize) -> Vec<I>,
    DA: FnOnce(usize) -> Vec<I>,
    SS: for<'r> Fn(&'r R, i16) -> Vec<I>,
    LS: for<'r> Fn(&'r R, i16) -> Vec<I>,
    TA: for<'t> Fn(&'t ir::Type) -> (usize, usize),
{
    /// Used for generating the Instructions to allocate the given space in Bytes on the Stack
    pub alloc_space: A,
    /// Used for generating the Instructions to deallocate the given Space in Bytes on the Stack
    pub dealloc_space: DA,
    /// Generates the Instructions to store the Register on the Stack at the provided Offset
    pub store_on_stack: SS,
    /// Generates the Instructions to load the Register from the Stack at the provided Offset
    pub load_on_stack: LS,
    /// Figures out the Size and Alignment for a given Type
    pub type_align_size: TA,
    /// TODO
    /// Figure this out
    pub stack_base: usize,
    /// The Alignment for the Stack-Pointer
    pub stack_alignment: usize,
    /// Needed for the R-Type
    pub _marker: PhantomData<R>,
}

/// This performs the corresponding allocation of the Stack for the given Function
pub fn allocate_stack<I, R, ASF, DSF, SS, LS, TAS>(
    func: &ir::FunctionDefinition,
    reg_map: &HashMap<ir::Variable, R>,
    conf: AllocateConfig<ASF, DSF, SS, LS, TAS, I, R>,
) -> StackAllocation<I>
where
    R: Register + Hash + Eq,
    ASF: FnOnce(usize) -> Vec<I>,
    DSF: FnOnce(usize) -> Vec<I>,
    SS: for<'r> Fn(&'r R, i16) -> Vec<I>,
    LS: for<'r> Fn(&'r R, i16) -> Vec<I>,
    TAS: for<'t> Fn(&'t ir::Type) -> (usize, usize),
{
    let func_graph = func.to_directed_graph();

    // Determine the used Registers
    let used_registers: HashSet<_> = reg_map.iter().map(|(_, r)| r).collect();

    // Determine the used Variables
    let raw_vars = vars_used(&func_graph, conf.type_align_size);

    // Determine the raw-number of allocations
    let raw_allocations = allocations(&func_graph);

    let alloc_iter = used_registers
        .iter()
        .map(|r| r.align_size())
        .chain(
            raw_vars
                .iter()
                .map(|(_, (alignment, size))| (*alignment, *size)),
        )
        .chain(
            raw_allocations
                .iter()
                .map(|(_, (alignment, size))| (*alignment, *size)),
        );

    // Determine the Space needed on the Stack for the provided Allocations
    let stack_space = stack_space(alloc_iter, conf.stack_base, conf.stack_alignment);

    // Generate the correct Setup for the given Stack-Space
    let mut setup = Vec::new();
    let stack_alloc = (conf.alloc_space)(stack_space);
    setup.extend(stack_alloc);

    let mut pre_ret_instr = Vec::new();

    let start_base = {
        let mut current_base = conf.stack_base as i16;
        for (_, raw_reg) in used_registers.iter().enumerate() {
            let (reg_align, reg_size) = raw_reg.align_size();
            let (reg_align, reg_size) = (reg_align as i16, reg_size as i16);

            if current_base % reg_align != 0 {
                current_base += reg_align - (current_base % reg_align);
            }

            let store_instr = (conf.store_on_stack)(raw_reg, current_base);
            let load_instr = (conf.load_on_stack)(raw_reg, current_base);

            setup.extend(store_instr);
            pre_ret_instr.extend(load_instr);

            current_base += reg_size;
        }

        current_base
    };

    pre_ret_instr.extend((conf.dealloc_space)(stack_space));

    let (var_offsets, start_base) = {
        let mut tmp = HashMap::new();

        let mut base = start_base;
        for (var, (alignment, size)) in raw_vars.iter() {
            let alignment = *alignment as i16;
            let size = *size as i16;

            if base % alignment != 0 {
                base += alignment - (base % alignment);
            }

            let offset = base;
            tmp.insert(var.clone(), offset as isize);

            base += size;
        }

        (tmp, base)
    };

    let allocations = {
        let mut tmp = HashMap::new();

        let mut base = start_base;
        for (var, (alignment, size)) in raw_allocations.iter() {
            let alignment = *alignment as i16;
            let size = *size as i16;

            if base % alignment != 0 {
                base += alignment - (base % alignment);
            }

            let offset = base;
            tmp.insert(var.clone(), offset as isize);

            base += size;
        }

        tmp
    };

    StackAllocation {
        setup_instr: setup,
        pre_return_instr: pre_ret_instr,
        var_offsets,
        allocations,
    }
}
