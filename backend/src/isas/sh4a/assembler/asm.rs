use std::collections::HashMap;

use crate::isas::sh4a;

#[derive(Debug)]
pub enum Instruction {
    Nop,
    /// mov #imm, Rn
    MovIR {
        dest: u8,
        immediate: u8,
    },
    /// mov Rm, Rn
    MovRR {
        src: u8,
        dest: u8,
    },
    /// mov.l @(R0, Rm), Rn
    MovLR0PRR {
        base: u8,
        target: u8,
    },
    /// mov.l Rm, @-Rn
    MovLRDR {
        base: u8,
        src: u8,
    },
    /// mov.l @Rm+, Rn
    MovLRIR {
        base: u8,
        dest: u8,
    },
    /// mov.l @(disp, PC), Rn
    MovLPcDispR {
        disp: u8,
        target: u8,
    },
    /// movt Rn
    MovT {
        target: u8,
    },
    /// sub Rm,Rn
    Sub {
        dest: u8,
        src2: u8,
    },
    /// shld Rm,Rn
    Shld {
        dest: u8,
        shift: u8,
    },
    /// or Rm,Rn
    OrRR {
        dest: u8,
        src2: u8,
    },
    /// cmp/gt Rm,Rn
    CmpGt {
        left: u8,
        right: u8,
    },
    /// cmp/pl Rn
    CmpPl {
        reg: u8,
    },
    /// braf Rm
    BraPCR {
        reg: u8,
    },
    /// bra
    BraDisp {
        disp: u8,
    },
    /// bf
    BFDisp {
        disp: i8,
    },
    /// jsr @Rm
    Jsr {
        reg: u8,
    },
    /// rts
    Rts,
    Raw(u16),
}

impl Instruction {
    pub fn from_instr(
        instr: sh4a::Instruction,
        pc: u32,
        offsets: &HashMap<String, u32>,
    ) -> Vec<Self> {
        match instr {
            sh4a::Instruction::Nop => {
                vec![Self::Nop]
            }
            sh4a::Instruction::MovIR { dest, immediate } => {
                let immediate = immediate.to_be_bytes()[0];
                let dest = dest.register().to_be();

                vec![Self::MovIR { dest, immediate }]
            }
            sh4a::Instruction::MovImmR { dest, immediate } => {
                let dest = dest.register().to_be();

                let imm_parts = immediate.to_be_bytes();
                let imm_part_2: u16 = u16::from_be_bytes([imm_parts[3], imm_parts[2]]).to_be();
                let imm_part_1: u16 = u16::from_be_bytes([imm_parts[1], imm_parts[0]]).to_be();

                let jump_disp = if pc % 4 != 0 { todo!() } else { 2 };
                let load_disp = if pc % 4 != 0 { todo!() } else { 1 };

                let mut result = vec![
                    Self::MovLPcDispR {
                        target: dest,
                        disp: load_disp,
                    },
                    Self::Nop,
                    Self::BraDisp { disp: jump_disp },
                    Self::Nop,
                ];

                if pc % 4 != 0 {
                    todo!()
                }

                result.extend([Self::Raw(imm_part_1), Self::Raw(imm_part_2)]);

                result
            }
            sh4a::Instruction::MovRR { src, dest } => {
                let src = src.register().to_be();
                let dest = dest.register().to_be();

                vec![Self::MovRR { src, dest }]
            }
            sh4a::Instruction::MovLR0PRR { base, target } => {
                let base = base.register().to_be();
                let target = target.register().to_be();

                vec![Self::MovLR0PRR { base, target }]
            }
            sh4a::Instruction::MovT { dest } => {
                let target = dest.register().to_be();

                vec![Self::MovT { target }]
            }
            sh4a::Instruction::PushL { reg } => {
                let base = 0x0f;
                let src = reg.register().to_be();

                vec![Self::MovLRDR { base, src }]
            }
            sh4a::Instruction::PopL { reg } => {
                let base = 0x0f;
                let dest = reg.register().to_be();

                vec![Self::MovLRIR { base, dest }]
            }
            sh4a::Instruction::Sub { dest, src2 } => {
                let dest = dest.register().to_be();
                let src2 = src2.register().to_be();

                vec![Self::Sub { dest, src2 }]
            }
            sh4a::Instruction::ShldRR { target, shift_reg } => {
                let dest = target.register().to_be();
                let shift = shift_reg.register().to_be();

                vec![Self::Shld { dest, shift }]
            }
            sh4a::Instruction::OrRR { target, src2 } => {
                let dest = target.register().to_be();
                let src2 = src2.register().to_be();

                vec![Self::OrRR { dest, src2 }]
            }
            sh4a::Instruction::CmpGt { left, right } => {
                let left = left.register().to_be();
                let right = right.register().to_be();

                vec![Self::CmpGt { left, right }]
            }
            sh4a::Instruction::CmpPl { src } => {
                let reg = src.register().to_be();

                vec![Self::CmpPl { reg }]
            }
            sh4a::Instruction::JumpLabel { label } => {
                let raw_target_pc: u32 = *offsets.get(&label).unwrap();
                let target_pc: i64 = raw_target_pc.try_into().unwrap();

                let jump_pc: i64 = (pc + 4).try_into().unwrap();
                let pc_difference: i32 = (target_pc - (jump_pc + 4)).try_into().unwrap();
                let pc_diff_parts = pc_difference.to_be_bytes();
                let pc_diff_part_2: u16 =
                    u16::from_be_bytes([pc_diff_parts[3], pc_diff_parts[2]]).to_be();
                let pc_diff_part_1: u16 =
                    u16::from_be_bytes([pc_diff_parts[1], pc_diff_parts[0]]).to_be();

                let needs_padding = pc % 4 != 0;
                let disp = if needs_padding { todo!() } else { 1 };

                let mut result = vec![
                    Self::MovLPcDispR { target: 4, disp },
                    Self::Nop,
                    Self::BraPCR { reg: 4 },
                    Self::Nop,
                ];

                if needs_padding {
                    result.push(Self::Nop);
                }

                result.push(Self::Raw(pc_diff_part_1));
                result.push(Self::Raw(pc_diff_part_2));

                result
            }
            sh4a::Instruction::BranchTrueLabel { label } => {
                let raw_target_pc: u32 = *offsets.get(&label).unwrap();
                let target_pc: i64 = raw_target_pc.try_into().unwrap();

                let jump_pc: i64 = (pc + 8).try_into().unwrap();
                let pc_difference: i32 = (target_pc - (jump_pc + 4)).try_into().unwrap();
                let pc_diff_parts = pc_difference.to_be_bytes();
                let pc_diff_part_1: u16 =
                    u16::from_be_bytes([pc_diff_parts[0], pc_diff_parts[1]]).to_be();
                let pc_diff_part_2: u16 =
                    u16::from_be_bytes([pc_diff_parts[2], pc_diff_parts[3]]).to_be();

                let jump_instr = {
                    let needs_padding = pc % 4 != 0;
                    let disp = if needs_padding { todo!() } else { 2 };

                    let mut tmp = vec![
                        Self::MovLPcDispR { target: 4, disp },
                        Self::Nop,
                        Self::BraPCR { reg: 4 },
                        Self::Nop,
                    ];

                    if needs_padding {
                        tmp.push(Self::Nop);
                    }

                    tmp.push(Self::Raw(pc_diff_part_1));
                    tmp.push(Self::Raw(pc_diff_part_2));

                    tmp
                };

                let br_distance: i8 = jump_instr.len().try_into().unwrap();

                let mut result = vec![
                    Self::BFDisp {
                        disp: br_distance.to_be(),
                    },
                    Self::Nop,
                ];
                result.extend(jump_instr);

                result
            }
            sh4a::Instruction::JumpSubroutine { target } => {
                let target = target.register().to_be();

                vec![Self::Jsr { reg: target }]
            }
            sh4a::Instruction::Return => {
                vec![Self::Rts, Self::Nop]
            }
            other => {
                dbg!(&other);
                todo!()
            }
        }
    }

