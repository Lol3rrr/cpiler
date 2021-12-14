use general::{Span, SpanData};
use syntax::{AssignTarget, Identifier};

use crate::{AExpression, APrimitive, AType, SemanticError, TypeDefinitions, VariableContainer};

#[derive(Debug, PartialEq)]
pub enum AAssignTarget {
    Variable {
        ident: Identifier,
        /// The Type of the Variable itself
        ty_info: SpanData<AType>,
    },
    ArrayAccess {
        target: Box<Self>,
        index: Box<AExpression>,
        /// The Type of the Array-Element
        ty_info: SpanData<AType>,
    },
    StructField {
        target: Box<Self>,
        field: Identifier,
        /// The Type of the Field of the Struct
        ty_info: SpanData<AType>,
    },
}

impl AAssignTarget {
    fn base_ty(&self) -> &AType {
        match self {
            Self::Variable { ty_info, .. } => &ty_info.data,
            Self::ArrayAccess { ty_info, .. } => &ty_info.data,
            Self::StructField { ty_info, .. } => &ty_info.data,
        }
    }

    pub fn parse<VC>(
        raw: AssignTarget,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        match raw {
            AssignTarget::Variable(ident) => {
                let (ty, sp) = match vars.get_var(&ident) {
                    Some(t) => t,
                    None => return Err(SemanticError::UnknownIdentifier { name: ident }),
                };

                Ok(AAssignTarget::Variable {
                    ident,
                    ty_info: SpanData {
                        span: sp.clone(),
                        data: ty.clone(),
                    },
                })
            }
            AssignTarget::ArrayAccess { base, index } => {
                let base_target = Self::parse(*base, ty_defs, vars)?;

                let a_index = AExpression::parse(index, ty_defs, vars)?;

                let index_type = a_index.result_type();
                match index_type {
                    AType::Primitve(APrimitive::Int) | AType::Primitve(APrimitive::LongInt) => {}
                    i_type => {
                        let i_span = a_index.entire_span();
                        dbg!(&i_type, &i_span);

                        todo!("Invalid Index-Type");
                    }
                };

                let (arr_ty, arr_span) = base_target.get_expected_type();
                let elem_ty = match arr_ty {
                    AType::Array(base) => base.ty,
                    AType::Pointer(base) => base,
                    other => {
                        dbg!(&other);

                        todo!("Indexed type is not an array");
                    }
                };

                // TODO
                // This span is actually pretty bad because as far as I can tell it just describes
                // the name of the array Variable and not the underlying Type
                let elem_span = arr_span;

                Ok(AAssignTarget::ArrayAccess {
                    target: Box::new(base_target),
                    index: Box::new(a_index),
                    ty_info: SpanData {
                        span: elem_span,
                        data: *elem_ty,
                    },
                })
            }
            AssignTarget::StructAccess { base, field } => {
                dbg!(&base, &field);

                let base_target = Self::parse(*base, ty_defs, vars)?;
                dbg!(&base_target);

                let (base_ty, _) = base_target.get_expected_type();

                let struct_def = match base_ty.get_struct_def() {
                    Some(s) => s,
                    None => {
                        dbg!(&base_ty);

                        todo!("Expected Struct");
                    }
                };
                dbg!(&struct_def);

                let (field_ty, field_span) = match struct_def.find_member(&field) {
                    Some(f) => (f.data, f.span),
                    None => {
                        todo!("Unknown field")
                    }
                };
                dbg!(&field_ty);

                Ok(Self::StructField {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty,
                    },
                })
            }
            AssignTarget::StructPtrAccess { base, field } => {
                dbg!(&base, &field);

                let base_target = Self::parse(*base, ty_defs, vars)?;
                dbg!(&base_target);

                let struct_def = match base_target.base_ty() {
                    AType::Pointer(inner) => match inner.get_struct_def() {
                        Some(s) => s,
                        None => {
                            dbg!(&inner);

                            todo!("Expected a Struct-Pointer");
                        }
                    },
                    other => {
                        dbg!(&other);

                        todo!("Expected a Struct-Pointer");
                    }
                };
                dbg!(&struct_def);

                let (field_ty, field_span) = match struct_def.find_member(&field) {
                    Some(f) => (f.data, f.span),
                    None => {
                        todo!("Unknown field");
                    }
                };
                dbg!(&field_ty);

                Ok(Self::StructField {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty,
                    },
                })
            }
        }
    }

    pub fn get_expected_type(&self) -> (AType, Span) {
        match &self {
            Self::Variable { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::ArrayAccess { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::StructField { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
        }
    }
}
