use general::SpanData;
use syntax::{DataType, Modifier, TypeToken};

use crate::APrimitive;

#[derive(Debug)]
pub enum BaseTypes {
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
}

impl BaseTypes {
    pub fn parse(ty: TypeToken) -> Option<Self> {
        match ty {
            TypeToken::Primitive(SpanData { data, .. }) => match data {
                DataType::Char => Some(Self::Char),
                DataType::Short => Some(Self::Short),
                DataType::Int => Some(Self::Int),
                DataType::Long => Some(Self::Long),
                DataType::Float => Some(Self::Float),
                DataType::Double => Some(Self::Double),
                _ => None,
            },
            TypeToken::Composition {
                modifier:
                    SpanData {
                        data: Modifier::Long,
                        ..
                    },
                base,
            } => {
                dbg!(&base);
                let base = Self::parse(*base)?;

                match base {
                    BaseTypes::Int => Some(Self::Long),
                    BaseTypes::Long => Some(Self::LongLong),
                    BaseTypes::Double => Some(Self::LongDouble),
                    _ => panic!("Unknown Long modifier for {:?}", base),
                }
            }
            _ => None,
        }
    }

    pub fn to_primitive(self) -> APrimitive {
        match self {
            Self::Char => APrimitive::Char,
            Self::Short => APrimitive::Short,
            Self::Int => APrimitive::Int,
            Self::Long => APrimitive::LongInt,
            Self::LongLong => APrimitive::LongLongInt,
            Self::Float => APrimitive::Float,
            Self::Double => APrimitive::Double,
            Self::LongDouble => APrimitive::LongDouble,
        }
    }
}
