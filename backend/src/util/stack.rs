//! Contains all the general Code for handling the Stack of functions

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
};

use register_allocation::Register;

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

fn vars_used<TAS>(start: &ir::BasicBlock, size_align: TAS) -> BTreeMap<String, (usize, usize)>
where
    TAS: Fn(&ir::Type) -> (usize, usize),
{
    let mut result = BTreeMap::new();

    for block in start.block_iter() {
        let statements = block.get_statements();

        for stmnt in statements {
            match stmnt {
                ir::Statement::Assignment { target, .. } if !target.is_tmp() => {
                    result.insert(target.name, size_align(&target.ty));
                }
                _ => {}
            };
        }
    }

    result
}

fn allocations(start: &ir::BasicBlock) -> BTreeMap<ir::Variable, (usize, usize)> {
    let mut result = BTreeMap::new();

    for block in start.block_iter() {
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
    pub var_offsets: HashMap<String, isize>,
    pub allocations: HashMap<ir::Variable, isize>,
}

/// This performs the corresponding allocation of the Stack for the given Function
pub fn allocate_stack<I, R, ASF, DSF, SS, LS, TAS>(
    // The function
    func: &ir::FunctionDefinition,
    reg_map: &HashMap<ir::Variable, R>,
    // A closure that returns the instructions for allocating the given space in bytes on the Stack
    alloc_space: ASF,
    // A closure that returns the instructions for deallocating the given space in bytes on the Stack
    dealloc_space: DSF,
    // A closure that returns the instructions to store a Register on the Stack at the given offset
    store_on_stack: SS,
    // A closure that returns the instructions to load a Value from the Stack into a Register
    load_from_stack: LS,
    // A closure that returns the alignment and size of the given Type
    type_align_size: TAS,
    // The Alignment that the stack needs on the Platform
    stack_alignment: usize,
    stack_base: usize,
) -> StackAllocation<I>
where
    R: Register + Hash + Eq,
    ASF: FnOnce(usize) -> Vec<I>,
    DSF: FnOnce(usize) -> Vec<I>,
    SS: Fn(&R, i16) -> Vec<I>,
    LS: Fn(&R, i16) -> Vec<I>,
    TAS: Fn(&ir::Type) -> (usize, usize),
{
    let used_registers: HashSet<_> = reg_map.iter().map(|(_, r)| r).collect();

    let raw_vars = vars_used(&func.block, type_align_size);

    let raw_allocations = allocations(&func.block);

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

    let stack_space = stack_space(alloc_iter, stack_base, stack_alignment);

    let mut setup = Vec::new();
    let stack_alloc = alloc_space(stack_space);
    setup.extend(stack_alloc);

    let mut pre_ret_instr = Vec::new();

    let start_base = {
        let mut current_base = stack_base as i16;
        for (_, raw_reg) in used_registers.iter().enumerate() {
            let (reg_align, reg_size) = raw_reg.align_size();
            let (reg_align, reg_size) = (reg_align as i16, reg_size as i16);

            if current_base % reg_align != 0 {
                current_base += reg_align - (current_base % reg_align);
            }

            let store_instr = store_on_stack(raw_reg, current_base);
            let load_instr = load_from_stack(raw_reg, current_base);

            setup.extend(store_instr);
            pre_ret_instr.extend(load_instr);

            current_base += reg_size;
        }

        current_base
    };

    pre_ret_instr.extend(dealloc_space(stack_space));

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
