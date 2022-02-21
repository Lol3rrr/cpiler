use std::{collections::BTreeSet, sync::Arc};

use crate::{BasicBlock, InnerBlock};

/// This Iterator iterates the Blocks that follow the starting Block
pub struct BlockIter {
    visited: BTreeSet<*const InnerBlock>,
    left: Vec<Arc<InnerBlock>>,
}

impl BlockIter {
    pub(crate) fn new(start: Arc<InnerBlock>) -> Self {
        Self {
            visited: BTreeSet::new(),
            left: vec![start],
        }
    }
}

impl Iterator for BlockIter {
    type Item = BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let raw_current = self.left.pop()?;
        let mut current: BasicBlock = raw_current.into();

        while self.visited.contains(&current.as_ptr()) {
            let raw_next = self.left.pop()?;
            current = raw_next.into();
        }

        let current_ptr = current.as_ptr();
        self.visited.insert(current_ptr);

        let following_iter = current.successors().into_iter().map(|(_, b)| b.0);
        self.left.extend(following_iter);

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{JumpMetadata, Statement};

    use super::*;

    #[test]
    fn simple_linear_blocks() {
        let starting = BasicBlock::initial(vec![]);

        let second = BasicBlock::new(vec![starting.weak_ptr()], vec![]);
        starting.add_statement(Statement::Jump(second.clone(), JumpMetadata::Linear));

        let third = BasicBlock::new(vec![second.weak_ptr()], vec![]);
        second.add_statement(Statement::Jump(third.clone(), JumpMetadata::Linear));

        let mut targets = HashSet::new();
        targets.insert(starting.as_ptr());
        targets.insert(second.as_ptr());
        targets.insert(third.as_ptr());

        let mut iter = starting.block_iter();

        targets.remove(&iter.next().unwrap().as_ptr());
        targets.remove(&iter.next().unwrap().as_ptr());
        targets.remove(&iter.next().unwrap().as_ptr());

        assert_eq!(None, iter.next());
        assert_eq!(0, targets.len());
    }

    #[test]
    fn blocks_with_cycle() {
        let starting = BasicBlock::initial(vec![]);

        let second = BasicBlock::new(vec![starting.weak_ptr()], vec![]);
        starting.add_statement(Statement::Jump(second.clone(), JumpMetadata::Linear));

        let third = BasicBlock::new(
            vec![second.weak_ptr()],
            vec![Statement::Jump(second.clone(), JumpMetadata::Linear)],
        );
        second.add_predecessor(third.weak_ptr());
        second.add_statement(Statement::Jump(third.clone(), JumpMetadata::Linear));

        let fourth = BasicBlock::new(vec![third.weak_ptr()], vec![]);
        third.add_statement(Statement::Jump(fourth.clone(), JumpMetadata::Linear));

        let mut targets = HashSet::new();
        targets.insert(starting.as_ptr());
        targets.insert(second.as_ptr());
        targets.insert(third.as_ptr());
        targets.insert(fourth.as_ptr());

        let mut iter = starting.block_iter();

        targets.remove(&iter.next().unwrap().as_ptr());
        targets.remove(&iter.next().unwrap().as_ptr());
        targets.remove(&iter.next().unwrap().as_ptr());
        targets.remove(&iter.next().unwrap().as_ptr());

        assert_eq!(None, iter.next());
        assert_eq!(0, targets.len());
    }
}
