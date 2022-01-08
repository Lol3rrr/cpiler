use crate::backends::aarch64_mac::asm;

pub fn constant_to_asm(con: &ir::Constant, dest: asm::GPRegister) -> Vec<asm::Instruction> {
    match con {
        ir::Constant::I64(val) => {
            let val = *val;
            if val >= 0 && val < 4096 {
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
