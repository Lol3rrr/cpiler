use std::collections::HashSet;

use general::{arch::Arch, Source, Span};
use graphs::directed::ChainEntry;
use ir::Program;

#[test]
fn loop_with_break() {
    let content = "
  void test() {
    int i = 0;
    while(1) {
      if (i == 0) {
        break;
      }
    }
  }
  ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let ast = syntax::parse(tokens).unwrap();
    let aast = semantic::parse(ast).unwrap();
    let ir_program: Program = aast.convert_to_ir(Arch::AArch64);

    let ir_func = ir_program.functions.get("test").unwrap();

    let func_graph = ir_func.to_directed_graph();
    let dot_str = func_graph.to_graphviz();
    std::fs::write("./directed-break.dot", dot_str).unwrap();

    let mut chain = func_graph.chain_iter();
    {
        let raw_entry = chain.next_entry();
        assert!(raw_entry.is_some());
        assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(_)));
    }
    {
        let raw_entry = chain.next_entry();
        assert!(raw_entry.is_some());
        assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(_)));
    }
    {
        let raw_entry = chain.next_entry();
        assert!(raw_entry.is_some());
        assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(_)));
    }
    {
        let raw_entry = chain.next_entry();
        assert!(raw_entry.is_some());
        let mut inner = match raw_entry.unwrap() {
            ChainEntry::Cycle { inner, .. } => inner,
            _ => panic!(),
        };

        {
            let raw_entry = inner.next_entry();
            assert!(raw_entry.is_some());
            assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(_)));
        }
        {
            let raw_entry = inner.next_entry();
            assert!(raw_entry.is_some());

            let (mut left, right) = match raw_entry.unwrap() {
                ChainEntry::Branched { sides } => sides,
                _ => panic!(),
            };

            assert!(right.is_none());

            {
                let raw_entry = left.next_entry();
                assert!(raw_entry.is_some());
            }
            {
                let raw_entry = left.next_entry();
                // assert!(raw_entry.is_none());
                dbg!(&raw_entry);
            }
            {
                let raw_entry = left.next_entry();
                dbg!(&raw_entry);
            }
        }
        {
            let raw_entry = inner.next_entry();
            assert!(raw_entry.is_some());
            assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(_)));
        }
        {
            let raw_entry = inner.next_entry();
            assert!(raw_entry.is_none());
        }
    }
    {
        let raw_entry = chain.next_entry();
        assert!(raw_entry.is_some());
        assert!(matches!(raw_entry.unwrap(), ChainEntry::Node(_)));
    }

    let flat_chain = func_graph.chain_iter().flatten();
    {
        let mut tmp = HashSet::new();
        for b in flat_chain {
            dbg!(b.as_ptr());
            assert!(tmp.insert(b.as_ptr()));
        }
    }
}

#[test]
fn nested_loops_with_break() {
    let content = "
  void test() {
    for (int i = 0; i < 5; i++) {
        for (int j = 0; j < 5; j++) {
            if (i == j) {
                break;
            }
        }
    }
  }
  ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let ast = syntax::parse(tokens).unwrap();
    let aast = semantic::parse(ast).unwrap();
    let ir_program: Program = aast.convert_to_ir(Arch::AArch64);

    let ir_func = ir_program.functions.get("test").unwrap();

    let func_graph = ir_func.to_directed_graph();

    {
        let flat_chain = func_graph.chain_iter().flatten();
        let mut tmp = HashSet::new();
        for b in flat_chain {
            dbg!(b.as_ptr());
            assert!(tmp.insert(b.as_ptr()));
        }
    }
}
