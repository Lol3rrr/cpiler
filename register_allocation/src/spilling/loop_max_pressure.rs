use std::collections::HashSet;

use graphs::directed::DirectedChain;
use ir::{BasicBlock, Statement, Variable};

use super::RegisterConfig;

fn used_vars<I>(iter: I) -> HashSet<Variable>
where
    I: Iterator<Item = Statement>,
{
    iter.flat_map(|stmnt| stmnt.used_vars()).collect()
}

// TODO
// This definetly needs more work on it
pub fn max_pressure(inner: DirectedChain<'_, BasicBlock>) -> RegisterConfig {
    let inner_used_vars = used_vars(inner.duplicate().flatten().flat_map(|b| b.get_statements()));
    dbg!(inner_used_vars);

    panic!("I dont know")
}

#[cfg(test)]
mod tests {
    use general::{arch::Arch, Source, Span};
    use graphs::directed::ChainEntry;
    use ir::Program;

    use super::*;

    #[test]
    fn low_pressure() {
        let content = "
        long test() {
            long outer = 13;

            long i = 0;
            while (i < 10) {
                i++;
            }

            long other = outer + i;
            return other;
        }
        ";

        let source = Source::new("test", content);
        let span: Span = source.into();
        let tokens = tokenizer::tokenize(span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir: Program = aast.convert_to_ir(Arch::X86_64);

        let test_func_ir = ir.functions.get("test").unwrap();
        dbg!(&test_func_ir);

        let graph = test_func_ir.to_directed_graph();
        let mut chain = graph.chain_iter();

        assert!(chain.next_entry().unwrap().is_node());
        assert!(chain.next_entry().unwrap().is_node());
        assert!(chain.next_entry().unwrap().is_node());

        let inner_chain = match chain.next_entry() {
            Some(ChainEntry::Cycle { inner, .. }) => inner,
            Some(_) => todo!(""),
            None => todo!(""),
        };

        let max_pressure = max_pressure(inner_chain);
        dbg!(max_pressure);

        todo!("Test")
    }
}
