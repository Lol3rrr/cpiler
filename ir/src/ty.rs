/// The Types that can be used for Variables
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Type {
    /// A non existing Type that is 0 sized
    Void,
    /// A signed 8 bit integer
    I8,
    /// A signed 16 bit integer
    I16,
    /// A signed 32 bit integer
    I32,
    /// A signed 64 bit integer
    I64,
    /// An unsigned 8 bit integer
    U8,
    /// An unsigned 16 bit integer
    U16,
    /// An unsigned 32 bit integer
    U32,
    /// An unsigned 64 bit integer
    U64,
    /// A 32 bit floating point number
    Float,
    /// A 64 bit floating point number
    Double,
    /// A 80 bit floating point number
    LongDouble,
    /// A Pointer to a another Type
    Pointer(Box<Self>),
    /// An Array of some Type with a given Size
    Array(Box<Self>, usize),
    /// A Struct with the given members
    Struct {
        /// The Members of the Struct
        members: Vec<(String, Type)>,
    },
}

impl Type {
    /// Checks if the given Type is a Pointer Type, which would currently be either a Pointer
    /// or an Array
    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Pointer(_) | Self::Array(_, _))
    }
}
