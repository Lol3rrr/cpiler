use ir::{BasicBlock, Operand, Value};
use syntax::Identifier;

use crate::{conversion::ConvertContext, AExpression, AType};

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: Identifier,
    pub arguments: Vec<AExpression>,
    pub result_ty: AType,
}

impl FunctionCall {
    fn argument_ir(
        arguments: Vec<AExpression>,
        block: &mut BasicBlock,
        ctx: &ConvertContext,
    ) -> Vec<ir::Operand> {
        let mut args = Vec::new();
        for tmp_arg in arguments {
            let arg_value = tmp_arg.to_ir(block, ctx);

            let arg_oper = AExpression::val_to_operand(arg_value, block, ctx);

            args.push(arg_oper);
        }

        args
    }

    fn cleanup_ir(args: &[Operand]) -> Vec<ir::Statement> {
        args.iter()
            .filter_map(|arg| match arg {
                Operand::Variable(var) if var.ty.is_ptr() => Some(ir::Statement::Assignment {
                    target: var.next_gen(),
                    value: Value::Unknown,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn to_ir(self, block: &mut BasicBlock, ctx: &ConvertContext) -> ir::Value {
        let name = self.name.0.data;
        let args = Self::argument_ir(self.arguments, block, ctx);
        let ty = self.result_ty.to_ir();

        let tmp_var = ir::Variable::tmp(ctx.next_tmp(), ty.clone());

        let cleanup_statements = Self::cleanup_ir(&args);

        let func_statement = ir::Statement::Assignment {
            target: tmp_var.clone(),
            value: Value::Expression(ir::Expression::FunctionCall {
                name,
                arguments: args,
                return_ty: ty,
            }),
        };
        block.add_statement(func_statement);

        for tmp in cleanup_statements {
            block.add_statement(tmp);
        }

        Value::Variable(tmp_var)
    }

    pub fn to_standalone_ir(self, block: &mut BasicBlock, ctx: &ConvertContext) {
        let name = self.name.0.data;
        let args = Self::argument_ir(self.arguments, block, ctx);
        let cleanup_statements = Self::cleanup_ir(&args);

        let func_statemnet = ir::Statement::Call {
            name,
            arguments: args,
        };

        block.add_statement(func_statemnet);
        for c_stmnt in cleanup_statements {
            block.add_statement(c_stmnt);
        }
    }
}
