use syntax::ExpressionOperator;

#[derive(Debug, PartialEq, Clone)]
pub enum AComparitor {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicCombinator {
    Or,
    And,
}

impl LogicCombinator {
    fn to_ir(&self) -> ir::BinaryLogicCombinator {
        match self {
            Self::And => ir::BinaryLogicCombinator::And,
            Self::Or => ir::BinaryLogicCombinator::Or,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArithemticOp {
    Add,
    Sub,
    Multiply,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AOperator {
    Comparison(AComparitor),
    Combinator(LogicCombinator),
    Arithmetic(ArithemticOp),
}

impl From<ExpressionOperator> for AOperator {
    fn from(src: ExpressionOperator) -> Self {
        match src {
            ExpressionOperator::Less => Self::Comparison(AComparitor::Less),
            ExpressionOperator::LessEqual => Self::Comparison(AComparitor::LessEqual),
            ExpressionOperator::Greater => Self::Comparison(AComparitor::Greater),
            ExpressionOperator::GreaterEqual => Self::Comparison(AComparitor::GreaterEqual),
            ExpressionOperator::Equal => Self::Comparison(AComparitor::Equal),
            ExpressionOperator::NotEqual => Self::Comparison(AComparitor::NotEqual),
            ExpressionOperator::LogicalOr => Self::Combinator(LogicCombinator::Or),
            ExpressionOperator::LogicalAnd => Self::Combinator(LogicCombinator::And),
            ExpressionOperator::Add => Self::Arithmetic(ArithemticOp::Add),
            ExpressionOperator::Sub => Self::Arithmetic(ArithemticOp::Sub),
            ExpressionOperator::Multiply => Self::Arithmetic(ArithemticOp::Multiply),
            unknown => panic!("Unknown OP: {:?}", unknown),
        }
    }
}

impl AOperator {
    pub fn to_ir(self) -> ir::BinaryOp {
        match self {
            Self::Arithmetic(arith_op) => match arith_op {
                ArithemticOp::Add => ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                ArithemticOp::Sub => ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Sub),
                ArithemticOp::Multiply => ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Multiply),
            },
            Self::Comparison(comp_op) => match comp_op {
                AComparitor::Equal => ir::BinaryOp::Logic(ir::BinaryLogicOp::Equal),
                AComparitor::NotEqual => ir::BinaryOp::Logic(ir::BinaryLogicOp::NotEqual),
                other => {
                    dbg!(&other);

                    todo!("Convert CompOP to ir")
                }
            },
            Self::Combinator(comb_op) => ir::BinaryOp::LogicCombinator(comb_op.to_ir()),
        }
    }
}
