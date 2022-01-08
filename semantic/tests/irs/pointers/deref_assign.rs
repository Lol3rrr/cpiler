use general::{arch::Arch, Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, Program, Statement, Type, Value,
    Variable, VariableMetadata,
};

#[test]
fn random_ptr_deref() {
    let content = "
void test() {
    int* x = (int*) 123;
    
    *x = 13;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let x_var = Variable::new("x_17563920617334630623", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::Pointer);

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::Pointer(Box::new(Type::I32)),
                    base: Operand::Constant(Constant::I64(123)),
                }),
            },
            Statement::SaveVariable { var: x_var.clone() },
            Statement::WriteMemory {
                target: Operand::Variable(x_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}

#[test]
fn var_ptr_deref() {
    let content = "
void test() {
    int y = 0;
    int* x = &y;
    
    *x = 13;

    return;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global_block = BasicBlock::initial(vec![]);

    let func_initial = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let y0_var = Variable::new("y_8304944181907028059", Type::I32);
    let y1_var = y0_var.next_gen();
    let x_var = Variable::new("x_5934430639642140251", Type::Pointer(Box::new(Type::I32)))
        .set_meta(VariableMetadata::VarPointer {
            var: Box::new(y0_var.name.clone()),
        });

    let func_inner = BasicBlock::new(
        vec![func_initial.weak_ptr()],
        vec![
            Statement::Assignment {
                target: y0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::SaveVariable {
                var: y0_var.clone(),
            },
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Expression(Expression::AdressOf {
                    base: Operand::Variable(y0_var.clone()),
                }),
            },
            Statement::SaveVariable { var: x_var.clone() },
            Statement::WriteMemory {
                target: Operand::Variable(x_var.clone()),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(13)),
                }),
            },
            Statement::Assignment {
                target: y1_var.clone(),
                value: Value::Unknown,
            },
            Statement::Return(None),
        ],
    );

    func_initial.add_statement(Statement::Jump(func_inner));

    let expected = Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
