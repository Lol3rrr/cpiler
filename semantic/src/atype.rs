use std::borrow::Borrow;

use syntax::{DataType, Identifier, Modifier, StructMembers, TypeDefType, TypeToken};

use crate::{AExpression, EvaluationValue, SemanticError, TypeDefinitions, VariableContainer};

mod struct_def;
pub use struct_def::*;

mod arith_type;
pub use arith_type::*;

pub mod assign_type;

#[derive(Debug, Clone)]
pub enum AType {
    Primitve(APrimitive),
    Pointer(Box<Self>),
    Struct(StructDef),
    Array(Array),
    Const(Box<Self>),
    TypeDef { name: Identifier, ty: Box<Self> },
}

impl<O> PartialEq<O> for AType
where
    O: Borrow<Self>,
{
    fn eq(&self, other: &O) -> bool {
        match (self, other.borrow()) {
            (Self::Primitve(s_prim), Self::Primitve(o_prim)) => s_prim.eq(o_prim),
            (Self::Pointer(s_inner), Self::Pointer(o_inner)) => s_inner.eq(o_inner),
            (Self::Struct(s_def), Self::Struct(o_def)) => s_def.eq(o_def),
            (Self::Array(s_arr), Self::Array(o_arr)) => s_arr.eq(o_arr),
            (Self::Const(s_c), Self::Const(o_c)) => s_c.eq(o_c),
            (Self::TypeDef { ty: s_ty, .. }, Self::TypeDef { ty: o_ty, .. }) => s_ty.eq(o_ty),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum APrimitive {
    Void,
    Char,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    LongInt,
    UnsignedLongInt,
    LongLongInt,
    UnsignedLongLongInt,
    Float,
    Double,
    LongDouble,
}

impl APrimitive {
    pub fn is_unsigned(&self) -> bool {
        match self {
            Self::UnsignedChar
            | Self::UnsignedShort
            | Self::UnsignedInt
            | Self::UnsignedLongInt
            | Self::UnsignedLongLongInt => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Char | Self::Short | Self::Int | Self::LongInt | Self::LongLongInt => true,
            _ => false,
        }
    }

    pub fn rank(&self) -> Option<usize> {
        match self {
            Self::Char | Self::UnsignedChar => Some(1),
            Self::Short | Self::UnsignedShort => Some(2),
            Self::Int | Self::UnsignedInt => Some(3),
            Self::LongInt | Self::UnsignedLongInt => Some(4),
            Self::LongLongInt | Self::UnsignedLongLongInt => Some(5),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum BaseTypes {
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
}

impl BaseTypes {
    pub fn parse(prim: DataType) -> Option<Self> {
        match prim {
            DataType::Char => Some(Self::Char),
            DataType::Short => Some(Self::Short),
            DataType::Int => Some(Self::Int),
            DataType::Long => Some(Self::Long),
            DataType::Float => Some(Self::Float),
            DataType::Double => Some(Self::Double),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Array {
    pub size: Option<usize>,
    pub ty: Box<AType>,
}

impl AType {
    pub fn ty(&self) -> &Self {
        match self {
            Self::TypeDef { ty, .. } => ty,
            other => other,
        }
    }
    pub fn into_ty(&self) -> &Self {
        match self {
            Self::TypeDef { ty, .. } => ty,
            other => other,
        }
    }

    pub fn get_struct_def(&self) -> Option<&StructDef> {
        match self {
            Self::Struct(def) => Some(&def),
            Self::TypeDef { ty, .. } => ty.get_struct_def(),
            Self::Pointer(inner) => inner.get_struct_def(),
            _ => None,
        }
    }

    pub fn implicitly_castable(&self, target: &Self) -> bool {
        if self == target {
            return true;
        }

        match (self, target) {
            (Self::Array(arr), Self::Pointer(inner)) => &arr.ty == inner,
            (base, Self::Const(inner_target)) => base.implicitly_castable(&inner_target),
            _ => false,
        }
    }

    pub fn parse_struct<VC>(
        members: StructMembers,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        dbg!(&members);

        let str_members_iter = members
            .into_iter()
            .map(|(raw_ty, ident)| (AType::parse(raw_ty, ty_defs, vars), ident));

        let str_members = {
            let mut tmp = Vec::new();

            for (str_res, ident) in str_members_iter {
                let str_ty = str_res?;
                tmp.push(StructMember {
                    ty: str_ty,
                    name: ident,
                });
            }

            tmp
        };

        Ok(Self::Struct(StructDef {
            members: str_members,
        }))
    }

    pub fn parse<VC>(
        raw: TypeToken,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        match raw {
            TypeToken::Primitive(prim) => {
                let prim_ty = match prim.data {
                    DataType::Void => APrimitive::Void,
                    DataType::Char => APrimitive::Char,
                    DataType::Short => APrimitive::Short,
                    DataType::Int => APrimitive::Int,
                    DataType::Float => APrimitive::Float,
                    DataType::Double => APrimitive::Double,
                    DataType::Long => APrimitive::LongInt,
                    unknwon => panic!("Unknown Primitive: {:?}", unknwon),
                };
                Ok(Self::Primitve(prim_ty))
            }
            TypeToken::Pointer(ptr) => {
                let inner = Self::parse(*ptr, ty_defs, vars)?;

                Ok(Self::Pointer(Box::new(inner)))
            }
            TypeToken::ArrayType { size, base } => {
                let base_ty = Self::parse(*base, ty_defs, vars)?;

                let base_size = match size {
                    Some(s) => {
                        let a_exp = AExpression::parse(*s, ty_defs, vars)?;

                        let size = match a_exp.const_evaluate() {
                            Ok(v) => match v {
                                EvaluationValue::Integer(s) => s,
                                EvaluationValue::FloatingPoint(_) => {
                                    panic!("Size cannot be floating point")
                                }
                            },
                            Err(e) => {
                                todo!("Evaluating Expression: {:?}", e);
                            }
                        };

                        if size >= 0 {
                            Some(size as usize)
                        } else {
                            panic!("Size cannot be negative");
                        }
                    }
                    None => None,
                };

                Ok(Self::Array(Array {
                    size: base_size,
                    ty: Box::new(base_ty),
                }))
            }
            TypeToken::TypeDefed { name } => {
                dbg!(&name);

                match ty_defs.get_definition(&name) {
                    Some(ty) => Ok(Self::TypeDef {
                        name,
                        ty: Box::new(ty.clone()),
                    }),
                    None => {
                        dbg!(&ty_defs);
                        todo!("Unknown Typename: {:?}", &name);
                    }
                }
            }
            TypeToken::Composition { base, modifier } => {
                dbg!(&base, &modifier);

                match (*base, modifier.data) {
                    (TypeToken::Primitive(base_span), Modifier::Signed) => {
                        let base_ty = BaseTypes::parse(base_span.data).unwrap();

                        match base_ty {
                            BaseTypes::Char => Ok(Self::Primitve(APrimitive::Char)),
                            BaseTypes::Short => Ok(Self::Primitve(APrimitive::Short)),
                            BaseTypes::Int => Ok(Self::Primitve(APrimitive::Int)),
                            BaseTypes::Long => Ok(Self::Primitve(APrimitive::LongInt)),
                            BaseTypes::Float => panic!("Cant have a signed Float"),
                            BaseTypes::Double => panic!("Cant have a signed Double"),
                        }
                    }
                    (TypeToken::Primitive(base_span), Modifier::Unsigned) => {
                        let base_ty = BaseTypes::parse(base_span.data).unwrap();

                        match base_ty {
                            BaseTypes::Char => Ok(Self::Primitve(APrimitive::UnsignedChar)),
                            BaseTypes::Short => Ok(Self::Primitve(APrimitive::UnsignedShort)),
                            BaseTypes::Int => Ok(Self::Primitve(APrimitive::UnsignedInt)),
                            BaseTypes::Long => Ok(Self::Primitve(APrimitive::LongInt)),
                            BaseTypes::Float => panic!("Cant have a unsigned Float"),
                            BaseTypes::Double => panic!("Cant have a unsigned Double"),
                        }
                    }
                    (TypeToken::Primitive(base_span), Modifier::Long) => {
                        let base_ty = BaseTypes::parse(base_span.data).unwrap();

                        match base_ty {
                            BaseTypes::Char => panic!("Cant have a long char"),
                            BaseTypes::Short => panic!("Cant have a long short"),
                            BaseTypes::Int => Ok(Self::Primitve(APrimitive::LongInt)),
                            BaseTypes::Long => Ok(Self::Primitve(APrimitive::LongInt)),
                            BaseTypes::Float => panic!("Cant have a long Float"),
                            BaseTypes::Double => Ok(Self::Primitve(APrimitive::LongDouble)),
                        }
                    }
                    (raw_ty, Modifier::Const) => {
                        dbg!(&raw_ty);

                        let ty = AType::parse(raw_ty, ty_defs, vars)?;
                        dbg!(&ty);

                        Ok(Self::Const(Box::new(ty)))
                    }
                    (base, modif) => panic!("Unknown Combination of {:?} with {:?}", modif, base),
                }
            }
            TypeToken::StructType { name } => {
                dbg!(&name);

                let target_ty = match ty_defs.get_definition(&name) {
                    Some(t) => t,
                    None => {
                        panic!("No known Type-Definition for the given Name");
                    }
                };
                dbg!(&target_ty);

                match &target_ty {
                    AType::Struct(_) => {}
                    other => {
                        dbg!(&other);

                        panic!("Expected a Struct-Type");
                    }
                };

                Ok(target_ty.clone())
            }
            unknown => panic!("Unknown TypeToken: {:?}", unknown),
        }
    }

    pub fn parse_typedef<VC>(
        raw: TypeDefType,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        match raw {
            TypeDefType::StructdDef { name, members } => {
                dbg!(&name, &members);

                let str_members: Vec<_> = members
                    .into_iter()
                    .filter_map(
                        |(raw_ty, ident)| match AType::parse(raw_ty, ty_defs, vars) {
                            Ok(ty) => Some(StructMember { name: ident, ty }),
                            _ => None,
                        },
                    )
                    .collect();

                let struct_def = StructDef {
                    members: str_members,
                };

                match name {
                    Some(n) => {
                        todo!("Named Struct");
                    }
                    None => Ok(Self::Struct(struct_def)),
                }
            }
            unknown => panic!("Unknown TypeDefType: {:?}", unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use general::{Source, Span, SpanData};
    use syntax::Modifier;

    use super::*;

    #[test]
    fn useable_array_pointer() {
        let root = AType::Array(Array {
            ty: Box::new(AType::Primitve(APrimitive::Int)),
            size: None,
        });

        let target = AType::Pointer(Box::new(AType::Primitve(APrimitive::Int)));

        assert_eq!(true, root.implicitly_castable(&target));
    }

    #[test]
    fn parse_const_ptr() {
        let source = Source::new("test", "const int*");
        let base = TypeToken::Composition {
            base: Box::new(TypeToken::Pointer(Box::new(TypeToken::Primitive(
                SpanData {
                    span: Span::new_source(source.clone(), 6..9),
                    data: DataType::Int,
                },
            )))),
            modifier: SpanData {
                span: Span::new_source(source.clone(), 0..5),
                data: Modifier::Const,
            },
        };

        let expected = Ok(AType::Const(Box::new(AType::Pointer(Box::new(
            AType::Primitve(APrimitive::Int),
        )))));

        let result = AType::parse(base, &TypeDefinitions::new(), &HashMap::new());

        assert_eq!(expected, result);
    }

    #[test]
    fn primitives() {
        let input_source = Source::new("test", " ");

        // Char Stuff
        assert_eq!(
            Ok(AType::Primitve(APrimitive::Char)),
            AType::parse(
                TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: DataType::Char,
                }),
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );
        assert_eq!(
            Ok(AType::Primitve(APrimitive::Char)),
            AType::parse(
                TypeToken::Composition {
                    modifier: SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: Modifier::Signed,
                    },
                    base: Box::new(TypeToken::Primitive(SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: DataType::Char,
                    })),
                },
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );
        assert_eq!(
            Ok(AType::Primitve(APrimitive::UnsignedChar)),
            AType::parse(
                TypeToken::Composition {
                    modifier: SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: Modifier::Unsigned,
                    },
                    base: Box::new(TypeToken::Primitive(SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: DataType::Char,
                    })),
                },
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );

        // Short stuff
        assert_eq!(
            Ok(AType::Primitve(APrimitive::Short)),
            AType::parse(
                TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: DataType::Short,
                }),
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );
        assert_eq!(
            Ok(AType::Primitve(APrimitive::Short)),
            AType::parse(
                TypeToken::Composition {
                    modifier: SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: Modifier::Signed,
                    },
                    base: Box::new(TypeToken::Primitive(SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: DataType::Short,
                    })),
                },
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );

        // Double stuff
        assert_eq!(
            Ok(AType::Primitve(APrimitive::Double)),
            AType::parse(
                TypeToken::Primitive(SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: DataType::Double,
                }),
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );
        assert_eq!(
            Ok(AType::Primitve(APrimitive::LongDouble)),
            AType::parse(
                TypeToken::Composition {
                    modifier: SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: Modifier::Long,
                    },
                    base: Box::new(TypeToken::Primitive(SpanData {
                        span: Span::new_source(input_source.clone(), 0..1),
                        data: DataType::Double,
                    })),
                },
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        );
    }
}
