use std::collections::{HashMap, HashSet};

use graphs::directed::DirectedChain;
use ir::{BasicBlock, Variable};

use super::RegisterConfig;

fn count_uses(iter: &mut dyn Iterator<Item = &ir::BasicBlock>) -> HashMap<ir::Variable, usize> {
    let mut tmp = HashMap::new();

    for stmnt in iter.flat_map(|b| b.get_statements()) {
        for var in stmnt.used_vars() {
            let entry = tmp.entry(var);
            let value = entry.or_insert(0);
            *value += 1;
        }
    }

    tmp
}

struct Pressure {
    inner: HashMap<Variable, usize>,
    max_gp: usize,
    max_fp: usize,
}

impl Pressure {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            max_gp: 0,
            max_fp: 0,
        }
    }

    pub fn define(&mut self, var: Variable, uses: usize) {
        if uses == 0 {
            return;
        }

        self.inner.insert(var, uses);

        self.max_gp = std::cmp::max(
            self.max_gp,
            self.inner.keys().filter(|v| !v.ty.is_float()).count(),
        );
        self.max_fp = std::cmp::max(
            self.max_fp,
            self.inner.keys().filter(|v| v.ty.is_float()).count(),
        );
    }
    pub fn used(&mut self, var: &Variable) {
        let count = self.inner.get_mut(var).unwrap();

        *count = count.saturating_sub(1);

        if *count == 0 {
            self.inner.remove(var);
        }
    }
}

// TODO
// This definetly needs more work on it
pub fn max_pressure<OU>(
    head: ir::BasicBlock,
    inner: DirectedChain<'_, BasicBlock>,
    outer_use: OU,
) -> RegisterConfig
where
    OU: Fn(&Variable) -> bool,
{
    let inner_defined: HashSet<_> = inner
        .duplicate()
        .flatten()
        .flat_map(|b| b.get_statements())
        .filter_map(|stmnt| match stmnt {
            ir::Statement::Assignment { target, .. } => Some(target),
            _ => None,
        })
        .collect();

    let mut total_uses = count_uses(&mut std::iter::once(&head).chain(inner.duplicate().flatten()));

    for (var, uses) in total_uses.iter_mut() {
        if outer_use(var) {
            *uses += 1;
        }
    }

    let mut pressure = Pressure::new();
    for (v, u) in total_uses
        .iter()
        .filter(|(v, _)| !inner_defined.contains(v))
    {
        pressure.define(v.clone(), *u);
    }

    for stmnt in inner.duplicate().flatten().flat_map(|b| b.get_statements()) {
        for var in stmnt.used_vars() {
            pressure.used(&var);
        }

        if let ir::Statement::Assignment { target, .. } = stmnt {
            let uses = total_uses.get(&target).unwrap();
            pressure.define(target, *uses);
        }
    }

    RegisterConfig {
        general_purpose_count: pressure.max_gp,
        floating_point_count: pressure.max_fp,
    }
}

#[cfg(test)]
mod tests {
    use general::{arch::Arch, Source, Span};
    use graphs::directed::ChainEntry;
    use ir::Program;

    use super::*;

    #[test]
    #[ignore = "test"]
    fn low_pressure() {
        let content = "
        long test() {
            long outer = 13;

            long i = 0;
            while (i < 10) {
                long other = 1;
                i = i + other;
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

        let head = match chain.next_entry().unwrap() {
            ChainEntry::Node(n) => n,
            _ => unreachable!(""),
        };

        let inner_chain = match chain.next_entry() {
            Some(ChainEntry::Cycle { inner, .. }) => inner,
            Some(_) => todo!(""),
            None => todo!(""),
        };

        let max_pressure = max_pressure(head.clone(), inner_chain, |_| false);
        dbg!(max_pressure);

        todo!("Test")
    }
}
