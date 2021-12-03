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

#[derive(Debug, PartialEq, Clone)]
pub enum ArithemticOp {
    Add,
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
            ExpressionOperator::Multiply => Self::Arithmetic(ArithemticOp::Multiply),
            unknown => panic!("Unknown OP: {:?}", unknown),
        }
    }
}
