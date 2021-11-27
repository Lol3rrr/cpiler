use general::SpanData;
use syntax::Identifier;

use crate::AType;

#[derive(Debug, PartialEq, Clone)]
pub struct StructMember {
    pub name: Identifier,
    pub ty: AType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    pub members: Vec<StructMember>,
}

impl StructDef {
    pub fn find_member(&self, name: &Identifier) -> Option<SpanData<AType>> {
        self.members
            .iter()
            .find(|memb| memb.name.0.data == name.0.data)
            .map(|memb| SpanData {
                span: memb.name.0.span.clone(),
                data: memb.ty.clone(),
            })
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span, SpanData};

    use crate::APrimitive;

    use super::*;

    #[test]
    fn find_member_valid() {
        let dummy_source = Source::new("test", "testing");

        let struct_def = StructDef {
            members: vec![StructMember {
                name: Identifier(SpanData {
                    span: Span::new_source(dummy_source.clone(), 0..1),
                    data: "t".to_string(),
                }),
                ty: AType::Primitve(APrimitive::Int),
            }],
        };

        let expected = Some(SpanData {
            span: Span::new_source(dummy_source.clone(), 0..1),
            data: AType::Primitve(APrimitive::Int),
        });

        let result = struct_def.find_member(&Identifier(SpanData {
            span: Span::new_source(dummy_source.clone(), 0..1),
            data: "t".to_string(),
        }));

        assert_eq!(expected, result);
    }
}
