use crate::{backends::sh4a_fxcg50::codegen, isas::sh4a};

#[derive(Debug)]
pub struct Context {
    pub inputs: Vec<sh4a::Register>,
    pub output: Option<sh4a::Register>,
}

pub fn convert(template: String, ctx: Context) -> Vec<sh4a::Instruction> {
    let line_iter = template
        .lines()
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty());

    line_iter.flat_map(|l| line_to_instr(&l, &ctx)).collect()
}

#[derive(Debug)]
enum Argument {
    GeneralPurposeRegister(u8),
    Immediate(u32),
}

impl Argument {
    fn parse(raw: &str) -> Self {
        if let Some(raw_reg_number) = raw.strip_prefix('r') {
            let reg_number: u8 = raw_reg_number.parse().unwrap();
            return Self::GeneralPurposeRegister(reg_number);
        }

        if let Some(raw_hex_str) = raw.strip_prefix("0x") {
            let hex_numb = u32::from_str_radix(raw_hex_str, 16).unwrap();
            return Self::Immediate(hex_numb);
        }

        dbg!(&raw);
        todo!()
    }
}

fn line_to_instr(line: &str, _ctx: &Context) -> Vec<sh4a::Instruction> {
    let first_sep = line.find(' ').unwrap_or(line.len());

    let op = &line[..first_sep];
    let rest = &line[first_sep..];

    match op {
        "mov.l" => {
            let mut args: Vec<_> = rest.split(',').map(|a| Argument::parse(a.trim())).collect();
            if args.len() != 2 {
                panic!("")
            }

            let right = args.pop().unwrap();
            let left = args.pop().unwrap();

            match (left, right) {
                (Argument::GeneralPurposeRegister(raw_target), value) => {
                    let target = sh4a::GeneralPurposeRegister::new(raw_target);

                    match value {
                        Argument::GeneralPurposeRegister(src) => {
                            dbg!(src);
                            todo!()
                        }
                        Argument::Immediate(imm) => codegen::constants::store_u32(target, imm),
                    }
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            }
        }
        "jsr" => {
            let arg = Argument::parse(rest.trim());

            match arg {
                Argument::GeneralPurposeRegister(gp) => {
                    let target = sh4a::GeneralPurposeRegister::new(gp);

                    vec![sh4a::Instruction::JumpSubroutine { target }]
                }
                other => {
                    dbg!(&other);
                    todo!()
                }
            }
        }
        "nop" => {
            vec![sh4a::Instruction::Nop]
        }
        other => {
            dbg!(&other, &rest);
            todo!()
        }
    }
}
