/// An Arithmetic Operation applied to a single Operand
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryArithmeticOp {
    /// Increments the Value of the Operand by 1
    Increment,
    /// Decrements the Value of the Operand by 1
    Decrement,
}

/// A Logic Operation applied to a single Operand
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryLogicOp {
    /// Switches the boolean logic Value of the Operand
    Not,
}

/// A Bitwise Operation applied to a single Operand
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryBitwiseOp {
    /// Switches all the bits of the Value
    Not,
}

/// An Operator that is applied to a single Operand at a Time
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    /// Performs arithmetic Operations
    Arith(UnaryArithmeticOp),
    /// Performs logic Operations
    Logic(UnaryLogicOp),
    /// Performs bitwise Operations
    Bitwise(UnaryBitwiseOp),
}
