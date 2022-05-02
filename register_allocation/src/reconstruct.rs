use std::{
    collections::{btree_set, HashMap},
    panic,
    rc::Weak,
};

use graphs::directed::{ChainEntry, DirectedChain};
use ir::{
    BasicBlock, Expression, FunctionDefinition, Operand, PhiEntry, Statement, Value, Variable,
    VariableGroup, WeakBlockPtr,
};

use crate::spilling::ReloadList;

mod lastdef;
use lastdef::{LastDef, LastDefinition};

/// Recconstructs the Programs SSA Form
pub fn reconstruct<R>(func: &FunctionDefinition, was_reloaded: R, reloads: ReloadList)
where
    R: Fn(&Variable) -> bool,
{
    let graph = func.to_directed_graph();
    let root_chain = graph.chain_iter();

    let mut defs = LastDefinition::new();

    reconstruct_chain(root_chain, &was_reloaded, &mut defs, &reloads);
}

fn reconstruct_chain<R>(
    chain: DirectedChain<'_, BasicBlock>,
    was_reloaded: &R,
    last_def: &mut LastDefinition,
    reloads: &ReloadList,
) where
    R: Fn(&Variable) -> bool,
{
    let mut last = None;
    let mut peekable = chain.peekable();
    while let Some(entry) = peekable.next() {
        match entry {
            ChainEntry::Node(node) => {
                let current_reloads = match reloads.get_block(node.as_ptr()) {
                    Some((_, r)) => r,
                    None => {
                        todo!()
                    }
                };

                let mut c_reloads = {
                    let mut tmp = current_reloads.clone();
                    tmp.sort_by(|a, b| b.position.cmp(&a.position));
                    tmp
                };

                let mut statements = node.get_statements();

                // dbg!(node.as_ptr(), &c_reloads);

                let mut index = 0;
                while let Some(stmnt) = statements.get_mut(index) {
                    while false {
                        match c_reloads.pop() {
                            Some(r) if r.position == index => {
                                let n_var = r.var;
                                last_def.defined_single(n_var, node.weak_ptr());
                            }
                            Some(r) if r.position < index => {
                                panic!()
                            }
                            Some(r) => {
                                c_reloads.push(r);
                                break;
                            }
                            None => break,
                        };
                    }

                    replace(stmnt, last_def);

                    match stmnt {
                        Statement::Assignment { target, .. } => {
                            let group: VariableGroup = target.clone().into();

                            last_def.defined_single(target.clone(), node.weak_ptr());
                        }
                        _ => {}
                    };

                    index += 1;
                }

                node.set_statements(statements);

                last = Some(node);
            }
            ChainEntry::Branched {
                sides: (left, right_opt),
            } => {
                let mut left_defs = last_def.clone();
                reconstruct_chain(left.duplicate(), was_reloaded, &mut left_defs, reloads);

                let mut right_defs = match right_opt.clone() {
                    Some(right) => {
                        let mut defs = last_def.clone();
                        reconstruct_chain(right.duplicate(), was_reloaded, &mut defs, reloads);
                        defs
                    }
                    None => last_def.clone(),
                };

                // Filters out all the Groups that are only in one of the Sides, this should only
                // include temporary/local variables that "will" never escape their scope
                // and therefore dont need to be considered for the rest
                left_defs.retain(|k, v| right_defs.contains(k.clone()));
                right_defs.retain(|k, v| left_defs.contains(k.clone()));

                let junction = left_defs.intersection(&right_defs);

                // If the two halves are now the same (ignoring local stuff) then we can just set last_def to their
                // junction and can move forward
                if left_defs
                    .groups()
                    .filter(|k| right_defs.contains((*k).clone()))
                    .count()
                    == junction.len()
                    && right_defs
                        .groups()
                        .filter(|k| left_defs.contains((*k).clone()))
                        .count()
                        == junction.len()
                {
                    debug_assert_eq!(left_defs, right_defs);
                    *last_def = junction;
                    continue;
                }

                // Here we know that Variables that will probably live after the Sides are different between the
                // two sides and we now need to figure out how we can join them up now

                let different_defs = left_defs.joined_iter(&right_defs);

                let raw_peeked = peekable
                    .peek_mut()
                    .expect("There should be a block after a Branch");
                let peeked = match raw_peeked {
                    ChainEntry::Node(n) => n,
                    other => {
                        todo!()
                    }
                };

                let mut statements = peeked.get_statements();

                let first_non_phi = statements
                    .iter()
                    .enumerate()
                    .find(|(_, s)| match s {
                        Statement::Assignment {
                            value: Value::Phi { .. },
                            ..
                        } => false,
                        _ => true,
                    })
                    .map(|(i, _)| i)
                    .unwrap();

                let left_last_block = match left.last().unwrap() {
                    ChainEntry::Node(n) => n,
                    _ => unreachable!(),
                };
                let right_last_block = match right_opt {
                    Some(right) => match right.last().unwrap() {
                        ChainEntry::Node(n) => n,
                        _ => unreachable!(),
                    },
                    None => last.unwrap(),
                };

                let mut n_defs = junction;

                let mut next_phi = first_non_phi;
                for (group, (left_var, right_var)) in different_defs.take(0) {
                    let left_var = match left_var {
                        LastDef::Single(s, _) => s,
                        LastDef::Two(_, _) => panic!(),
                    };
                    let right_var = match right_var {
                        LastDef::Single(s, _) => s,
                        LastDef::Two(_, _) => panic!(),
                    };

                    let insert_pos = {
                        let find_res = statements.iter().enumerate().find(|(i, s)| match s {
                            Statement::Assignment {
                                target,
                                value: Value::Phi { .. },
                            } if group.contains(target) => true,
                            _ => false,
                        });
                        if let Some(index) = find_res.map(|(i, _)| i) {
                            Ok(index)
                        } else {
                            let tmp = next_phi;
                            next_phi += 1;
                            Err(tmp)
                        }
                    };

                    let n_sources = vec![
                        PhiEntry {
                            block: left_last_block.weak_ptr(),
                            var: left_var.clone(),
                        },
                        PhiEntry {
                            block: right_last_block.weak_ptr(),
                            var: right_var.clone(),
                        },
                    ];

                    match insert_pos {
                        Ok(update_pos) => {
                            let entry = statements.get_mut(update_pos).unwrap();
                            let var = match entry {
                                Statement::Assignment {
                                    target,
                                    value: Value::Phi { sources },
                                } => {
                                    *sources = n_sources;

                                    target.clone()
                                }
                                _ => unreachable!(),
                            };

                            println!("Updated {:?}", var);

                            n_defs.defined_single(var, peeked.weak_ptr());
                        }
                        Err(insert_pos) => {
                            let new_var = left_var.next_gen();
                            let statement = Statement::Assignment {
                                target: new_var.clone(),
                                value: Value::Phi { sources: n_sources },
                            };

                            println!("Inserted {:?}", new_var);

                            statements.insert(insert_pos, statement);
                            n_defs.defined_single(new_var, peeked.weak_ptr());
                        }
                    };
                }
                peeked.set_statements(statements);

                *last_def = n_defs;
            }
            ChainEntry::Cycle { inner, .. } => {
                let mut inner_def = last_def.clone();
                reconstruct_chain(inner.clone(), was_reloaded, &mut inner_def, reloads);

                let intersection = last_def.intersection(&inner_def);

                let diff_iter = last_def
                    .joined_iter(&inner_def)
                    .filter(|(g, _)| !intersection.contains(VariableGroup::clone(g)));

                let header_statements = last.as_ref().expect("").get_statements();

                for (group, (l_var, inner_var)) in diff_iter {
                    // dbg!(group, l_var, inner_var);

                    let group_assignments = header_statements.iter().filter(|stmnt| matches!(stmnt, Statement::Assignment{target, ..} if group.contains(&target)));

                    assert!(group_assignments.clone().count() > 0);

                    for assignment in group_assignments {
                        // dbg!(assignment);
                    }
                }
            }
        };
    }
}

