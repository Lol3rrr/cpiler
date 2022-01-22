use std::collections::HashMap;

use ir::{BasicBlock, FunctionDefinition, Program, Variable};

use crate::{AScope, AAST};

mod expression;

mod context;
pub use context::*;

mod function;

pub fn convert(ast: AAST, arch: general::arch::Arch) -> Program {
    let (global_block, global_vars) = convert_global(
        ast.global_scope.0.clone(),
        ConvertContext::base(arch.clone()),
    );

    let mut functions = HashMap::new();
    for (name, (func_dec, func_scope)) in ast.global_scope.0.function_definitions {
        let return_ty = func_dec.return_ty.clone().to_ir();

        let args = {
            let mut tmp = Vec::new();

            for arg in func_dec.arguments.iter() {
                let name = arg.data.name.clone();
                let ty = arg.data.ty.clone().to_ir();

                tmp.push((name, ty));
            }

            tmp
        };

        let func_block = function::convert(
            &global_block,
            global_vars.clone(),
            name.clone(),
            func_dec,
            func_scope,
            arch.clone(),
        );

        functions.insert(
            name.clone(),
            FunctionDefinition {
                name,
                arguments: args,
                return_ty,
                block: func_block,
            },
        );
    }

    Program {
        global: global_block,
        functions,
    }
}

fn convert_global(raw_global: AScope, mut ctx: ConvertContext) -> (BasicBlock, Vec<Variable>) {
    ctx.set_global(true);

    let result_block = BasicBlock::initial(vec![]);

    let ret_block = raw_global.to_ir(&result_block, &ctx);
    if result_block.as_ptr() != ret_block.as_ptr() {
        panic!("The Block generated should be one continuos block")
    }

    let global_statements = result_block.get_statements();
    let global_vars: HashMap<String, ir::Variable> = global_statements
        .into_iter()
        .filter_map(|stmnt| match stmnt {
            ir::Statement::Assignment { target, .. } => Some(target.clone()),
            _ => None,
        })
        .map(|var| (var.name.clone(), var))
        .collect();

    result_block.add_statement(ir::Statement::Return(None));

    (
        result_block,
        global_vars.into_iter().map(|(_, v)| v).collect(),
    )
}
