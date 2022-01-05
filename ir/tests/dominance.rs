use general::{Source, Span};
use ir::{DominanceTree, FunctionDefinition, Type, Variable};

#[test]
fn linear_program() {
    let content = "
long test() {
    long x = 0;
    long y = 13;
    long z = x + y;
    long w = x + 2;

    return z + w;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let semantic_ast = semantic::parse(syntax_ast).unwrap();
    let ir = semantic_ast.convert_to_ir(general::arch::Arch::X86_64);

    let ir_func: &FunctionDefinition = ir.functions.get("test").unwrap();

    let result_tree = ir_func.dominance_tree();
    dbg!(&result_tree);

    let var_x = Variable::new("x_17563920617334630623", Type::I64);
    let var_y = Variable::new("y_16744721608688310825", Type::I64);
    let var_z = Variable::new("z_9845864589953920289", Type::I64);
    let var_w = Variable::new("w_2874211422146313536", Type::I64);
    let var_tmp0 = Variable::tmp(0, Type::I64);

    let mut expected_tree = DominanceTree::new();
    expected_tree.append(var_x);
    expected_tree.append(var_y);
    expected_tree.append(var_z);
    expected_tree.append(var_w);
    expected_tree.append(var_tmp0);

    assert_eq!(expected_tree, result_tree);
}

#[test]
fn branched_program() {
    let content = "
long test() {
    long x = 0;
    long y = 13;
    if (13) {
        long z = x + y;
    } else {
        long z = y + 2;
    }
    long w = 13;

    return x + w;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let semantic_ast = semantic::parse(syntax_ast).unwrap();
    let ir = semantic_ast.convert_to_ir(general::arch::Arch::X86_64);

    let ir_func: &FunctionDefinition = ir.functions.get("test").unwrap();

    let result_tree = ir_func.dominance_tree();
    dbg!(&result_tree);

    let var_x = Variable::new("x_17563920617334630623", Type::I64);
    let var_y = Variable::new("y_16744721608688310825", Type::I64);
    let var_tmp0 = Variable::tmp(0, Type::I64);
    let var_z0 = Variable::new("z_1072484349275958054", Type::I64);
    let var_z1 = Variable::new("z_18125714544917642090", Type::I64);
    let var_w = Variable::new("w_2596968069347863476", Type::I64);
    let var_tmp1 = Variable::tmp(1, Type::I64);

    let mut expected_tree = DominanceTree::new();
    expected_tree.append(var_x);
    expected_tree.append(var_y);
    expected_tree.append(var_tmp0);
    expected_tree.append(var_z0);
    expected_tree.insert_at_level(var_z1);
    expected_tree.insert_at_level(var_w);
    expected_tree.append(var_tmp1);

    assert_eq!(expected_tree, result_tree);
}
