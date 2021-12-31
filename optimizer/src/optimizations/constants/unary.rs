use ir::{Constant, Expression, Operand, UnaryArithmeticOp, UnaryOp, Value};

pub fn arith_con(op: UnaryArithmeticOp, con: Constant) -> Value {
    match op {
        UnaryArithmeticOp::Negate => Value::Expression(Expression::UnaryOp {
            op: UnaryOp::Arith(op),
            base: Operand::Constant(con),
        }),
        UnaryArithmeticOp::Increment => match con {
            Constant::I8(v) => Value::Constant(Constant::I8(v + 1)),
            Constant::I16(v) => Value::Constant(Constant::I16(v + 1)),
            Constant::I32(v) => Value::Constant(Constant::I32(v + 1)),
            Constant::I64(v) => Value::Constant(Constant::I64(v + 1)),
            Constant::U8(v) => Value::Constant(Constant::U8(v + 1)),
            Constant::U16(v) => Value::Constant(Constant::U16(v + 1)),
            Constant::U32(v) => Value::Constant(Constant::U32(v + 1)),
            Constant::U64(v) => Value::Constant(Constant::U64(v + 1)),
            other => Value::Expression(Expression::UnaryOp {
                op: UnaryOp::Arith(UnaryArithmeticOp::Increment),
                base: Operand::Constant(other),
            }),
        },
        UnaryArithmeticOp::Decrement => match con {
            Constant::I8(v) => Value::Constant(Constant::I8(v - 1)),
            Constant::I16(v) => Value::Constant(Constant::I16(v - 1)),
            Constant::I32(v) => Value::Constant(Constant::I32(v - 1)),
            Constant::I64(v) => Value::Constant(Constant::I64(v - 1)),
            Constant::U8(v) => Value::Constant(Constant::U8(v - 1)),
            Constant::U16(v) => Value::Constant(Constant::U16(v - 1)),
            Constant::U32(v) => Value::Constant(Constant::U32(v - 1)),
            Constant::U64(v) => Value::Constant(Constant::U64(v - 1)),
            other => Value::Expression(Expression::UnaryOp {
                op: UnaryOp::Arith(UnaryArithmeticOp::Decrement),
                base: Operand::Constant(other),
            }),
        },
    }
}
