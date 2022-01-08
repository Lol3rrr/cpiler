use crate::backends::aarch64_mac::asm;

pub fn constant_to_asm(con: &ir::Constant, dest: asm::Register) -> Vec<asm::Instruction> {
    match (dest, con) {
        (asm::Register::GeneralPurpose(dest), ir::Constant::I64(val)) => {
            let val = *val;
            if (0..4096).contains(&val) {
                vec![asm::Instruction::Movz {
                    dest,
                    immediate: val as u16,
                    shift: 0,
                }]
            } else {
                todo!()
            }
        }
        other => {
            dbg!(&other);

            todo!()
        }
    }
}
