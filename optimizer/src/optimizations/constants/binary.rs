use ir::{BinaryArithmeticOp, BinaryOp, Constant, Expression, Operand, Value};

pub fn binary_arith_consts(
    op: BinaryArithmeticOp,
    left_con: Constant,
    right_con: Constant,
) -> Value {
    match op {
        BinaryArithmeticOp::Add => match (left_con, right_con) {
            (Constant::I64(l_c), Constant::I64(r_c)) => Value::Constant(Constant::I64(l_c + r_c)),
            (Constant::I32(l_c), Constant::I32(r_c)) => Value::Constant(Constant::I32(l_c + r_c)),
            (Constant::I16(l_c), Constant::I16(r_c)) => Value::Constant(Constant::I16(l_c + r_c)),
            (Constant::I8(l_c), Constant::I8(r_c)) => Value::Constant(Constant::I8(l_c + r_c)),
            (l, r) => Value::Expression(Expression::BinaryOp {
                op: BinaryOp::Arith(BinaryArithmeticOp::Add),
                left: Operand::Constant(l),
                right: Operand::Constant(r),
            }),
        },
        BinaryArithmeticOp::Sub => match (left_con, right_con) {
            (Constant::I64(l_c), Constant::I64(r_c)) => Value::Constant(Constant::I64(l_c - r_c)),
            (Constant::I32(l_c), Constant::I32(r_c)) => Value::Constant(Constant::I32(l_c - r_c)),
            (Constant::I16(l_c), Constant::I16(r_c)) => Value::Constant(Constant::I16(l_c - r_c)),
            (Constant::I8(l_c), Constant::I8(r_c)) => Value::Constant(Constant::I8(l_c - r_c)),
            (l, r) => Value::Expression(Expression::BinaryOp {
                op: BinaryOp::Arith(BinaryArithmeticOp::Sub),
                left: Operand::Constant(l),
                right: Operand::Constant(r),
            }),
        },
        BinaryArithmeticOp::Multiply => match (left_con, right_con) {
            (Constant::I64(l_c), Constant::I64(r_c)) => Value::Constant(Constant::I64(l_c * r_c)),
            (Constant::I32(l_c), Constant::I32(r_c)) => Value::Constant(Constant::I32(l_c * r_c)),
            (Constant::I16(l_c), Constant::I16(r_c)) => Value::Constant(Constant::I16(l_c * r_c)),
            (Constant::I8(l_c), Constant::I8(r_c)) => Value::Constant(Constant::I8(l_c * r_c)),
            (l, r) => Value::Expression(Expression::BinaryOp {
                op: BinaryOp::Arith(BinaryArithmeticOp::Multiply),
                left: Operand::Constant(l),
                right: Operand::Constant(r),
            }),
        },
        other => Value::Expression(Expression::BinaryOp {
            op: BinaryOp::Arith(other),
            left: Operand::Constant(left_con),
            right: Operand::Constant(right_con),
        }),
    }
}
