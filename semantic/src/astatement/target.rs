use general::{Span, SpanData};
use ir::BasicBlock;
use syntax::{AssignTarget, Identifier};

use crate::{
    conversion::ConvertContext, AExpression, APrimitive, AType, SemanticError, TypeDefinitions,
    VariableContainer,
};

#[derive(Debug, PartialEq)]
pub struct ArrayAccessTarget {
    target: Box<AAssignTarget>,
    index: Box<AExpression>,
    ty_info: SpanData<AType>,
}

impl ArrayAccessTarget {
    pub fn to_exp(self, block: &BasicBlock, ctx: &ConvertContext) -> ir::Expression {
        let base_target = match *self.target {
            AAssignTarget::Variable { ident, ty_info } => {
                assert!(matches!(&ty_info.data, AType::Array(_)));

                AExpression::Variable {
                    ident,
                    ty: ty_info.data,
                }
            }
            other => {
                dbg!(&other);

                todo!()
            }
        };

        let elem_size = self.ty_info.data.byte_size(ctx.arch());

        let index_value = self.index.to_ir(block, ctx);
        let index_oper = AExpression::val_to_operand(index_value, block, ctx);
        dbg!(&index_oper);

        let offset_exp = ir::Expression::BinaryOp {
            op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Multiply),
            left: index_oper,
            right: ir::Operand::Constant(ir::Constant::I64(elem_size as i64)),
        };
        let offset_value = ir::Value::Expression(offset_exp);
        let offset_oper = AExpression::val_to_operand(offset_value, block, ctx);

        let base_value = base_target.to_ir(block, ctx);
        let base_oper = AExpression::val_to_operand(base_value, block, ctx);

        ir::Expression::BinaryOp {
            op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
            left: base_oper,
            right: offset_oper,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StructFieldTarget {
    /// The underlying Struct Target, like a Variable or Pointer
    pub target: Box<AAssignTarget>,
    /// The Name of the Field
    pub field: Identifier,
    /// The Type of the Field of the Struct
    pub ty_info: SpanData<AType>,
}

impl StructFieldTarget {
    pub fn to_exp(self, block: &BasicBlock, ctx: &ConvertContext) -> ir::Expression {
        let (base_target, struct_def) = match *self.target {
            AAssignTarget::Variable { ident, ty_info } => {
                dbg!(&ty_info);
                match ty_info.data {
                    AType::Struct(def) => {
                        let var = AExpression::Variable {
                            ident,
                            ty: AType::Struct(def.clone()),
                        };

                        (var, def)
                    }
                    _ => panic!("Exected Struct Ptr"),
                }
            }
            other => {
                dbg!(&other);

                todo!()
            }
        };
        dbg!(&base_target, &struct_def);

        let offset = struct_def
            .member_offset(&self.field.0.data, ctx.arch())
            .expect("Field does not exist");

        let base_value = base_target.to_ir(block, ctx);
        let base_oper = AExpression::val_to_operand(base_value, block, ctx);
        dbg!(&base_oper);

        ir::Expression::BinaryOp {
            op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
            left: base_oper,
            right: ir::Operand::Constant(ir::Constant::I64(offset as i64)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AAssignTarget {
    Variable {
        ident: Identifier,
        /// The Type of the Variable itself
        ty_info: SpanData<AType>,
    },
    Deref {
        /// The Target Address to write the Value to
        exp: AExpression,
        /// The Type that is stored that that Address
        ty_info: SpanData<AType>,
    },
    ArrayAccess(ArrayAccessTarget),
    StructField(StructFieldTarget),
}

impl AAssignTarget {
    fn base_ty(&self) -> &AType {
        match self {
            Self::Variable { ty_info, .. } => &ty_info.data,
            Self::Deref { ty_info, .. } => &ty_info.data,
            Self::ArrayAccess(ArrayAccessTarget { ty_info, .. }) => &ty_info.data,
            Self::StructField(StructFieldTarget { ty_info, .. }) => &ty_info.data,
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

                Ok(AAssignTarget::ArrayAccess(ArrayAccessTarget {
                    target: Box::new(base_target),
                    index: Box::new(a_index),
                    ty_info: SpanData {
                        span: elem_span,
                        data: *elem_ty,
                    },
                }))
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

                Ok(Self::StructField(StructFieldTarget {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty,
                    },
                }))
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

                Ok(Self::StructField(StructFieldTarget {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty,
                    },
                }))
            }
        }
    }

    pub fn get_expected_type(&self) -> (AType, Span) {
        match &self {
            Self::Variable { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::Deref { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::ArrayAccess(ArrayAccessTarget { ty_info, .. }) => {
                (ty_info.data.clone(), ty_info.span.clone())
            }
            Self::StructField(StructFieldTarget { ty_info, .. }) => {
                (ty_info.data.clone(), ty_info.span.clone())
            }
        }
    }
}
