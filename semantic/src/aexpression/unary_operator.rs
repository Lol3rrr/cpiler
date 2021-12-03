use syntax::SingleOperation;

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryArithmeticOp {
    SuffixIncrement,
    Negate,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryLogicOp {
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Logic(UnaryLogicOp),
    Arithmetic(UnaryArithmeticOp),
}

impl From<SingleOperation> for UnaryOperator {
    fn from(op: SingleOperation) -> Self {
        match op {
            SingleOperation::SuffixIncrement => {
                Self::Arithmetic(UnaryArithmeticOp::SuffixIncrement)
            }
            SingleOperation::Negative => Self::Arithmetic(UnaryArithmeticOp::Negate),
            SingleOperation::LogicalNot => Self::Logic(UnaryLogicOp::Not),
            unknown => todo!("Parse SingleOP: {:?}", unknown),
        }
    }
}
