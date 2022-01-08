use std::collections::{BTreeMap, HashMap, HashSet};

use crate::backends::aarch64_mac::{asm, codegen, ArmRegister};

fn ty_size_align(ty: &ir::Type) -> (usize, usize) {
    match ty {
        ir::Type::I32 => (4, 4),
        ir::Type::I64 => (8, 8),
        ir::Type::Pointer(_) => (8, 8),
        other => {
            dbg!(&other);
            todo!()
        }
    }
}

fn vars_used(start: &ir::BasicBlock) -> BTreeMap<String, (usize, usize)> {
    let mut result = BTreeMap::new();

    for block in start.block_iter() {
        let statements = block.get_statements();

        for stmnt in statements {
            if let ir::Statement::Assignment { target, .. } = stmnt {
                result.insert(target.name, ty_size_align(&target.ty));
            }
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

#[derive(Debug)]
pub struct StackAllocation {
    pub setup_instr: Vec<asm::Instruction>,
    pub pre_return_instr: Vec<asm::Instruction>,
    pub var_offsets: HashMap<String, isize>,
    pub allocations: HashMap<ir::Variable, isize>,
}

pub fn allocate_stack(
    func: &ir::FunctionDefinition,
    reg_map: &HashMap<ir::Variable, ArmRegister>,
) -> StackAllocation {
    let used_registers: HashSet<_> = reg_map.iter().map(|(_, r)| r.clone()).collect();

    let raw_vars = vars_used(&func.block);

    let raw_allocations = allocations(&func.block);

    let alloc_iter = used_registers
        .iter()
        .map(|r| match r {
            ArmRegister::GeneralPurpose(_) => (8, 8),
            ArmRegister::FloatingPoint(_) => todo!("Size and Alignemnt of Float Register"),
        })
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

    let stack_space = codegen::stack_space(alloc_iter);

    let mut setup = vec![asm::Instruction::StpPreIndex {
        first: asm::GPRegister::DWord(29),
        second: asm::GPRegister::DWord(30),
        base: asm::GpOrSpRegister::SP,
        offset: -(stack_space as i16),
    }];

    let mut pre_ret_instr = Vec::new();

    let start_base = {
        let base: i16 = 16;
        for (index, raw_reg) in used_registers.iter().enumerate() {
            let offset = base + (index as i16) * 8;

            let reg = match raw_reg {
                ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                ArmRegister::FloatingPoint(_) => todo!(),
            };

            let store_instr = asm::Instruction::StoreRegisterUnscaled {
                reg: reg.clone(),
                base: asm::GpOrSpRegister::SP,
                offset,
            };
            let load_instr = asm::Instruction::LoadRegisterUnscaled {
                reg: reg.clone(),
                base: asm::GpOrSpRegister::SP,
                offset,
            };

            setup.push(store_instr);
            pre_ret_instr.push(load_instr);
        }

        base + (used_registers.len() as i16) * 8
    };

    pre_ret_instr.push(asm::Instruction::LdpPostIndex {
        first: asm::GPRegister::DWord(29),
        second: asm::GPRegister::DWord(30),
        base: asm::GpOrSpRegister::SP,
        offset: stack_space as i16,
    });

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
            dbg!(&var, &alignment, &size);

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
