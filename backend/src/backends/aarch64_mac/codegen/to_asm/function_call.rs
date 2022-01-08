use crate::backends::aarch64_mac::{
    asm,
    codegen::{self, ArgTarget, Context},
    ArmRegister,
};

pub fn to_asm(
    name: String,
    arguments: Vec<ir::Operand>,
    ret_ty: ir::Type,
    target: Option<asm::GPRegister>,
    ctx: &Context,
    instructions: &mut Vec<asm::Instruction>,
) {
    dbg!(&name, &arguments, &ret_ty);

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
        dbg!(&arg, &mapped);
    }

    // 3.
    instructions.push(asm::Instruction::BranchLinkLabel {
        target: name.clone(),
    });

    // 4.
    if let Some(target) = target {
        let mut res_mapping = codegen::arguments(std::iter::once(ret_ty.clone()));
        let res_target = res_mapping.remove(0);
        match res_target {
            ArgTarget::GPRegister(reg) => {
                instructions.push(asm::Instruction::MovRegister {
                    dest: target,
                    src: asm::GPRegister::DWord(reg),
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
