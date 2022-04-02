use general::{Span, SpanData};

use crate::{AExpression, APrimitive, AType, SemanticError};

pub fn determine_type(
    base: AExpression,
    target: (&AType, &Span),
) -> Result<AExpression, SemanticError> {
    let res_type = base.result_type();
    if res_type == target.0 {
        return Ok(base);
    }

    match (&res_type, target.0) {
        (AType::Primitve(res_prim), AType::Primitve(target_prim)) => {
            match (&target_prim, &res_prim) {
                (APrimitive::Float, APrimitive::Float)
                | (APrimitive::Double, APrimitive::Float)
                | (APrimitive::LongDouble, APrimitive::Float)
                | (APrimitive::Double, APrimitive::Double)
                | (APrimitive::LongDouble, APrimitive::Double)
                | (APrimitive::LongDouble, APrimitive::LongDouble) => {}
                (_, APrimitive::Float) | (_, APrimitive::Double) | (_, APrimitive::LongDouble) => {
                    return Err(SemanticError::AmbiguousTypeConversion {
                        target: SpanData {
                            span: target.1.clone(),
                            data: target.0.clone(),
                        },
                        base: SpanData {
                            span: base.entire_span(),
                            data: res_type,
                        },
                    });
                }
                _ => {}
            };

            // TODO
            // This currently allows for some very bad implicit casts, like float to int
            let casted = AExpression::Cast {
                target: target.0.clone(),
                base: Box::new(base),
            };

            Ok(casted)
        }
        (AType::Pointer(_), AType::Pointer(target_val))
            if *target_val.into_ty() == AType::Primitve(APrimitive::Void) =>
        {
            let casted = AExpression::Cast {
                target: target.0.clone(),
                base: Box::new(base),
            };

            Ok(casted)
        }
        (AType::Pointer(target_val), AType::Pointer(_))
            if *target_val.into_ty() == AType::Primitve(APrimitive::Void) =>
        {
            let casted = AExpression::Cast {
                target: target.0.clone(),
                base: Box::new(base),
            };

            Ok(casted)
        }
        (AType::Array(arr_ty), AType::Pointer(ptr_ty)) if arr_ty.ty.eq(ptr_ty) => {
            let casted = AExpression::Cast {
                target: target.0.clone(),
                base: Box::new(base),
            };

            Ok(casted)
        }
        (_, AType::Const(const_ty)) => determine_type(base, (const_ty, target.1)),
        _ => Err(SemanticError::MismatchedTypes {
            expected: SpanData {
                span: target.1.clone(),
                data: target.0.clone(),
            },
            received: SpanData {
                span: base.entire_span(),
                data: base.result_type(),
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, SpanData};

    use crate::{APrimitive, Literal};

    use super::*;

    #[test]
    fn matching_types() {
        let input_source = Source::new("test", " ");

        assert_eq!(
            Ok(AExpression::Literal(Literal::Integer(SpanData {
                span: Span::new_source(input_source.clone(), 0..1),
                data: 0,
            }))),
            determine_type(
                AExpression::Literal(Literal::Integer(SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: 0,
                })),
                (
                    &AType::Primitve(APrimitive::LongInt),
                    &Span::new_source(input_source, 0..1)
                )
            )
        );
    }

    #[test]
    fn mismatched_types_float_to_int() {
        let input_source = Source::new("test", " ");

        assert_eq!(
            Err(SemanticError::AmbiguousTypeConversion {
                base: SpanData {
                    data: AType::Primitve(APrimitive::Float),
                    span: Span::new_source(input_source.clone(), 0..1),
                },
                target: SpanData {
                    data: AType::Primitve(APrimitive::Int),
                    span: Span::new_source(input_source.clone(), 0..1),
                },
            }),
            determine_type(
                AExpression::Literal(Literal::FloatingPoint(SpanData {
                    span: Span::new_source(input_source.clone(), 0..1),
                    data: 1.3,
                })),
                (
                    &AType::Primitve(APrimitive::Int),
                    &Span::new_source(input_source, 0..1)
                )
            )
        );
    }
}
