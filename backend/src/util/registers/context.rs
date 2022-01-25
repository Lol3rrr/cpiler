use std::collections::HashSet;

pub mod conditional_spill;
pub mod linear_spill;
pub mod loop_spill;

pub enum SpillResult {
    Linear(linear_spill::SpillResult),
    Conditional(conditional_spill::SpillResult),
    Loop(loop_spill::SpillResult),
}

impl SpillResult {
    pub fn replacement(&self) -> ir::Variable {
        match self {
            Self::Linear(res) => res.var.next_gen(),
            Self::Conditional(cond) => match cond {
                conditional_spill::SpillResult::OuterVariable { var } => var.next_gen(),
                conditional_spill::SpillResult::InnerVariable { var } => var.next_gen(),
            },
            Self::Loop(cond) => match cond {
                loop_spill::SpillResult::Outer { var } => var.next_gen(),
            },
        }
    }
}

#[derive(Debug)]
pub enum SpillContext {
    /// We are in a linear Control-Flow Section, meaning there are no branches or loops, just
    /// straiht Control-Flow from Top to Bottom
    Linear {
        /// The starting Block
        start: ir::BasicBlock,
        /// The last Block that is Part of this Linear Section
        end: ir::BasicBlock,
    },
    /// We are in one Side of a Conditional Block
    Conditional {
        /// The Common "Root" Block, from which the Control-Flow splits into the current Block
        header: ir::BasicBlock,
        end: ir::BasicBlock,
        current_start: ir::BasicBlock,
        /// The last block in the current Conditional Section
        current_end: ir::BasicBlock,
        other_start: Option<ir::BasicBlock>,
    },
    Loop {
        header: ir::BasicBlock,
        first_inner: ir::BasicBlock,
        first_after: ir::BasicBlock,
    },
}

impl SpillContext {
    pub fn determine(start: ir::BasicBlock) -> Self {
        let mut block = start.clone();

        loop {
            let succs = block.successors();
            if succs.len() == 0 {
                // If there are no more successors, we have reached the end of a Function and since
                // we found nothing else, need to be in a Linear Code Block
                return Self::Linear { start, end: block };
            } else if succs.len() == 1 {
                // If there is just one successor we change nothing regarding our Context and just
                // continue onto that Block and do nothing else
                let (_, tmp) = succs.into_iter().next().unwrap();
                block = tmp;
            } else if succs.len() == 2 {
                // If there are two sucessors, that means the current Block is the header of a Loop
                // or Conditional, in either case we want to skip to the end of that Control-Flow
                // part and continue with the first common Block afterwards

                let mut tmp: Vec<_> = succs.into_iter().map(|(_, b)| b).collect();
                let left = tmp.pop().unwrap();
                let right = tmp.pop().unwrap();

                let common = left.earliest_common_block(&right).unwrap();

                block = common;
                continue;
            }

            let preds = block.get_predecessors();
            if preds.len() == 1 {
                // If we have just one Predecessor we can still be in any Type of Context and so we
                // change nothing and just go back to the top of the Loop
                continue;
            } else if preds.len() == 2 {
                // If this Block has two Predecessors, we just reached a Block were Control-Flow
                // merges from two different Paths, this could either mean we just found the Common
                // End block of a Conditional or we reached the Head of a Loop.
                //
                // We now need to try and differentiate between these two Cases and to do that we
                // will

                let current_ptr = block.as_ptr();

                let mut header = block.clone();

                let mut visited = HashSet::new();
                let pred_iter = block.predecessor_iter().skip(1);

                for pred in pred_iter {
                    let pred_ptr = pred.as_ptr();
                    visited.insert(pred_ptr);

                    if pred_ptr == current_ptr {
                        let mut h_succs: Vec<_> =
                            header.successors().into_iter().map(|(_, b)| b).collect();

                        let raw_first = h_succs.remove(0);
                        let raw_second = h_succs.remove(0);

                        let (inner, outer) = if visited.contains(&raw_first.as_ptr()) {
                            (raw_first, raw_second)
                        } else {
                            (raw_second, raw_first)
                        };

                        return Self::Loop {
                            header,
                            first_inner: inner,
                            first_after: outer,
                        };
                    }

                    if pred.successors().len() == 2 {
                        header = pred;
                        continue;
                    }
                }

                let mut succs: Vec<_> = header.successors().into_iter().map(|(_, b)| b).collect();

                let first = succs.remove(0);
                let second = succs.remove(0);

                let (current_start, other_start) = if visited.contains(&first.as_ptr()) {
                    let other = if second.as_ptr() != block.as_ptr() {
                        Some(second)
                    } else {
                        None
                    };

                    (first, other)
                } else {
                    let other = if first.as_ptr() != block.as_ptr() {
                        Some(first)
                    } else {
                        None
                    };

                    (second, other)
                };

                let mut end_preds: Vec<_> = block
                    .get_predecessors()
                    .into_iter()
                    .map(|p| p.upgrade().unwrap())
                    .collect();

                let first_preds: HashSet<_> = end_preds
                    .get(0)
                    .unwrap()
                    .predecessor_iter()
                    .map(|b| b.as_ptr())
                    .collect();

                let current_end = if first_preds.contains(&current_start.as_ptr()) {
                    end_preds.remove(0)
                } else {
                    end_preds.remove(1)
                };

                return Self::Conditional {
                    header,
                    end: block,
                    current_start,
                    current_end,
                    other_start,
                };
            }
        }
    }

    pub fn determine_spill_var(
        &self,
        largest_vars: HashSet<ir::Variable>,
        spill_block: ir::BasicBlock,
        start_index: usize,
    ) -> SpillResult {
        match self {
            Self::Linear { start, end } => {
                let res = linear_spill::determine_spill_var(largest_vars, spill_block, start_index);
                SpillResult::Linear(res)
            }
            Self::Conditional {
                header,
                end,
                other_start,
                current_start,
                current_end,
            } => {
                let res = conditional_spill::spill_var(
                    &largest_vars,
                    spill_block,
                    start_index,
                    header.clone(),
                    end.clone(),
                    current_start.clone(),
                    other_start.clone(),
                );
                SpillResult::Conditional(res)
            }
            Self::Loop {
                header,
                first_inner,
                first_after,
            } => {
                dbg!(header.as_ptr(), first_inner.as_ptr(), first_after.as_ptr());
                let res = loop_spill::spill_var(
                    largest_vars,
                    spill_block,
                    start_index,
                    header.clone(),
                    first_inner.clone(),
                    first_after.clone(),
                );

                todo!("Spill in Loop")
            }
        }
    }
}
