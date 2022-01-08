use crate::backends::aarch64_mac::{asm, codegen::Context, ArmRegister};

pub fn to_asm(
    op: ir::UnaryOp,
    t_reg: asm::GPRegister,
    base: ir::Operand,
    ctx: &Context,
    instr: &mut Vec<asm::Instruction>,
) {
    let (base_reg, imm_reg) = match base {
        ir::Operand::Variable(base_var) => match ctx.registers.get(&base_var).unwrap() {
            ArmRegister::GeneralPurpose(n) => {
                (asm::GPRegister::DWord(*n), asm::GPRegister::DWord(9))
            }
            ArmRegister::FloatingPoint(_) => {
                todo!("Floating Point Registers")
            }
        },
        ir::Operand::Constant(base_con) => {
            dbg!(&base_con);
            todo!()
        }
    };

    match op {
        ir::UnaryOp::Arith(arith_op) => {
            match arith_op {
                ir::UnaryArithmeticOp::Increment => {
                    instr.push(asm::Instruction::AddImmediate {
                        dest: t_reg,
                        src: asm::GpOrSpRegister::GP(base_reg),
                        immediate: 1,
                        shift: 0,
                    });
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            };
        }
        other => {
            dbg!(&other);
            todo!()
        }
    };
}
