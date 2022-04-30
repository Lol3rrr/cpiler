use crate::{AScope, FunctionDeclaration};

use super::ConvertContext;

pub fn convert(
    global: &ir::BasicBlock,
    global_vars: Vec<ir::Variable>,
    _name: String,
    func_dec: FunctionDeclaration,
    inner_scope: AScope,
    arch: general::arch::Arch,
) -> ir::BasicBlock {
    // Put the Arguments into the first basic Block and then place a Jump as the last Statement
    // that will jump to the actual function code

    let arg_statements = {
        let mut tmp = Vec::new();

        for tmp_arg in func_dec.arguments.iter() {
            let var_data = &tmp_arg.data;
            let var_ty = tmp_arg.data.ty.clone().to_ir();
            let var = ir::Variable::new(&var_data.name, var_ty);

            tmp.push(ir::Statement::Assignment {
                target: var,
                value: ir::Value::Unknown,
            });
        }

        tmp
    };

    /*
    let globals: Vec<_> = global_vars
        .iter()
        .map(|var| (var.next_gen(), var.clone()))
        .map(|(target, src_var)| ir::Statement::Assignment {
            target,
            value: ir::Value::Expression(ir::Expression::ReadGlobalVariable { name: src_var.name }),
        })
        .collect();
    */

    let init_statements: Vec<_> = arg_statements
        .into_iter()
        //.chain(globals.into_iter())
        .collect();

    let global_weak = global.weak_ptr();
    let head_block = ir::BasicBlock::new(vec![global_weak], init_statements);

    let context = ConvertContext::base(
        arch,
        global_vars
            .into_iter()
            .map(|v| (v.name().to_string(), v))
            .collect(),
    );

    let head_weak = head_block.weak_ptr();
    let func_block = ir::BasicBlock::new(vec![head_weak], vec![]);
    inner_scope.to_ir(&func_block, &context);

    // Update Head-Blocks last Jump to the next
    head_block.add_statement(ir::Statement::Jump(func_block, ir::JumpMetadata::Linear));

    head_block
}
