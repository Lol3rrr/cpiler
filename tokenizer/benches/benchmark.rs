use criterion::{black_box, criterion_group, criterion_main, Criterion};
use general::{Source, Span};

pub fn declarations(c: &mut Criterion) {
    let content = "
int test;
void other;
short testing;
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();

    c.bench_function("declarations", |b| {
        b.iter(|| tokenizer::tokenize(black_box(span.clone())))
    });
}

criterion_group!(benches, declarations);
criterion_main!(benches);
