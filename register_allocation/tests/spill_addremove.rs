#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum TestRegister {
    GeneralPurpose(usize),
    FloatingPoint(usize),
}

impl register_allocation::Register for TestRegister {
    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (8, 8),
            Self::FloatingPoint(_) => (8, 8),
        }
    }

    fn reg_type(&self) -> register_allocation::RegisterType {
        match self {
            Self::GeneralPurpose(_) => register_allocation::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => register_allocation::RegisterType::FloatingPoint,
        }
    }
}

#[test]
fn addremove() {
    let test_program = include_str!("addremove.c");
    let test_source = general::Source::new("test", test_program);
    let test_span: general::Span = test_source.into();
    let tokens = tokenizer::tokenize(test_span);
    let ast = syntax::parse(tokens).unwrap();
    let aast = semantic::parse(ast).unwrap();
    let ir: ir::Program = aast.convert_to_ir(general::arch::Arch::X86_64);

    let func = ir.functions.get("addRemoveCourse").unwrap();

    let registers = [
        TestRegister::GeneralPurpose(0),
        TestRegister::GeneralPurpose(1),
        TestRegister::GeneralPurpose(2),
        TestRegister::GeneralPurpose(3),
        TestRegister::FloatingPoint(0),
        TestRegister::FloatingPoint(1),
        TestRegister::FloatingPoint(2),
        TestRegister::FloatingPoint(3),
    ];
    let mapping = register_allocation::RegisterMapping::allocate(
        func,
        &registers,
        register_allocation::AllocationCtx { build_path: None },
    );

    todo!()
}