fn replace(stmnt: &mut Statement, defs: &LastDefinition) {
    match stmnt {
        Statement::Assignment { value, .. } => {
            replace_value(value, defs);
        }
        // TODO
        Statement::SaveVariable { var } => {}
        // TODO
        Statement::SaveGlobalVariable { name, value } => {
            let n_var = match defs.get_last(&value) {
                Some(LastDef::Single(d, _)) => d,
                _ => panic!(),
            };
            *value = n_var.clone();
        }
        Statement::WriteMemory { target, value } => {
            replace_oper(target, defs);
            replace_oper(value, defs);
        }
        Statement::Call { arguments, .. } => {
            for arg in arguments.iter_mut() {
                replace_oper(arg, defs);
            }
        }
        Statement::Jump(_, _) => {}
        Statement::JumpTrue(var, _, _) => {
            let group: VariableGroup = var.clone().into();
            let def = match defs.get_last(var) {
                Some(LastDef::Single(s, _)) => s,
                Some(LastDef::Two(_, _)) => {
                    todo!()
                }
                None => unreachable!(),
            };
            *var = def.clone();
        }
        Statement::Return(Some(var)) => {
            let group: VariableGroup = var.clone().into();
            let def = match defs.get_last(var) {
                Some(LastDef::Single(s, _)) => s,
                Some(LastDef::Two(_, _)) => {
                    todo!()
                }
                None => unreachable!(),
            };
            *var = def.clone();
        }
        Statement::Return(None) => {}
        other => {
            dbg!(other);
            todo!()
        }
    };
}

