// https://developer.apple.com/documentation/xcode/writing-arm64-code-for-apple-platforms
// https://developer.arm.com/documentation/102374/0101/Registers-in-AArch64---general-purpose-registers
// General: https://developer.arm.com/documentation/ddi0487/latest/
// Call ABI: https://developer.arm.com/documentation/ihi0055/b/
// https://stackoverflow.com/questions/65351533/apple-clang12-llvm-unknown-aarch64-fixup-kind

use std::{collections::HashMap, marker::PhantomData, process::Command};

use ir::Variable;

use crate::{backends::aarch64_mac::codegen::ArgTarget, util};

use super::{Target, TargetConfig};

use isas::armv8a as asm;

mod codegen;

pub struct Backend {}

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    /// General Purpose Registers
    fn registers() -> [ArmRegister; 34] {
        [
            //ArmRegister::GeneralPurpose(8),
            /*
            ArmRegister::GeneralPurpose(9),
            ArmRegister::GeneralPurpose(10),
            ArmRegister::GeneralPurpose(11),
            ArmRegister::GeneralPurpose(12),
            ArmRegister::GeneralPurpose(13),
            ArmRegister::GeneralPurpose(14),
            ArmRegister::GeneralPurpose(15),
            */
            //ArmRegister::GeneralPurpose(16),
            //ArmRegister::GeneralPurpose(17),
            //ArmRegister::GeneralPurpose(18),
            ArmRegister::GeneralPurpose(19),
            ArmRegister::GeneralPurpose(20),
            ArmRegister::GeneralPurpose(21),
            ArmRegister::GeneralPurpose(22),
            ArmRegister::GeneralPurpose(23),
            ArmRegister::GeneralPurpose(24),
            ArmRegister::GeneralPurpose(25),
            ArmRegister::GeneralPurpose(26),
            ArmRegister::GeneralPurpose(27),
            ArmRegister::GeneralPurpose(28),
            //ArmRegister::GeneralPurpose(29),
            //ArmRegister::GeneralPurpose(30),
            /*
             * I think these are mainly used in Function Calls and I therefor would rather save
             * them just to be save
            ArmRegister::FloatingPoint(0),
            ArmRegister::FloatingPoint(1),
            ArmRegister::FloatingPoint(2),
            ArmRegister::FloatingPoint(3),
            ArmRegister::FloatingPoint(4),
            ArmRegister::FloatingPoint(5),
            ArmRegister::FloatingPoint(6),
            ArmRegister::FloatingPoint(7),
            */
            ArmRegister::FloatingPoint(8),
            ArmRegister::FloatingPoint(9),
            ArmRegister::FloatingPoint(10),
            ArmRegister::FloatingPoint(11),
            ArmRegister::FloatingPoint(12),
            ArmRegister::FloatingPoint(13),
            ArmRegister::FloatingPoint(14),
            ArmRegister::FloatingPoint(15),
            ArmRegister::FloatingPoint(16),
            ArmRegister::FloatingPoint(17),
            ArmRegister::FloatingPoint(18),
            ArmRegister::FloatingPoint(19),
            ArmRegister::FloatingPoint(20),
            ArmRegister::FloatingPoint(21),
            ArmRegister::FloatingPoint(22),
            ArmRegister::FloatingPoint(23),
            ArmRegister::FloatingPoint(24),
            ArmRegister::FloatingPoint(25),
            ArmRegister::FloatingPoint(26),
            ArmRegister::FloatingPoint(27),
            ArmRegister::FloatingPoint(28),
            ArmRegister::FloatingPoint(29),
            ArmRegister::FloatingPoint(30),
            ArmRegister::FloatingPoint(31),
        ]
    }

    fn codegen(
        &self,
        func: &ir::FunctionDefinition,
        register_map: HashMap<Variable, ArmRegister>,
    ) -> Vec<asm::Block> {
        let stack_allocation = util::stack::allocate_stack(
            func,
            &register_map,
            util::stack::AllocateConfig {
                alloc_space: |space| {
                    vec![asm::Instruction::StpPreIndex {
                        first: asm::GPRegister::DWord(29),
                        second: asm::GPRegister::DWord(30),
                        base: asm::GpOrSpRegister::SP,
                        offset: -(space as i16),
                    }]
                },
                dealloc_space: |space| {
                    vec![asm::Instruction::LdpPostIndex {
                        first: asm::GPRegister::DWord(29),
                        second: asm::GPRegister::DWord(30),
                        base: asm::GpOrSpRegister::SP,
                        offset: space as i16,
                    }]
                },
                store_on_stack: |register, offset| match register {
                    ArmRegister::GeneralPurpose(n) => {
                        vec![asm::Instruction::StoreRegisterUnscaled {
                            reg: asm::GPRegister::DWord(*n),
                            base: asm::GpOrSpRegister::SP,
                            offset: asm::Imm9Signed::new(offset).unwrap(),
                        }]
                    }
                    ArmRegister::FloatingPoint(n) => vec![asm::Instruction::StoreFPUnscaled {
                        reg: asm::FPRegister::DoublePrecision(*n),
                        base: asm::GpOrSpRegister::SP,
                        offset: asm::Imm9Signed::new(offset).unwrap(),
                    }],
                },
                load_on_stack: |register, offset| match register {
                    ArmRegister::GeneralPurpose(n) => {
                        vec![asm::Instruction::LoadRegisterUnscaled {
                            reg: asm::GPRegister::DWord(*n),
                            base: asm::GpOrSpRegister::SP,
                            offset: asm::Imm9Signed::new(offset).unwrap(),
                        }]
                    }
                    ArmRegister::FloatingPoint(n) => vec![asm::Instruction::LoadFPUnscaled {
                        reg: asm::FPRegister::DoublePrecision(*n),
                        base: asm::GpOrSpRegister::SP,
                        offset: asm::Imm9Signed::new(offset).unwrap(),
                    }],
                },
                type_align_size: |ty| match ty {
                    ir::Type::I32 => (4, 4),
                    ir::Type::I64 => (8, 8),
                    ir::Type::Pointer(_) => (8, 8),
                    ir::Type::Float => (4, 4),
                    ir::Type::Void => {
                        // TODO
                        // This should probably not happen
                        (1, 1)
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                },
                stack_alignment: 16,
                stack_base: 16,
                _marker: PhantomData {},
            },
        );

        let arg_moves = {
            let starting_statements = func.block.get_statements();
            let mut statement_iter = starting_statements.into_iter();

            let mut args_moves = Vec::new();
            let arg_targets = codegen::arguments(func.arguments.iter().map(|(_, t)| t.clone()));
            for ((arg, arg_src), s) in func
                .arguments
                .iter()
                .zip(arg_targets.iter())
                .zip(statement_iter.by_ref())
            {
                let target_reg = match s {
                    ir::Statement::Assignment { target, value } => {
                        assert_eq!(arg.0, target.name);
                        assert_eq!(value, ir::Value::Unknown);

                        match register_map.get(&target).unwrap() {
                            ArmRegister::GeneralPurpose(n) => asm::GPRegister::DWord(*n),
                            ArmRegister::FloatingPoint(_n) => todo!("Floating Point Register"),
                        }
                    }
                    other => {
                        dbg!(&other);
                        //todo!()
                        // TODO
                        // Figure out this part of the Code
                        continue;
                    }
                };

                match arg_src {
                    ArgTarget::GPRegister(n) => {
                        args_moves.push(asm::Instruction::MovRegister {
                            dest: target_reg,
                            src: asm::GPRegister::DWord(*n),
                        });
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };
            }

            let remaining = statement_iter.collect();
            func.block.set_statements(remaining);

            args_moves
        };

        let asm_ctx = codegen::Context {
            registers: register_map.into(),
            var: stack_allocation.var_offsets,
            pre_ret_instr: stack_allocation.pre_return_instr.clone(),
            stack_allocs: stack_allocation.allocations,
        };

        dbg!(&func.name);
        let mut asm_blocks: Vec<_> = func
            .block
            .block_iter()
            .map(|b| codegen::block_to_asm(b, &asm_ctx))
            .collect();

        {
            let first = asm_blocks.get_mut(0).unwrap();

            let mut final_instr = arg_moves;
            final_instr.extend(std::mem::take(&mut first.instructions));
            first.instructions = final_instr;
        }

        asm_blocks.insert(
            0,
            asm::Block {
                name: func.name.clone(),
                instructions: vec![asm::Instruction::JumpLabel {
                    target: codegen::block_name(&func.block),
                }],
            },
        );

        {
            let first_block = asm_blocks.first_mut().unwrap();

            let n_instr: Vec<_> = stack_allocation
                .setup_instr
                .into_iter()
                .chain(std::mem::take(&mut first_block.instructions))
                .collect();
            first_block.instructions = n_instr;
        }

        asm_blocks
    }

    fn global_init(
        &self,
        global: ir::BasicBlock,
        ctx: &TargetConfig,
    ) -> (String, Vec<asm::Block>, Vec<(String, ir::Type)>) {
        let global_vars: HashMap<String, ir::Type> = global
            .get_statements()
            .into_iter()
            .filter_map(|s| match s {
                ir::Statement::Assignment { target, .. } => Some(target),
                _ => None,
            })
            .map(|v| (v.name, v.ty))
            .collect();

        let name = "g_init".to_string();
        let tmp_func = ir::FunctionDefinition {
            name: name.clone(),
            block: global.clone(),
            arguments: Vec::new(),
            return_ty: ir::Type::Void,
        };

        let leading_block = ir::BasicBlock::new(Vec::new(), Vec::new());
        global.add_predecessor(leading_block.weak_ptr());

        let registers = util::registers::allocate_registers(
            &tmp_func,
            &Self::registers(),
            Some(ctx.build_dir.clone()),
        );
        util::destructure::destructure_func(&tmp_func);

        let func_blocks = self.codegen(&tmp_func, registers);

        (name, func_blocks, global_vars.into_iter().collect())
    }

    fn assemble(&self, input_file: &str, target_file: &str) {
        println!("Assemble File");

        let output = Command::new("as")
            .args(["-o", target_file, input_file])
            .output()
            .expect("Failed to assemble");

        if !output.status.success() {
            let err_str = String::from_utf8(output.stderr).unwrap();
            panic!("{}", err_str);
        }
    }

    fn link(&self, files: &[&str], target_file: &str) {
        println!("Linking");

        let output = Command::new("ld")
            .args(["-macosx_version_min", "12.0.0"])
            .args(["-o", target_file])
            .args(files)
            .args([
                "-L/Library/Developer/CommandLineTools/SDKs/MacOSX12.sdk/usr/lib",
                "-lSystem",
                "-e",
                "_start",
                "-arch",
                "arm64",
            ])
            .output()
            .expect("Failed to link");

        if !output.status.success() {
            let err_str = String::from_utf8(output.stderr).unwrap();
            panic!("{}", err_str);
        }
    }

    fn asm_start_func(&self, init_name: &str, ctx: &TargetConfig) -> Vec<asm::Block> {
        let lead_block = ir::BasicBlock::new(vec![], vec![]);

        let res_var = ir::Variable::new("main_res", ir::Type::I32);
        let raw_block = ir::BasicBlock::new(
            vec![lead_block.weak_ptr()],
            vec![
                ir::Statement::Call {
                    name: init_name.to_string(),
                    arguments: Vec::new(),
                },
                ir::Statement::Assignment {
                    target: res_var.clone(),
                    value: ir::Value::Expression(ir::Expression::FunctionCall {
                        name: "main".to_string(),
                        arguments: Vec::new(),
                        return_ty: ir::Type::I32,
                    }),
                },
                ir::Statement::Return(Some(res_var)),
            ],
        );

        let raw_func = ir::FunctionDefinition {
            name: "_start".to_string(),
            arguments: Vec::new(),
            return_ty: ir::Type::Void,
            block: raw_block,
        };

        let registers = util::registers::allocate_registers(
            &raw_func,
            &Self::registers(),
            Some(ctx.build_dir.clone()),
        );

        util::destructure::destructure_func(&raw_func);

        self.codegen(&raw_func, registers)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ArmRegister {
    GeneralPurpose(u8),
    FloatingPoint(u8),
}

impl From<ArmRegister> for asm::Register {
    fn from(src: ArmRegister) -> Self {
        match src {
            ArmRegister::GeneralPurpose(n) => {
                asm::Register::GeneralPurpose(asm::GPRegister::DWord(n))
            }
            ArmRegister::FloatingPoint(n) => {
                asm::Register::FloatingPoint(asm::FPRegister::DoublePrecision(n))
            }
        }
    }
}

impl register_allocation::Register for ArmRegister {
    fn reg_type(&self) -> register_allocation::RegisterType {
        match self {
            Self::GeneralPurpose(_) => register_allocation::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => register_allocation::RegisterType::FloatingPoint,
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (8, 8),
            Self::FloatingPoint(_) => (8, 8),
        }
    }
}

impl Target for Backend {
    fn generate(&self, program: ir::Program, conf: TargetConfig) {
        let (g_init_name, global_blocks, global_vars) =
            self.global_init(program.global.clone(), &conf);

        let all_registers = Self::registers();
        let mut blocks = Vec::new();
        for (_, func) in program.functions.iter() {
            let registers = util::registers::allocate_registers(
                func,
                &all_registers,
                Some(conf.build_dir.clone()),
            );

            let reg_ir = conf.build_dir.join(format!("reg-{}-ir.s", func.name));
            std::fs::write(&reg_ir, ir::text_rep::generate_text_rep(func)).unwrap();

            util::destructure::destructure_func(func);

            let func_blocks = self.codegen(func, registers);
            blocks.extend(func_blocks);
        }

        let post_dest_ir = conf.build_dir.join("dest-ir.s");
        std::fs::write(&post_dest_ir, ir::text_rep::program_text_rep(&program)).unwrap();

        let mut asm_text = "
.global _start
.align 2
"
        .to_string();

        let (g_var_blocks, g_var_decls): (Vec<_>, Vec<_>) = global_vars
            .into_iter()
            .map(|(g_var, g_type)| {
                let instr = match g_type {
                    ir::Type::I64 | ir::Type::U64 | ir::Type::Pointer(_) => {
                        vec![asm::Instruction::Literal(".quad 0".to_string())]
                    }
                    ir::Type::I32 | ir::Type::U32 => {
                        vec![
                            //asm::Instruction::Literal(".data".to_string()),
                            asm::Instruction::Literal(".long 0".to_string()),
                        ]
                    }
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                };

                (
                    asm::Block {
                        name: g_var.clone(),
                        instructions: instr,
                    },
                    g_var,
                )
            })
            .unzip();

        let start_block = self.asm_start_func(&g_init_name, &conf);

        for block in start_block.into_iter().chain(global_blocks).chain(blocks) {
            let block_text = block.to_text();
            asm_text.push_str(&block_text);
        }

        asm_text.push_str(".data\n");
        for decl in g_var_decls {
            dbg!(&decl);
            //asm_text.push_str(&format!(".global {}\n", &decl));
        }
        for block in g_var_blocks {
            let block_text = block.to_text();
            asm_text.push_str(&block_text);
        }

        let asm_path = conf.build_dir.join("code.s");
        std::fs::write(&asm_path, asm_text).unwrap();

        let obj_path = conf.build_dir.join("code.o");
        self.assemble(asm_path.to_str().unwrap(), obj_path.to_str().unwrap());
        self.link(
            &[obj_path.to_str().unwrap()],
            conf.target_file.as_deref().unwrap_or("./code"),
        );

        /*
            Keep the build artifacts around for easier debugging
            std::fs::remove_file(asm_path).unwrap();
            std::fs::remove_file(obj_path).unwrap();
        */
    }
}
