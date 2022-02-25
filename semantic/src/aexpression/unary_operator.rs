use ir::{BasicBlock, Value};
use syntax::SingleOperation;

use crate::{conversion::ConvertContext, AExpression, AStatement};

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryArithmeticOp {
    SuffixIncrement,
    SuffixDecrement,
    Positive,
    Negate,
    /// Simply increments the base Value and returns the Result
    Increment,
    /// Simply decrements the base Value and returns the Result
    Decrement,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryLogicOp {
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Bitwise {
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Logic(UnaryLogicOp),
    Arithmetic(UnaryArithmeticOp),
    Bitwise(Bitwise),
    Derference,
}

impl From<SingleOperation> for UnaryOperator {
    fn from(op: SingleOperation) -> Self {
        match op {
            SingleOperation::SuffixIncrement => {
                Self::Arithmetic(UnaryArithmeticOp::SuffixIncrement)
            }
            SingleOperation::SuffixDecrement => {
                Self::Arithmetic(UnaryArithmeticOp::SuffixDecrement)
            }
            SingleOperation::Negative => Self::Arithmetic(UnaryArithmeticOp::Negate),
            SingleOperation::LogicalNot => Self::Logic(UnaryLogicOp::Not),
            SingleOperation::Dereference => Self::Derference,
            SingleOperation::PrefixIncrement => Self::Arithmetic(UnaryArithmeticOp::Increment),
            SingleOperation::PrefixDecrement => Self::Arithmetic(UnaryArithmeticOp::Decrement),
            SingleOperation::Positive => Self::Arithmetic(UnaryArithmeticOp::Positive),
            SingleOperation::BitwiseNot => Self::Bitwise(Bitwise::Not),
            unknown => todo!("Parse SingleOP: {:?}", unknown),
        }
    }
}

impl UnaryOperator {
    /// Converts the Operator with the given Base into their corresponding IR
    pub fn to_ir(
        self,
        base: Box<AExpression>,
        block: &mut BasicBlock,
        ctx: &ConvertContext,
    ) -> ir::Value {
        let base_value = base.clone().to_ir(block, ctx);

        match self {
            Self::Arithmetic(UnaryArithmeticOp::SuffixDecrement) => {
                let base_target = base.clone().assign_target();

                let result_var = ir::Variable::tmp(ctx.next_tmp(), base.result_type().to_ir())
                    .set_description("Temp Variable holding Value before Decrementing");
                let result_assign = ir::Statement::Assignment {
                    target: result_var.clone(),
                    value: base_value,
                };
                block.add_statement(result_assign);

                let update_assign = AStatement::Assignment {
                    target: base_target,
                    value: AExpression::UnaryOperator {
                        base,
                        op: UnaryOperator::Arithmetic(UnaryArithmeticOp::Decrement),
                    },
                };
                update_assign.to_ir(block, ctx);

                ir::Value::Variable(result_var)
            }
            Self::Arithmetic(UnaryArithmeticOp::SuffixIncrement) => {
                let base_target = base.clone().assign_target();

                let result_var = ir::Variable::tmp(ctx.next_tmp(), base.result_type().to_ir())
                    .set_description("Temp Variable holding Value before Incrementing");

                let result_assign = ir::Statement::Assignment {
                    target: result_var.clone(),
                    value: base_value,
                };
                block.add_statement(result_assign);

                let update_assign = AStatement::Assignment {
                    target: base_target,
                    value: AExpression::UnaryOperator {
                        base,
                        op: UnaryOperator::Arithmetic(UnaryArithmeticOp::Increment),
                    },
                };
                update_assign.to_ir(block, ctx);

                ir::Value::Variable(result_var)
            }
            Self::Arithmetic(UnaryArithmeticOp::Positive) => base.to_ir(block, ctx),
            Self::Arithmetic(UnaryArithmeticOp::Negate) => {
                let base_operand = AExpression::val_to_operand(base_value, block, ctx);
                Value::Expression(ir::Expression::UnaryOp {
                    op: ir::UnaryOp::Arith(ir::UnaryArithmeticOp::Negate),
                    base: base_operand,
                })
            }
            Self::Arithmetic(UnaryArithmeticOp::Increment) => {
                let base_operand = AExpression::val_to_operand(base_value, block, ctx);
                Value::Expression(ir::Expression::UnaryOp {
                    op: ir::UnaryOp::Arith(ir::UnaryArithmeticOp::Increment),
                    base: base_operand,
                })
            }
            Self::Arithmetic(UnaryArithmeticOp::Decrement) => {
                let base_operand = AExpression::val_to_operand(base_value, block, ctx);
                Value::Expression(ir::Expression::UnaryOp {
                    op: ir::UnaryOp::Arith(ir::UnaryArithmeticOp::Decrement),
                    base: base_operand,
                })
            }
            Self::Logic(UnaryLogicOp::Not) => {
                let base_operand = AExpression::val_to_operand(base_value, block, ctx);

                Value::Expression(ir::Expression::UnaryOp {
                    base: base_operand,
                    op: ir::UnaryOp::Logic(ir::UnaryLogicOp::Not),
                })
            }
            Self::Bitwise(Bitwise::Not) => {
                let base_operand = AExpression::val_to_operand(base_value, block, ctx);

                Value::Expression(ir::Expression::UnaryOp {
                    base: base_operand,
                    op: ir::UnaryOp::Bitwise(ir::UnaryBitwiseOp::Not),
                })
            }
            Self::Derference => {
                let read_ty = base.result_type();
                let base_operand = AExpression::val_to_operand(base_value, block, ctx);

                ir::Value::Expression(ir::Expression::ReadMemory {
                    address: base_operand,
                    read_ty: read_ty.to_ir(),
                })
            }
        }
    }
}