    pub fn into_bytes(self) -> u16 {
        match self {
            Self::Nop => 0x0009,
            Self::MovRR { dest, src } => {
                let dest = dest as u16;
                let src = src as u16;

                0x6003 | (dest << 8) | (src << 4)
            }
            Self::MovIR { dest, immediate } => {
                let immediate = immediate as u16;
                let dest = dest as u16;

                0xe000 | (dest << 8) | immediate
            }
            Self::MovLR0PRR { base, target } => {
                let base = base as u16;
                let target = target as u16;

                0x000e | (target << 8) | (base << 4)
            }
            Self::MovLRDR { base, src } => {
                let base = base as u16;
                let src = src as u16;

                0x2006 | (base << 8) | (src << 4)
            }
            Self::MovLRIR { base, dest } => {
                let base = base as u16;
                let dest = dest as u16;

                0x6006 | (dest << 8) | (base << 4)
            }
            Self::MovLPcDispR { target, disp } => {
                let target = target as u16;
                let disp = disp as u16;

                0xd000 | (target << 8) | disp
            }
            Self::MovT { target } => {
                let target = target as u16;

                0x0029 | (target << 8)
            }
            Self::Sub { dest, src2 } => {
                let dest = dest as u16;
                let src2 = src2 as u16;

                0x3008 | (dest << 8) | (src2 << 4)
            }
            Self::Shld { dest, shift } => {
                let dest = dest as u16;
                let shift = shift as u16;

                0x400d | (dest << 8) | (shift << 4)
            }
            Self::OrRR { dest, src2 } => {
                let dest = dest as u16;
                let src2 = src2 as u16;

                0x200b | (dest << 8) | (src2 << 4)
            }
            Self::CmpGt { left, right } => {
                let left = left as u16;
                let right = right as u16;

                0x3007 | (left << 8) | (right << 4)
            }
            Self::CmpPl { reg } => {
                let reg = reg as u16;

                0x4015 | (reg << 8)
            }
            Self::BraPCR { reg } => {
                let reg = reg as u16;

                0x0023 | (reg << 8)
            }
            Self::BraDisp { disp } => {
                let disp = disp as u16;

                0xa000 | disp
            }
            Self::BFDisp { disp } => {
                let disp = disp as u16;

                0x8b00 | disp
            }
            Self::Jsr { reg } => {
                let reg = reg as u16;

                0x400b | (reg << 8)
            }
            Self::Rts => 0x000b,
            Self::Raw(data) => data,
        }
    }
}
