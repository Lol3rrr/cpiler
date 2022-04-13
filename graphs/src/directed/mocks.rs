use super::GraphNode;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MockNode {
    pub id: usize,
    pub successors: Vec<usize>,
}

impl GraphNode for MockNode {
    type Id = usize;
    type SuccessorIterator = std::vec::IntoIter<usize>;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn successors(&self) -> Self::SuccessorIterator {
        self.successors.clone().into_iter()
    }
}
