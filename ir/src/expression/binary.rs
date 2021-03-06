/// An Arithmetic Operation applied to two Operands
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryArithmeticOp {
    /// Adds the two Operands together
    Add,
    /// Subtracts the second Operand from the first
    Sub,
    /// Multiplies the two Operands together
    Multiply,
    /// Divides the first Operand by the second
    Divide,
    /// Returns the Rest of the Integer Division of the first Operand by the second
    Modulo,
}

/// A Logic Operation applied to two Operands
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryLogicOp {
    /// Checks if the two Operands are Equal
    Equal,
    /// Checks if the two Operands are not Equal
    NotEqual,
    /// Checks if the first Operand is less than the second
    Less,
    /// Checks if the first Operand is less than or equal the second
    LessEq,
    /// Checks if the first Operand is greater than the second
    Greater,
    /// Checks if the first Operand is greater or equal the second
    GreaterEq,
}

/// A Logic Combination Operation applied to two Operands
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryLogicCombinator {
    /// Checks if both Operands are true
    And,
    /// Checks if at least one of the Operands is true
    Or,
}

/// A Bitwise Operation applied to two Operands
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryBitwiseOp {
    /// Combines the two Operands with a bitwise AND
    And,
    /// Combines the two Operands with a bitwise OR
    Or,
    /// Combines the two Operands with a bitwise XOR
    Xor,
    /// Shifts the Bits left in the first Operand by the amount specified with the second Operand
    ShiftLeft,
    /// Shifts the Bits right in the first Operand by the amount specified with the second Operand
    ShiftRight,
}

/// An Operator that is applied to two Operands at a Time
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    /// Performs arithmetic Operations
    Arith(BinaryArithmeticOp),
    /// Performs logic Operations
    Logic(BinaryLogicOp),
    /// Performs the Combination of two Logic Operands
    LogicCombinator(BinaryLogicCombinator),
    /// Performs bitwise Operations
    Bitwise(BinaryBitwiseOp),
}
