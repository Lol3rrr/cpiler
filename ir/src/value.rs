use crate::{general, Expression, Operand, Type, Variable, VariableMetadata, WeakBlockPtr};

/// This holds the Information for a single Source for a PhiNode
pub type PhiEntry = general::PhiEntry<WeakBlockPtr>;

/// A Value that will be assigned to a Variable
pub type Value = general::Value<WeakBlockPtr>;

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
                var: Box::new(base_var.name().to_string()),
            },
            (_, VariableMetadata::VarPointer { .. }) => VariableMetadata::Pointer,
            _ => target_meta.clone(),
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