fn replace_value(value: &mut Value, defs: &LastDefinition) {
    match value {
        Value::Expression(exp) => {
            replace_exp(exp, defs);
        }
        Value::Unknown => {}
        Value::Variable(var) => {
            let def = match defs.get_last(var) {
                Some(LastDef::Single(s, _)) => s,
                Some(LastDef::Two(_, _)) => {
                    todo!()
                }
                None => unreachable!(),
            };
            *var = def.clone();
        }
        Value::Constant(_) => {}
        // TODO
        Value::Phi { sources } => {
            for source in sources.iter_mut() {
                match defs.get_last(&source.var) {
                    Some(LastDef::Single(s, b)) => {
                        source.var = s.clone();
                        source.block = b.clone();
                    }
                    Some(LastDef::Two(_, _)) => {
                        todo!("Two")
                    }
                    None => {
                        println!("Unknown Phi-Source: {:?}", source);
                    }
                };
            }

            let first = sources.first().unwrap();
            if sources.iter().all(|entry| entry.var == first.var) {
                *value = Value::Variable(first.var.clone());
            }
        }
        other => {
            dbg!(other);
            todo!()
        }
    };
}

fn replace_oper(oper: &mut Operand, defs: &LastDefinition) {
    match oper {
        Operand::Constant(_) => {}
        Operand::Variable(var) => {
            let def = match defs.get_last(&var) {
                Some(LastDef::Single(d, _)) => d,
                Some(LastDef::Two(_, _)) => {
                    todo!()
                }
                None => {
                    dbg!(var);
                    todo!()
                }
            };
            *var = def.clone();
        }
    };
}

fn replace_exp(exp: &mut Expression, defs: &LastDefinition) {
    match exp {
        Expression::UnaryOp { base, .. } => {
            replace_oper(base, defs);
        }
        Expression::BinaryOp { left, right, .. } => {
            replace_oper(left, defs);
            replace_oper(right, defs);
        }
        Expression::Cast { base, .. } => {
            replace_oper(base, defs);
        }
        Expression::FunctionCall { arguments, .. } => {
            for arg in arguments.iter_mut() {
                replace_oper(arg, defs);
            }
        }
        Expression::StackAlloc { .. } => {}
        Expression::ReadGlobalVariable { .. } => {}
        Expression::ReadMemory { address, .. } => {
            replace_oper(address, defs);
        }
        Expression::AdressOf { base } => {
            replace_oper(base, defs);
        }
        other => {
            dbg!(other);
            todo!()
        }
    };
}
