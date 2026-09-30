use std::collections::HashMap;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use lele_code_viewer::index::{CodeBlock, ItemEdge, ItemGraph, ItemNode};
use lele_code_viewer::render::adjacency_json;

fn fixture(nodes: usize) -> ItemGraph {
    let mut graph = ItemGraph {
        nodes: Vec::new(),
        edges: Vec::new(),
        externals: Vec::new(),
        groups: Vec::new(),
    };
    for i in 0..nodes {
        graph.nodes.push(ItemNode {
            id: format!("m::n{i}"),
            name: format!("n{i}"),
            kind: CodeBlock::Function,
            external: Vec::new(),
            layer: 0,
        });
    }
    for i in 0..nodes {
        let next = i.wrapping_add(1);
        if next < nodes {
            graph.edges.push(ItemEdge { from: i, to: next });
        }
        let skip = i.wrapping_add(2);
        if skip < nodes {
            graph.edges.push(ItemEdge { from: i, to: skip });
        }
    }
    graph
}

fn lookup_old(graph: &ItemGraph, target: &str) -> usize {
    let mut count: usize = 0;
    for edge in &graph.edges {
        let from_hit = graph
            .nodes
            .get(edge.from)
            .is_some_and(|node| node.id == target);
        let to_hit = graph
            .nodes
            .get(edge.to)
            .is_some_and(|node| node.id == target);
        if from_hit || to_hit {
            count = count.wrapping_add(1);
        }
    }
    count
}

fn build_index(graph: &ItemGraph) -> HashMap<&str, Vec<usize>> {
    let mut map: HashMap<&str, Vec<usize>> = HashMap::new();
    for (n, edge) in graph.edges.iter().enumerate() {
        if let Some(node) = graph.nodes.get(edge.from) {
            map.entry(node.id.as_str()).or_default().push(n);
        }
        if let Some(node) = graph.nodes.get(edge.to) {
            map.entry(node.id.as_str()).or_default().push(n);
        }
    }
    map
}

fn bench_adjacency(c: &mut Criterion) {
    let mut group = c.benchmark_group("adjacency_json");
    for nodes in [300usize, 1000, 3000] {
        let graph = fixture(nodes);
        group.throughput(Throughput::Elements(
            u64::try_from(graph.edges.len()).unwrap_or(0),
        ));
        group.bench_with_input(BenchmarkId::from_parameter(nodes), &graph, |b, graph| {
            b.iter(|| adjacency_json(black_box(graph)));
        });
    }
    group.finish();
}

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hover_lookup");
    for nodes in [300usize, 1000, 3000] {
        let graph = fixture(nodes);
        let mid = graph.nodes.len().wrapping_div(2);
        let target: &str = graph
            .nodes
            .get(mid)
            .map_or("m::missing", |node| node.id.as_str());
        let index = build_index(&graph);
        group.throughput(Throughput::Elements(
            u64::try_from(graph.edges.len()).unwrap_or(0),
        ));
        group.bench_with_input(BenchmarkId::new("linear", nodes), &graph, |b, graph| {
            b.iter(|| lookup_old(black_box(graph), black_box(target)));
        });
        group.bench_with_input(BenchmarkId::new("indexed", nodes), &index, |b, index| {
            b.iter(|| black_box(index.get(black_box(target))));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_adjacency, bench_lookup);
criterion_main!(benches);
