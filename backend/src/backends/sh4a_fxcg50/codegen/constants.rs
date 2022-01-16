use crate::isas::sh4a;

pub fn store_isize(register: sh4a::GeneralPurposeRegister, value: isize) -> Vec<sh4a::Instruction> {
    if (0..(i8::MAX as isize)).contains(&value) {
        return vec![sh4a::Instruction::MovIR {
            dest: register,
            immediate: value.try_into().unwrap(),
        }];
    }

    todo!()
}

pub fn store_i64(register: sh4a::GeneralPurposeRegister, value: i64) -> Vec<sh4a::Instruction> {
    if let Ok(store_val) = value.try_into() {
        return vec![sh4a::Instruction::MovIR {
            dest: register,
            immediate: store_val,
        }];
    }

    todo!()
}

pub fn store_u32(register: sh4a::GeneralPurposeRegister, value: u32) -> Vec<sh4a::Instruction> {
    vec![sh4a::Instruction::MovImmR {
        immediate: value,
        dest: register,
    }]
}
