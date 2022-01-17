use general::SpanData;
use ir::{BasicBlock, Constant, Expression, Operand, Statement, Type, Value, Variable};

use crate::conversion::ConvertContext;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(SpanData<i64>),
    FloatingPoint(SpanData<f64>),
    StringLiteral(SpanData<String>),
    CharLiteral(SpanData<char>),
}

impl Literal {
    pub fn to_value(self, block: &BasicBlock, ctx: &ConvertContext) -> Value {
        match self {
            Self::Integer(SpanData { data, .. }) => Value::Constant(Constant::I64(data)),
            Self::StringLiteral(SpanData { data, .. }) => {
                dbg!(&data);

                if !data.is_ascii() {
                    panic!("Currently only supports ASCII-Strings");
                }

                let data_len = data.len();
                let data_bytes = data.bytes();

                let arr_tmp_name = ctx.next_tmp();
                let arr_tmp = Variable::tmp(arr_tmp_name, Type::Pointer(Box::new(Type::U8)));
                let arr_decl = Statement::Assignment {
                    target: arr_tmp.clone(),
                    value: Value::Expression(Expression::StackAlloc {
                        size: data_len,
                        alignment: 1,
                    }),
                };

                block.add_statement(arr_decl);

                for (index, tmp_byte) in data_bytes.enumerate() {
                    let offset_exp = Expression::BinaryOp {
                        op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                        left: Operand::Variable(arr_tmp.clone()),
                        right: Operand::Constant(Constant::I64(index as i64)),
                    };
                    let offset_var = Variable::tmp(ctx.next_tmp(), Type::I64);

                    let offset_statement = Statement::Assignment {
                        target: offset_var.clone(),
                        value: Value::Expression(offset_exp),
                    };
                    block.add_statement(offset_statement);

                    let assign_statement = Statement::WriteMemory {
                        target: Operand::Variable(offset_var),
                        value: Value::Constant(Constant::U8(tmp_byte)),
                    };
                    block.add_statement(assign_statement);
                }

                Value::Variable(arr_tmp)
            }
            Self::CharLiteral(SpanData { data, .. }) => {
                assert!(data.is_ascii());

                let result = data as u8;

                ir::Value::Constant(ir::Constant::U8(result))
            }
            Self::FloatingPoint(SpanData { data, .. }) => {
                ir::Value::Constant(ir::Constant::F64(data))
            }
        }
    }
}
