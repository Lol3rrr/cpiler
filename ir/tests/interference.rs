use general::{Source, Span};
use ir::{DefaultInterferenceGraph, FunctionDefinition, InterferenceGraph, NodeId, Type, Variable};

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

    let mut result_graph = DefaultInterferenceGraph::new();
    ir_func.interference_graph(&mut result_graph, |_, _, _| {});
    dbg!(&result_graph);

    let x_var = Variable::new("x_17563920617334630623", Type::I64);
    let y_var = Variable::new("y_16744721608688310825", Type::I64);
    let z_var = Variable::new("z_9845864589953920289", Type::I64);
    let w_var = Variable::new("w_2874211422146313536", Type::I64);
    let tmp_var = Variable::tmp(0, Type::I64);

    let mut expected = DefaultInterferenceGraph::new();
    expected.add_node(NodeId::new(x_var.clone()));
    expected.add_node(NodeId::new(y_var.clone()));
    expected.add_node(NodeId::new(z_var.clone()));
    expected.add_node(NodeId::new(w_var.clone()));
    expected.add_node(NodeId::new(tmp_var.clone()));

    expected.add_edge(NodeId::new(x_var.clone()), NodeId::new(y_var.clone()));
    expected.add_edge(NodeId::new(x_var.clone()), NodeId::new(z_var.clone()));
    expected.add_edge(NodeId::new(x_var.clone()), NodeId::new(w_var.clone()));

    expected.add_edge(NodeId::new(y_var.clone()), NodeId::new(z_var.clone()));

    expected.add_edge(NodeId::new(z_var.clone()), NodeId::new(w_var.clone()));
    expected.add_edge(NodeId::new(z_var.clone()), NodeId::new(tmp_var.clone()));

    expected.add_edge(NodeId::new(w_var.clone()), NodeId::new(tmp_var.clone()));

    assert_eq!(expected, result_graph);
}

