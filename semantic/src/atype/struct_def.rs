use general::{arch::Arch, SpanData};
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

    pub fn entire_size(&self, arch: &Arch) -> usize {
        let mut size = 0;
        for memb in self.members.iter() {
            let memb_type = &memb.ty;
            let memb_size = memb_type.byte_size(arch) as usize;
            let memb_align = memb_type.alignment(arch) as usize;

            let align_rest = size % memb_align;
            if align_rest == 0 {
                size += memb_size;
            } else {
                size = size - align_rest + memb_align;
                size += memb_size;
            }
        }

        size
    }

    pub fn alignment(&self, arch: &Arch) -> usize {
        self.members
            .iter()
            .map(|m| m.ty.alignment(arch) as usize)
            .max()
            .unwrap_or(1)
    }

    pub fn member_offset(&self, name: &str, arch: &Arch) -> Option<usize> {
        let mut offset = 0;
        for memb in self.members.iter() {
            let memb_type = &memb.ty;
            let memb_size = memb_type.byte_size(arch) as usize;
            let memb_align = memb_type.alignment(arch) as usize;

            let align_rest = offset % memb_align;
            if align_rest == 0 {
                if memb.name.0.data == name {
                    return Some(offset);
                }

                offset += memb_size;
            } else {
                offset = offset - align_rest + memb_align;
                if memb.name.0.data == name {
                    return Some(offset);
                }

                offset += memb_size;
            }
        }

        None
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

    #[test]
    fn entire_size_without_ptr() {
        let dummy_source = Source::new("test", "testing");

        let struct_def = StructDef {
            members: vec![
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 0..1),
                        data: "t".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 1..2),
                        data: "e".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Char),
                },
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 2..3),
                        data: "s".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
            ],
        };

        assert_eq!(12, struct_def.entire_size(&Arch::X86));
        assert_eq!(12, struct_def.entire_size(&Arch::X86_64));
        assert_eq!(12, struct_def.entire_size(&Arch::AArch64));
    }

    #[test]
    fn entire_size_with_ptr() {
        let dummy_source = Source::new("test", "testing");

        let struct_def = StructDef {
            members: vec![
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 0..1),
                        data: "t".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 1..2),
                        data: "e".to_string(),
                    }),
                    ty: AType::Pointer(Box::new(AType::Primitve(APrimitive::Char))),
                },
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 2..3),
                        data: "s".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
            ],
        };

        assert_eq!(12, struct_def.entire_size(&Arch::X86));
        assert_eq!(20, struct_def.entire_size(&Arch::X86_64));
        assert_eq!(20, struct_def.entire_size(&Arch::AArch64));
    }

    #[test]
    fn offset_without_ptr() {
        let dummy_source = Source::new("test", "testing");

        let struct_def = StructDef {
            members: vec![
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 0..1),
                        data: "t".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 1..2),
                        data: "e".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Char),
                },
                StructMember {
                    name: Identifier(SpanData {
                        span: Span::new_source(dummy_source.clone(), 2..3),
                        data: "s".to_string(),
                    }),
                    ty: AType::Primitve(APrimitive::Int),
                },
            ],
        };

        assert_eq!(Some(0), struct_def.member_offset("t", &Arch::X86));
        assert_eq!(Some(4), struct_def.member_offset("e", &Arch::X86));
        assert_eq!(Some(8), struct_def.member_offset("s", &Arch::X86));

        assert_eq!(Some(0), struct_def.member_offset("t", &Arch::X86_64));
        assert_eq!(Some(4), struct_def.member_offset("e", &Arch::X86_64));
        assert_eq!(Some(8), struct_def.member_offset("s", &Arch::X86_64));

        assert_eq!(Some(0), struct_def.member_offset("t", &Arch::AArch64));
        assert_eq!(Some(4), struct_def.member_offset("e", &Arch::AArch64));
        assert_eq!(Some(8), struct_def.member_offset("s", &Arch::AArch64));
    }
}
