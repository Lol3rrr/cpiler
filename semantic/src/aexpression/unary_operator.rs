use syntax::SingleOperation;

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryArithmeticOp {
    SuffixIncrement,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Arithmetic(UnaryArithmeticOp),
}

impl From<SingleOperation> for UnaryOperator {
    fn from(op: SingleOperation) -> Self {
        match op {
            SingleOperation::SuffixIncrement => {
                Self::Arithmetic(UnaryArithmeticOp::SuffixIncrement)
            }
            unknown => todo!("Parse SingleOP: {:?}", unknown),
        }
    }
}
