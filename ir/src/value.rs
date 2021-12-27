use crate::{Expression, Operand, Type, Variable, VariableMetadata, WeakBlockPtr};

/// This holds the Information for a single Source for a PhiNode
#[derive(Debug, Clone)]
pub struct PhiEntry {
    /// The Block in which this Variable definition can be found
    pub block: WeakBlockPtr,
    /// The Variable found
    pub var: Variable,
}

impl PartialEq for PhiEntry {
    fn eq(&self, other: &Self) -> bool {
        self.var == other.var
    }
}

/// A Value that will be assigned to a Variable
#[derive(Debug, Clone)]
pub enum Value {
    /// The Value is unknown at compile-time, like arguments for a Function
    Unknown,
    /// The Value is a constant and known at compile-time
    Constant(Constant),
    /// The Value is the same as the Value of the Variable
    Variable(Variable),
    /// The Value is one of the Variables depending on which BasicBlock we came to this Point in
    /// the Program
    Phi {
        /// The Different sources for the Value of this Value
        sources: Vec<PhiEntry>,
    },
    /// The Value can be obtained from the given Expression
    Expression(Expression),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::Constant(s_c), Self::Constant(o_c)) => s_c == o_c,
            (Self::Variable(s_var), Self::Variable(o_var)) => s_var == o_var,
            (Self::Phi { sources: s_sources }, Self::Phi { sources: o_sources }) => {
                s_sources == o_sources
            }
            (Self::Expression(s_exp), Self::Expression(o_exp)) => s_exp == o_exp,
            _ => false,
        }
    }
}

impl Value {
    /// Generates the correct Variable-Metadata
    pub fn assign_meta(&self, target: &Variable) -> VariableMetadata {
        let target_meta = target.meta();

        match (self, target_meta) {
            (
                Self::Expression(Expression::AdressOf {
                    base: Operand::Variable(base_var),
                }),
                _,
            ) => VariableMetadata::VarPointer {
                var: Box::new(base_var.clone()),
            },
            (_, VariableMetadata::VarPointer { .. }) => VariableMetadata::Pointer,
            _ => target_meta.clone(),
        }
    }

    /// Returns a list of all the Variables used by this Value
    pub fn used_vars(&self) -> Vec<Variable> {
        match self {
            Self::Unknown => Vec::new(),
            Self::Constant(_) => Vec::new(),
            Self::Expression(exp) => exp.used_vars(),
            Self::Variable(var) => vec![var.clone()],
            Self::Phi { sources } => sources.iter().map(|entry| entry.var.clone()).collect(),
        }
    }
}

/// A Constant Value that is already known at compile-time
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    /// 8 bit signed integer
    I8(i8),
    /// 16 bit signed integer
    I16(i16),
    /// 32 bit signed integer
    I32(i32),
    /// 64 bit signed integer
    I64(i64),
    /// 8 bit unsigned integer
    U8(u8),
    /// 16 bit unsigned integer
    U16(u16),
    /// 32 bit unsigned integer
    U32(u32),
    /// 64 bit unsigned integer
    U64(u64),
    /// 32 bit floating Point Number
    F32(f32),
    /// 64 bit floating Point Number
    F64(f64),
}

impl Constant {
    /// Returns the Type corresponding to the Constant
    pub fn ty(&self) -> Type {
        match self {
            Self::I8(_) => Type::I8,
            Self::I16(_) => Type::I16,
            Self::I32(_) => Type::I32,
            Self::I64(_) => Type::I64,
            Self::U8(_) => Type::U8,
            Self::U16(_) => Type::U16,
            Self::U32(_) => Type::U32,
            Self::U64(_) => Type::U64,
            Self::F32(_) => Type::Float,
            Self::F64(_) => Type::Double,
        }
    }
}
