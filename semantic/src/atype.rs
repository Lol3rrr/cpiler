use std::borrow::Borrow;

use general::{Span, SpanData};
use syntax::{DataType, Identifier, Modifier, StructMembers, TypeDefType, TypeToken};

use crate::{AExpression, EvaluationValue, SemanticError, TypeDefinitions, VariableContainer};

mod struct_def;
pub use struct_def::*;

mod arith_type;
pub use arith_type::*;

pub mod assign_type;

mod base;
use base::BaseTypes;

#[derive(Debug, Clone)]
pub enum AType {
    Primitve(APrimitive),
    Pointer(Box<Self>),
    Struct { def: StructDef, area: Span },
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
            (
                Self::Struct {
                    def: s_def,
                    area: s_area,
                },
                Self::Struct {
                    def: o_def,
                    area: o_area,
                },
            ) => s_def.eq(o_def) && s_area.eq(o_area),
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
        matches!(
            self,
            Self::UnsignedChar
                | Self::UnsignedShort
                | Self::UnsignedInt
                | Self::UnsignedLongInt
                | Self::UnsignedLongLongInt
        )
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::Char | Self::Short | Self::Int | Self::LongInt | Self::LongLongInt
        )
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

    pub fn byte_size(&self) -> u64 {
        match self {
            Self::Char | Self::UnsignedChar => 1,
            Self::Short | Self::UnsignedShort => 2,
            Self::Int | Self::UnsignedInt => 4,
            _ => todo!("Size of {:?} in Bytes", self),
        }
    }

    pub fn alignment(&self) -> u64 {
        match self {
            Self::Char | Self::UnsignedChar => 1,
            Self::Short | Self::UnsignedShort => 2,
            Self::Int | Self::UnsignedInt => 4,
            _ => todo!("Size of {:?} in Bytes", self),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Array {
    pub size: Option<usize>,
    pub ty: Box<AType>,
}

impl AType {
    #[must_use]
    pub fn ty(self) -> Self {
        match self {
            Self::TypeDef { ty, .. } => *ty,
            other => other,
        }
    }
    pub fn into_ty(&self) -> &Self {
        match self {
            Self::TypeDef { ty, .. } => ty,
            other => other,
        }
    }

    pub fn get_struct_def(&self) -> Option<(&StructDef, &Span)> {
        match self {
            Self::Struct { def, area } => Some((def, area)),
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
            (base, Self::Const(inner_target)) => base.implicitly_castable(inner_target),
            _ => false,
        }
    }

    pub fn parse_struct<VC>(
        members: StructMembers,
        entire_span: Span,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
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

        Ok(Self::Struct {
            def: StructDef {
                members: str_members,
            },
            area: entire_span,
        })
    }

    fn parse_composition<VC>(
        modifier: SpanData<Modifier>,
        base: Box<TypeToken>,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        dbg!(&base, &modifier);

        match (*base, modifier.data) {
            (base, Modifier::Signed) => {
                let base_ty = BaseTypes::parse(base).unwrap();

                match base_ty {
                    BaseTypes::Char => Ok(Self::Primitve(APrimitive::Char)),
                    BaseTypes::Short => Ok(Self::Primitve(APrimitive::Short)),
                    BaseTypes::Int => Ok(Self::Primitve(APrimitive::Int)),
                    BaseTypes::Long => Ok(Self::Primitve(APrimitive::LongInt)),
                    BaseTypes::LongLong => Ok(Self::Primitve(APrimitive::LongLongInt)),
                    BaseTypes::Float => panic!("Cant have a signed Float"),
                    BaseTypes::Double => panic!("Cant have a signed Double"),
                    BaseTypes::LongDouble => panic!("Cant have a signed LongDouble"),
                }
            }
            (base, Modifier::Unsigned) => {
                let base_ty = BaseTypes::parse(base).unwrap();

                match base_ty {
                    BaseTypes::Char => Ok(Self::Primitve(APrimitive::UnsignedChar)),
                    BaseTypes::Short => Ok(Self::Primitve(APrimitive::UnsignedShort)),
                    BaseTypes::Int => Ok(Self::Primitve(APrimitive::UnsignedInt)),
                    BaseTypes::Long => Ok(Self::Primitve(APrimitive::UnsignedLongInt)),
                    BaseTypes::LongLong => Ok(Self::Primitve(APrimitive::UnsignedLongLongInt)),
                    BaseTypes::Float => panic!("Cant have a unsigned Float"),
                    BaseTypes::Double => panic!("Cant have a unsigned Double"),
                    BaseTypes::LongDouble => panic!("Cant have a unsigned LongDouble"),
                }
            }
            (raw_ty, Modifier::Const) => {
                dbg!(&raw_ty);

                let ty = AType::parse(raw_ty, ty_defs, vars)?;
                dbg!(&ty);

                Ok(Self::Const(Box::new(ty)))
            }
            (base, Modifier::Long) => {
                let tmp_tok = TypeToken::Composition {
                    base: Box::new(base),
                    modifier: SpanData {
                        span: modifier.span,
                        data: Modifier::Long,
                    },
                };

                let base_ty = BaseTypes::parse(tmp_tok).unwrap();

                let prim = base_ty.to_primitive();
                Ok(Self::Primitve(prim))
            }
        }
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
            TypeToken::TypeDefed { name } => match ty_defs.get_definition(&name) {
                Some(ty) => Ok(Self::TypeDef {
                    name,
                    ty: Box::new(ty.clone()),
                }),
                None => {
                    return Err(SemanticError::UnknownType { name });
                }
            },
            TypeToken::Composition { base, modifier } => {
                Self::parse_composition(modifier, base, ty_defs, vars)
            }
            TypeToken::StructType { name } => {
                let target_ty = match ty_defs.get_definition(&name) {
                    Some(t) => t,
                    None => {
                        panic!("No known Type-Definition for the given Name");
                    }
                };

                match &target_ty {
                    AType::Struct { .. } => {}
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
            TypeDefType::StructdDef {
                name,
                members,
                entire_span,
            } => {
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
                        dbg!(&n);
                        todo!("Named Struct");
                    }
                    None => Ok(Self::Struct {
                        def: struct_def,
                        area: entire_span,
                    }),
                }
            }
            TypeDefType::Type(inner_type) => {
                let ty = Self::parse(inner_type, ty_defs, vars)?;
                dbg!(&ty);

                todo!()
            }
            TypeDefType::NamedStruct { name } => {
                dbg!(&name);

                todo!()
            }
        }
    }

    pub fn to_ir(self) -> ir::Type {
        match self {
            Self::Primitve(prim) => match prim {
                APrimitive::Void => ir::Type::Void,
                APrimitive::Char => ir::Type::I8,
                APrimitive::Short => ir::Type::I16,
                APrimitive::Int => ir::Type::I32,
                APrimitive::LongInt => ir::Type::I64,
                APrimitive::LongLongInt => todo!("Unsupported 128 bit"),
                APrimitive::Float => ir::Type::Float,
                APrimitive::Double => ir::Type::Double,
                APrimitive::UnsignedInt => ir::Type::U32,
                other => {
                    dbg!(&other);

                    todo!("Unkown Primitve Conversion")
                }
            },
            Self::Pointer(raw_inner) => {
                let inner = raw_inner.to_ir();
                ir::Type::Pointer(Box::new(inner))
            }
            Self::Array(arr) => ir::Type::Pointer(Box::new(arr.ty.to_ir())),
            Self::Struct { .. } => ir::Type::Pointer(Box::new(ir::Type::Void)),
            Self::TypeDef { ty, .. } => ty.to_ir(),
            other => {
                dbg!(&other);

                todo!("Unknown type conversion")
            }
        }
    }

    pub fn byte_size(&self, arch: &general::arch::Arch) -> u64 {
        match self {
            Self::Primitve(prim) => prim.byte_size(),
            Self::Pointer(_) => arch.ptr_size() as u64,
            Self::Array(arr) => arr.ty.byte_size(arch) * (arr.size.unwrap() as u64),
            Self::Struct { def, .. } => def.entire_size(arch) as u64,
            Self::TypeDef { ty, .. } => ty.byte_size(arch),
            _ => todo!("Size of {:?} in Bytes", self),
        }
    }

    pub fn alignment(&self, arch: &general::arch::Arch) -> u64 {
        match self {
            Self::Primitve(prim) => prim.alignment(),
            Self::Pointer(_) => arch.ptr_size() as u64,
            Self::Array(arr) => arr.ty.alignment(arch),
            Self::Struct { def, .. } => def.alignment(arch) as u64,
            Self::TypeDef { ty, .. } => ty.alignment(arch),
            _ => todo!("Alignment of {:?} in Bytes", self),
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
