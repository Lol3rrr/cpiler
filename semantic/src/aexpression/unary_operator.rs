use syntax::SingleOperation;

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryArithmeticOp {
    SuffixIncrement,
    SuffixDecrement,
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
            SingleOperation::SuffixDecrement => {
                Self::Arithmetic(UnaryArithmeticOp::SuffixDecrement)
            }
            SingleOperation::Negative => Self::Arithmetic(UnaryArithmeticOp::Negate),
            SingleOperation::LogicalNot => Self::Logic(UnaryLogicOp::Not),
            unknown => todo!("Parse SingleOP: {:?}", unknown),
        }
    }
}
