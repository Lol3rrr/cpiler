use general::{Source, Span};
use ir::{BasicBlock, FunctionDefinition, Statement, Value, Variable};

#[test]
fn read_global() {
    let content = "
int tmp;

void test() {
    int x = tmp;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let tmp_var = Variable::new("tmp_631366842563799064", ir::Type::I32);
    let x_var = Variable::new("x_14387504199494487623", ir::Type::I32);

    let global_block = BasicBlock::initial(vec![Statement::Assignment {
        target: tmp_var.clone(),
        value: Value::Unknown,
    }]);

    let func_start = BasicBlock::new(vec![global_block.weak_ptr()], vec![]);

    let func_inner = BasicBlock::new(
        vec![func_start.weak_ptr()],
        vec![Statement::Assignment {
            target: x_var.clone(),
            value: Value::Variable(tmp_var.clone()),
        }],
    );
    func_start.add_statement(Statement::Jump(func_inner.clone()));

    let expected = ir::Program {
        global: global_block,
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: ir::Type::Void,
                block: func_start,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir(general::arch::Arch::X86_64);
    dbg!(&result);

    assert_eq!(expected, result);
}
