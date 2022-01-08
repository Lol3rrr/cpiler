use crate::backends::aarch64_mac::{asm, codegen::Context};

pub fn to_asm(
    op: ir::UnaryOp,
    t_reg: asm::Register,
    base: ir::Operand,
    ctx: &Context,
    instr: &mut Vec<asm::Instruction>,
) {
    let (base_reg, imm_reg) = match base {
        ir::Operand::Variable(base_var) => match ctx.registers.get_reg(&base_var).unwrap() {
            asm::Register::GeneralPurpose(gp) => match gp {
                asm::GPRegister::DWord(n) => (asm::GPRegister::DWord(n), asm::GPRegister::DWord(9)),
                asm::GPRegister::Word(n) => (asm::GPRegister::Word(n), asm::GPRegister::Word(9)),
            },
            asm::Register::FloatingPoint(_) => {
                todo!("Floating Point Registers")
            }
        },
        ir::Operand::Constant(base_con) => {
            dbg!(&base_con);
            todo!()
        }
    };

    match (t_reg, op) {
        (asm::Register::GeneralPurpose(t_reg), ir::UnaryOp::Arith(arith_op)) => {
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
