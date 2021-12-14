use std::collections::HashMap;

use ir::{BasicBlock, Statement, Type, Variable};
use syntax::Scope;

use crate::{
    conversion::ConvertContext, AAssignTarget, AExpression, AStatement, FunctionDeclaration,
    SemanticError,
};

mod state;
pub use state::*;

#[derive(Debug, PartialEq)]
pub struct ARootScope(pub AScope);
impl ARootScope {
    pub fn parse(scope: Scope) -> Result<Self, SemanticError> {
        let initial_state = ParseState::new();

        let scope = AScope::parse(&initial_state, scope)?;
        Ok(Self(scope))
    }
}

#[derive(Debug, PartialEq)]
pub struct AScope {
    pub statements: Vec<AStatement>,
    pub function_definitions: HashMap<String, (FunctionDeclaration, AScope)>,
}

impl AScope {
    pub fn parse(external_state: &ParseState, scope: Scope) -> Result<Self, SemanticError> {
        let mut current = ParseState::based(external_state);
        dbg!(&current);

        let mut statements: Vec<AStatement> = Vec::new();

        for statement in scope.statements {
            if let Some(tmp) = AStatement::parse(statement, &mut current)? {
                dbg!(&tmp);

                statements.push(tmp);
            }
        }

        let func_definitions = current.destructure();

        Ok(Self {
            statements,
            function_definitions: func_definitions,
        })
    }

    pub fn from_parse_state(state: ParseState, statements: Vec<AStatement>) -> Self {
        let func_definitions = state.destructure();

        Self {
            statements,
            function_definitions: func_definitions,
        }
    }

    /// This will convert the current Scope into the IR used for in the Rest of the Compiler.
    /// The given Block will be used as the starting Point but as there may be multiple basic
    /// blocks produced as a result of this conversion (like with different control-flows)
    /// so the final BasicBlock will be returned and can then be used as a starting Point
    /// for the next Statements following this Scope
    pub fn to_ir(self, block: &BasicBlock, ctx: &ConvertContext) -> BasicBlock {
        let mut block = block.clone();

        for tmp_stmnt in self.statements {
            match tmp_stmnt {
                AStatement::Assignment { target, value } => {
                    dbg!(&target, &value);

                    let next_var = match target {
                        AAssignTarget::Variable { ident, ty_info } => {
                            dbg!(&ident, &ty_info);

                            let var_name = ident.0.data;
                            match block.definition(&var_name) {
                                Some(var) => var.next_gen(),
                                None => {
                                    let target_ty = ty_info.data.to_ir();

                                    ir::Variable::new(var_name.clone(), target_ty)
                                }
                            }
                        }
                        other => {
                            dbg!(&other);

                            todo!("Unknown Assign Target");
                        }
                    };
                    dbg!(&next_var);

                    let value_exp = value.to_ir(&block);

                    block.add_statement(ir::Statement::Assignment {
                        target: next_var,
                        value: value_exp,
                    });
                }
                AStatement::Expression(exp) => {
                    dbg!(&exp);

                    // TODO
                    // No idea how I might go about doing this

                    match exp {
                        AExpression::FunctionCall {
                            name,
                            arguments,
                            result_ty,
                        } => {
                            dbg!(&name, &arguments, &result_ty);

                            todo!("Handle raw FunctionCall");
                        }
                        other => {
                            dbg!(&other);

                            todo!("Unknown Standalone Expression")
                        }
                    };
                }
                AStatement::Return { value } => {
                    dbg!(&value);

                    let ret_stmnt = match value {
                        Some(val) => {
                            dbg!(&val);

                            todo!("")
                        }
                        None => ir::Statement::Return(None),
                    };

                    block.add_statement(ret_stmnt);
                }
                AStatement::If {
                    body,
                    condition,
                    else_,
                } => {
                    dbg!(&body, &condition);

                    let cond_value = condition.to_ir(&block);
                    let tmp_var_name = block.get_next_tmp_name();
                    let cond_var = Variable::new(tmp_var_name, Type::I64);

                    let cond_statement = ir::Statement::Assignment {
                        target: cond_var.clone(),
                        value: cond_value,
                    };
                    block.add_statement(cond_statement);

                    // The final resulting Block we reach after the If-Statement is complete
                    let end_block = BasicBlock::new(vec![], vec![]);

                    // The Block for the inner Scope of the If-Statement if true
                    let true_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                    let end_true_body = body.to_ir(&true_block, ctx);
                    end_true_body.add_statement(Statement::Jump(end_block.clone()));
                    end_block.add_predecessor(end_true_body.weak_ptr());

                    block.add_statement(Statement::JumpTrue(cond_var.clone(), true_block.clone()));

                    if let Some(else_) = else_ {
                        let false_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                        let end_false_block = else_.to_ir(&false_block, ctx);
                        end_false_block.add_statement(Statement::Jump(end_block.clone()));
                        block.add_statement(Statement::Jump(end_false_block.clone()));
                        end_block.add_predecessor(end_false_block.weak_ptr());
                    } else {
                        // Jump to the end Block directly
                        block.add_statement(Statement::Jump(end_block.clone()));
                        end_block.add_predecessor(block.weak_ptr());
                    }

                    block = end_block;
                }
                AStatement::WhileLoop { condition, body } => {
                    dbg!(&condition, &body);

                    let start_block = BasicBlock::new(vec![block.weak_ptr()], vec![]);
                    let inner_block = BasicBlock::new(vec![start_block.weak_ptr()], vec![]);
                    let end_block = BasicBlock::new(vec![start_block.weak_ptr()], vec![]);

                    // Generate the first iteration of the start Block
                    {
                        let cond_value = condition.to_ir(&start_block);
                        let tmp_var_name = start_block.get_next_tmp_name();
                        let cond_var = Variable::new(tmp_var_name, Type::I64);

                        let cond_statement = ir::Statement::Assignment {
                            target: cond_var.clone(),
                            value: cond_value,
                        };
                        start_block.add_statement(cond_statement);

                        start_block
                            .add_statement(Statement::JumpTrue(cond_var, inner_block.clone()));
                        start_block.add_statement(Statement::Jump(end_block.clone()));
                    }

                    // Generate the inner Part of the Loop
                    {
                        let loop_ctx = ctx.with_loop(start_block.clone(), end_block.clone());

                        let inner_end_block = body.to_ir(&inner_block, &loop_ctx);
                        inner_end_block.add_statement(Statement::Jump(start_block.clone()));
                        start_block.add_predecessor(inner_end_block.weak_ptr());
                    }

                    // TODO
                    // Regenerate the condition of the Loop

                    block.add_statement(Statement::Jump(start_block));
                    block = end_block;
                }
                AStatement::Break => {
                    let loop_end_block = match ctx.get_loop_end() {
                        Some(b) => b,
                        None => panic!("Break outside of Loop"),
                    };

                    loop_end_block.add_predecessor(block.weak_ptr());
                    block.add_statement(Statement::Jump(loop_end_block.clone()));
                }
                AStatement::Continue => {
                    dbg!(ctx);

                    todo!()
                }
                other => {
                    dbg!(&other);

                    todo!()
                }
            };
        }

        block
    }
}
