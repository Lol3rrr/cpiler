use crate::backends::aarch64_mac::{
    asm,
    codegen::{self, ArgTarget, Context},
};

pub fn to_asm(
    name: String,
    arguments: Vec<ir::Operand>,
    ret_ty: ir::Type,
    target: Option<asm::Register>,
    ctx: &Context,
    instructions: &mut Vec<asm::Instruction>,
) {
    // 1. store the arguments to the current function on the Stack
    // 2. load all the arguments into the argument registers/etc.
    // 3. call the function
    // 4. move the result into the target
    // 5. restore previous arguments from stack

    let mut offset = 0;
    let arg_regs = [
        asm::GPRegister::DWord(0),
        asm::GPRegister::DWord(1),
        asm::GPRegister::DWord(2),
        asm::GPRegister::DWord(3),
        asm::GPRegister::DWord(4),
        asm::GPRegister::DWord(5),
        asm::GPRegister::DWord(6),
        asm::GPRegister::DWord(7),
    ];
    // 1.
    for chunk in arg_regs.chunks(2) {
        let first = chunk[0].clone();
        let second = chunk[1].clone();

        instructions.push(asm::Instruction::StpPreIndex {
            first,
            second,
            offset: -16,
            base: asm::GpOrSpRegister::SP,
        });
        offset += 16;
    }

    // 2.
    let arg_mapping = codegen::arguments(arguments.iter().map(|a| a.ty()));
    for (arg, mapped) in arguments.into_iter().zip(arg_mapping) {
        let (arg_src, dword_sized) = match arg {
            ir::Operand::Variable(var) => match ctx.registers.get_reg(&var).unwrap() {
                asm::Register::GeneralPurpose(gp) => {
                    (gp.clone(), matches!(gp, asm::GPRegister::DWord(_)))
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            },
            ir::Operand::Constant(con) => {
                dbg!(&con);
                todo!()
            }
        };

        match mapped {
            ArgTarget::GPRegister(n) => {
                let target_reg = if dword_sized {
                    asm::GPRegister::DWord(n)
                } else {
                    asm::GPRegister::Word(n)
                };

                instructions.push(asm::Instruction::MovRegister {
                    dest: target_reg,
                    src: arg_src,
                });
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    // 3.
    instructions.push(asm::Instruction::BranchLinkLabel { target: name });

    // 4.
    if let Some(target) = target {
        let mut res_mapping = codegen::arguments(std::iter::once(ret_ty));
        let res_target = res_mapping.remove(0);
        match (target, res_target) {
            (asm::Register::GeneralPurpose(target), ArgTarget::GPRegister(reg)) => {
                let src_reg = match &target {
                    asm::GPRegister::DWord(_) => asm::GPRegister::DWord(reg),
                    asm::GPRegister::Word(_) => asm::GPRegister::Word(reg),
                };

                instructions.push(asm::Instruction::MovRegister {
                    dest: target,
                    src: src_reg,
                });
            }
            (asm::Register::FloatingPoint(target), ArgTarget::FPRegister(reg)) => {
                let src_reg = match &target {
                    asm::FPRegister::SinglePrecision(_) => asm::FPRegister::SinglePrecision(reg),
                    asm::FPRegister::DoublePrecision(_) => asm::FPRegister::DoublePrecision(reg),
                };

                instructions.push(asm::Instruction::FMovRegister {
                    dest: target,
                    src: src_reg,
                });
            }
            other => {
                dbg!(&other);
                todo!()
            }
        };
    }

    // 5.
    for chunk in arg_regs.chunks(2).rev() {
        let first = chunk[0].clone();
        let second = chunk[1].clone();

        instructions.push(asm::Instruction::LdpPostIndex {
            first,
            second,
            offset: 16,
            base: asm::GpOrSpRegister::SP,
        });
    }
}
