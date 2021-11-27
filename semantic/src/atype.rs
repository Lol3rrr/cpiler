use syntax::{DataType, Modifier, TypeDefType, TypeToken};

use crate::{AExpression, EvaluationValue, SemanticError, TypeDefinitions, VariableContainer};

mod struct_def;
pub use struct_def::*;

#[derive(Debug, PartialEq, Clone)]
pub enum AType {
    Primitve(APrimitive),
    Pointer(Box<Self>),
    AnonStruct(StructDef),
    Array(Array),
    Composition(Modifier, Box<Self>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum APrimitive {
    Void,
    Int,
    Char,
    Float,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Array {
    pub size: Option<usize>,
    pub ty: Box<AType>,
}

impl AType {
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
                    DataType::Int => APrimitive::Int,
                    DataType::Char => APrimitive::Char,
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
                        let a_exp = AExpression::parse(*s, vars)?;

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
                    Some(ty) => Ok(ty.clone()),
                    None => {
                        dbg!(&ty_defs);
                        todo!("Unknown Typename: {:?}", &name);
                    }
                }
            }
            TypeToken::Composition { base, modifier } => {
                dbg!(&base, &modifier);

                let base_ty = Self::parse(*base, ty_defs, vars)?;
                dbg!(&base_ty);

                Ok(Self::Composition(modifier.data, Box::new(base_ty)))
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
                    None => Ok(Self::AnonStruct(struct_def)),
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

    use super::*;

    #[test]
    fn parse_primitives() {
        let source = Source::new("test", "int");
        assert_eq!(
            Ok(AType::Primitve(APrimitive::Int)),
            AType::parse(
                TypeToken::Primitive(SpanData {
                    span: Span::new_source(source, 0..4),
                    data: DataType::Int,
                }),
                &TypeDefinitions::new(),
                &HashMap::new()
            )
        )
    }
}
