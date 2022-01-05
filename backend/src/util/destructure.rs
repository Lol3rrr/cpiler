use ir::{Statement, Value};

pub fn destructure_func(func: &ir::FunctionDefinition) {
    for c_block in func.block.block_iter() {
        let statements = c_block.get_statements();
        let phis: Vec<_> = statements
            .iter()
            .filter_map(|s| match s {
                Statement::Assignment {
                    target,
                    value: Value::Phi { sources },
                } => Some((target, sources)),
                _ => None,
            })
            .collect();

        for (p_target, p_sources) in phis {
            for source in p_sources {
                let s_block = match source.block.upgrade() {
                    Some(b) => b,
                    None => {
                        panic!("Block does not exist anymore");
                        continue;
                    }
                };

                let mut s_stmnts = s_block.get_statements();

                let (assign_index, _) = s_stmnts
                    .iter()
                    .enumerate()
                    .find(|(_, s)| match s {
                        Statement::Assignment { target, .. } if target == &source.var => true,
                        _ => false,
                    })
                    .unwrap();

                s_stmnts.insert(
                    assign_index + 1,
                    Statement::Assignment {
                        target: p_target.clone(),
                        value: Value::Variable(source.var.clone()),
                    },
                );

                s_block.set_statements(s_stmnts);
            }
        }

        c_block.set_statements(
            statements
                .into_iter()
                .filter(|s| {
                    !matches!(
                        s,
                        Statement::Assignment {
                            value: Value::Phi { .. },
                            ..
                        }
                    )
                })
                .collect(),
        );
    }
}