#[test]
fn branched_program() {
    let content = "
long test() {
    long x = 0;
    long y = 13;
    long w;
    if (1) {
        long z = 23;
        w = z + y;
    } else {
        long z = 24;
        w = z + y;
    }

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

    let mut result_graph = DefaultInterferenceGraph::new();
    ir_func.interference_graph(&mut result_graph, |_, _, _| {});
    dbg!(&result_graph);

    let var_x = Variable::new("x_17563920617334630623", Type::I64);
    let var_y = Variable::new("y_16744721608688310825", Type::I64);
    let var_w0 = Variable::new("w_8337266112918192722", Type::I64);
    let var_w1 = var_w0.next_gen();
    let var_w2 = var_w0.next_gen();
    let var_w3 = var_w0.next_gen();
    let var_tmp0 = Variable::tmp(0, Type::I64);
    let var_z1 = Variable::new("z_6378042013892026610", Type::I64);
    let var_z0 = Variable::new("z_3684889136638212640", Type::I64);
    let var_tmp1 = Variable::tmp(1, Type::I64);

    std::fs::write("./if_graph.dot", result_graph.to_dot()).unwrap();

    let mut expected = DefaultInterferenceGraph::new();
    expected.add_node(NodeId::new(var_x.clone()));
    expected.add_node(NodeId::new(var_y.clone()));
    expected.add_node(NodeId::new(var_w0.clone()));
    expected.add_node(NodeId::new(var_w1.clone()));
    expected.add_node(NodeId::new(var_w2.clone()));
    expected.add_node(NodeId::new(var_w3.clone()));
    expected.add_node(NodeId::new(var_tmp0.clone()));
    expected.add_node(NodeId::new(var_z0.clone()));
    expected.add_node(NodeId::new(var_z1.clone()));
    expected.add_node(NodeId::new(var_tmp1.clone()));

    expected.add_edge(var_x.clone(), var_w2.clone());
    expected.add_edge(var_x.clone(), var_w0.clone());
    expected.add_edge(var_x.clone(), var_y.clone());
    expected.add_edge(var_x.clone(), var_tmp0.clone());
    expected.add_edge(var_x.clone(), var_tmp1.clone());
    expected.add_edge(var_x.clone(), var_z0.clone());
    expected.add_edge(var_x.clone(), var_z1.clone());
    expected.add_edge(var_x.clone(), var_w3.clone());
    expected.add_edge(var_x.clone(), var_w1.clone());

    expected.add_edge(var_y.clone(), var_x.clone());
    expected.add_edge(var_y.clone(), var_w2.clone());
    expected.add_edge(var_y.clone(), var_w0.clone());
    expected.add_edge(var_y.clone(), var_z0.clone());
    expected.add_edge(var_y.clone(), var_w1.clone());
    expected.add_edge(var_y.clone(), var_tmp0.clone());
    expected.add_edge(var_y.clone(), var_z1.clone());

    expected.add_edge(var_w0.clone(), var_w3.clone());
    expected.add_edge(var_w0.clone(), var_w1.clone());
    expected.add_edge(var_w0.clone(), var_tmp0.clone());
    expected.add_edge(var_w0.clone(), var_w2.clone());
    expected.add_edge(var_w0.clone(), var_x.clone());
    expected.add_edge(var_w0.clone(), var_z0.clone());
    expected.add_edge(var_w0.clone(), var_z1.clone());
    expected.add_edge(var_w0.clone(), var_y.clone());
    expected.add_edge(var_w0.clone(), var_tmp1.clone());

    expected.add_edge(var_tmp0.clone(), var_x.clone());
    expected.add_edge(var_tmp0.clone(), var_y.clone());
    expected.add_edge(var_tmp0.clone(), var_w0.clone());

    expected.add_edge(var_z0.clone(), var_w1.clone());
    expected.add_edge(var_z0.clone(), var_x.clone());
    expected.add_edge(var_z0.clone(), var_y.clone());
    expected.add_edge(var_z0.clone(), var_w0.clone());

    expected.add_edge(var_w1.clone(), var_x.clone());
    expected.add_edge(var_w1.clone(), var_w3.clone());
    expected.add_edge(var_w1.clone(), var_z0.clone());
    expected.add_edge(var_w1.clone(), var_y.clone());
    expected.add_edge(var_w1.clone(), var_w0.clone());

    expected.add_edge(var_z1.clone(), var_w2.clone());
    expected.add_edge(var_z1.clone(), var_y.clone());
    expected.add_edge(var_z1.clone(), var_w0.clone());
    expected.add_edge(var_z1.clone(), var_x.clone());

    expected.add_edge(var_w2.clone(), var_y.clone());
    expected.add_edge(var_w2.clone(), var_w0.clone());
    expected.add_edge(var_w2.clone(), var_x.clone());
    expected.add_edge(var_w2.clone(), var_w3.clone());
    expected.add_edge(var_w2.clone(), var_z1.clone());

    expected.add_edge(var_w3.clone(), var_w0.clone());
    expected.add_edge(var_w3.clone(), var_tmp1.clone());
    expected.add_edge(var_w3.clone(), var_w1.clone());
    expected.add_edge(var_w3.clone(), var_w2.clone());
    expected.add_edge(var_w3.clone(), var_x.clone());

    expected.add_edge(var_tmp1.clone(), var_x.clone());
    expected.add_edge(var_tmp1.clone(), var_w0.clone());
    expected.add_edge(var_tmp1.clone(), var_w3.clone());

    std::fs::write("./if_e_graph.dot", expected.to_dot()).unwrap();

    assert_eq!(expected, result_graph);
}

#[test]
#[ignore = ""]
fn looped_program() {
    let content = "
long test() {
    long x = 0;
    long y;
    while (13) {
        y = 10;
    }

    return x + y;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let semantic_ast = semantic::parse(syntax_ast).unwrap();
    let ir = semantic_ast.convert_to_ir(general::arch::Arch::X86_64);
    let mut opt_config = optimizer::Config::new();
    opt_config.add_pass(optimizer::optimizations::Merger::new());
    let ir = optimizer::optimize(ir, opt_config);

    let ir_func: &FunctionDefinition = ir.functions.get("test").unwrap();

    dbg!(&ir_func);

    let mut result_graph = DefaultInterferenceGraph::new();
    ir_func.interference_graph(&mut result_graph, |_, _, _| {});
    dbg!(&result_graph);

    std::fs::write("./if_graph.dot", result_graph.to_dot()).unwrap();

    let var_x0 = Variable::new("x_17563920617334630623", Type::I64);
    let var_x1 = var_x0.next_gen();
    let var_y0 = Variable::new("y_16744721608688310825", Type::I64);
    let var_y1 = var_y0.next_gen();
    let var_y2 = var_y1.next_gen();
    let var_tmp0 = Variable::tmp(0, Type::I64);
    let var_tmp1 = Variable::tmp(1, Type::I64);

    let mut expected = DefaultInterferenceGraph::new();

    expected.add_node(var_x0.clone());
    expected.add_node(var_x1.clone());
    expected.add_node(var_y0.clone());
    expected.add_node(var_y1.clone());
    expected.add_node(var_y2.clone());
    expected.add_node(var_tmp0.clone());
    expected.add_node(var_tmp1.clone());

    expected.add_edge(var_y0.clone(), var_x0.clone());

    expected.add_edge(var_tmp0.clone(), var_y0.clone());
    expected.add_edge(var_tmp0.clone(), var_x0.clone());

    expected.add_edge(var_x1.clone(), var_y0.clone());
    expected.add_edge(var_x1.clone(), var_x0.clone());

    expected.add_edge(var_y2.clone(), var_x1.clone());
    expected.add_edge(var_y2.clone(), var_y0.clone());

    expected.add_edge(var_y1.clone(), var_x1.clone());
    expected.add_edge(var_y1.clone(), var_y2.clone());

    expected.add_edge(var_tmp1.clone(), var_x1.clone());
    expected.add_edge(var_tmp1.clone(), var_y2.clone());
    expected.add_edge(var_tmp1.clone(), var_y1.clone());

    assert_eq!(expected, result_graph);
}
