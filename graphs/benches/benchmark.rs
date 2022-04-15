use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use graphs::directed::{DirectedGraph, GraphNode};

struct BenchNode {
    id: usize,
    succs: Vec<usize>,
}

impl GraphNode for BenchNode {
    type Id = usize;
    type SuccessorIterator = std::vec::IntoIter<usize>;

    fn id(&self) -> Self::Id {
        self.id
    }
    fn successors(&self) -> Self::SuccessorIterator {
        self.succs.clone().into_iter()
    }
}

fn linear_graph(graph: &mut DirectedGraph<BenchNode>, start_id: usize, count: usize) -> usize {
    assert!(count > 0);

    for id in (0..count).map(|i| i + start_id) {
        let succ_id = id + 1;
        assert!(succ_id < start_id + count + 1);
        graph.add_node(BenchNode {
            id,
            succs: vec![succ_id],
        });
    }

    start_id + count
}

pub fn linear_code(c: &mut Criterion) {
    let mut graph = DirectedGraph::new();

    let count = 1000;
    {
        let next_id = linear_graph(&mut graph, 0, count);
        graph.add_node(BenchNode {
            id: next_id,
            succs: vec![],
        });
    }

    let mut c_group = c.benchmark_group("graph-chain-linear");

    c_group.throughput(Throughput::Elements(count as u64 + 1));

    c_group.bench_function("iter", |b| {
        b.iter(|| {
            let mut chain = graph.chain_iter();

            while let Some(e) = chain.next_entry() {
                let _ = e;
            }

            assert!(chain.next_entry().is_none());
        });
    });
}

pub fn branched(c: &mut Criterion) {
    let mut graph = DirectedGraph::new();

    {
        let inner_branch_length = 100;

        graph.add_node(BenchNode {
            id: 0,
            succs: vec![1],
        });
        graph.add_node(BenchNode {
            id: 1,
            succs: vec![2, inner_branch_length + 2],
        });

        {
            for id in 2..(inner_branch_length + 1) {
                graph.add_node(BenchNode {
                    id,
                    succs: vec![id + 1],
                });
            }
            graph.add_node(BenchNode {
                id: inner_branch_length + 1,
                succs: vec![(inner_branch_length * 2) + 2],
            });
        }
        {
            for id in (inner_branch_length + 2)..((inner_branch_length * 2) + 1) {
                graph.add_node(BenchNode {
                    id,
                    succs: vec![id + 1],
                });
            }
            graph.add_node(BenchNode {
                id: (inner_branch_length * 2) + 1,
                succs: vec![(inner_branch_length * 2) + 2],
            });
        }

        graph.add_node(BenchNode {
            id: (inner_branch_length * 2) + 2,
            succs: vec![],
        });
    }

    let mut c_group = c.benchmark_group("graph-chain-flat");

    c_group.bench_function("branched", |b| {
        b.iter(|| {
            let mut chain = graph.chain_iter().flatten();

            while let Some(e) = chain.next_entry() {
                let _ = e;
            }

            assert!(chain.next_entry().is_none());
        });
    });
}

pub fn nested_branch(c: &mut Criterion) {
    let mut graph = DirectedGraph::new();

    {
        graph.add_node(BenchNode {
            id: 0,
            succs: vec![1, 2],
        });
        graph.add_node(BenchNode {
            id: 3,
            succs: vec![],
        });
        graph.add_node(BenchNode {
            id: 1,
            succs: vec![3],
        });
        graph.add_node(BenchNode {
            id: 2,
            succs: vec![3],
        });
    }

    let mut c_group = c.benchmark_group("graph-chain-flat");

    c_group.bench_function("nested-branched", |b| {
        b.iter(|| {
            let mut chain = graph.chain_iter().flatten();

            while let Some(e) = chain.next_entry() {
                let _ = e;
            }

            assert!(chain.next_entry().is_none());
        });
    });
}

pub fn cycled(c: &mut Criterion) {
    let mut graph = DirectedGraph::new();

    {
        let inner_count = 100;

        graph.add_node(BenchNode {
            id: 0,
            succs: vec![1, 2],
        });
        graph.add_node(BenchNode {
            id: 1,
            succs: vec![inner_count + 3],
        });

        for id in 2..(2 + inner_count) {
            graph.add_node(BenchNode {
                id,
                succs: vec![id + 1],
            });
        }
        graph.add_node(BenchNode {
            id: inner_count + 2,
            succs: vec![0],
        });

        let last_id = linear_graph(&mut graph, inner_count + 3, 1000);
        graph.add_node(BenchNode {
            id: last_id,
            succs: vec![],
        });
    }

    let mut c_group = c.benchmark_group("graph-chain-flat");

    c_group.bench_function("cycled", |b| {
        b.iter(|| {
            let mut chain = graph.chain_iter().flatten();

            while let Some(e) = chain.next_entry() {
                let _ = e;
            }

            assert!(chain.next_entry().is_none());
        });
    });
}

criterion_group!(benches, linear_code, branched, nested_branch, cycled);
criterion_main!(benches);
