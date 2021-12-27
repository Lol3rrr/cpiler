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

impl AComparitor {
    fn to_ir(&self) -> ir::BinaryLogicOp {
        match self {
            Self::Less => ir::BinaryLogicOp::Less,
            Self::LessEqual => ir::BinaryLogicOp::LessEq,
            Self::Greater => ir::BinaryLogicOp::Greater,
            Self::GreaterEqual => ir::BinaryLogicOp::GreaterEq,
            Self::Equal => ir::BinaryLogicOp::Equal,
            Self::NotEqual => ir::BinaryLogicOp::NotEqual,
        }
    }
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

impl ArithemticOp {
    fn to_ir(&self) -> ir::BinaryArithmeticOp {
        match self {
            Self::Add => ir::BinaryArithmeticOp::Add,
            Self::Sub => ir::BinaryArithmeticOp::Sub,
            Self::Multiply => ir::BinaryArithmeticOp::Multiply,
        }
    }
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
            Self::Arithmetic(arith_op) => ir::BinaryOp::Arith(arith_op.to_ir()),
            Self::Comparison(comp_op) => ir::BinaryOp::Logic(comp_op.to_ir()),
            Self::Combinator(comb_op) => ir::BinaryOp::LogicCombinator(comb_op.to_ir()),
        }
    }
}
