use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lib::{wavefront_alignment::wavefront_align, alignment_lib::Penalties};

fn wavefront_bench_l100_e1(c: &mut Criterion) {
    let query = "ACTCTATTTTACTCAGTGCAGGGTGAGCCGCCTATGCGGAGTGCAGTTACATAGGGAAAGCGGGGCTCAATTGCTACTCGTATGGGGTGTCACAGACGC";
    let text = "ACTCTATTTTACTCAGTGCAGGGTGAGCCGCCTATGCGGAGTGCAGTTACATAGGGTAAAGCGGGGCTCAATTGCTACTCGTATGGGGTGTCACAGACGC";
    let pens = Penalties {
        mismatch_pen: 1,
        open_pen: 2,
        extd_pen: 2,
    };

    c.bench_function("wfa length 100 1% error", |b| b.iter(|| wavefront_align(black_box(query), black_box(text), black_box(&pens))));
}

fn wavefront_bench_l100_e10(c: &mut Criterion) {
    let query = "ACTCTATTTTACTCAGTGCAGGGTGAGCCGCCTATGCGGAGTGCAGTTACATAGGGAAAGCGGGGCTCAATTGCTACTCGTATGGGGTGTCACAGACGC";
    let text = "ACTCTATTTTACTCAGTGCAGGGTGAGCCGCCTATGCGGAGTGCAGTTACATAGGGTAAAGCGGGGCTCAATTGCTACTCGTATGGGGTGTCACAGACGC";
    let pens = Penalties {
        mismatch_pen: 1,
        open_pen: 2,
        extd_pen: 2,
    };

    c.bench_function("wfa length 100 1% error", |b| b.iter(|| wavefront_align(black_box(query), black_box(text), black_box(&pens))));
}

fn wavefront_bench_l100_e30(c: &mut Criterion) {
}

fn wavefront_bench_l1000_e1(c: &mut Criterion) {
}

fn wavefront_bench_l1000_e10(c: &mut Criterion) {
}

fn wavefront_bench_l1000_e30(c: &mut Criterion) {
}

fn wavefront_bench_l10000_e1(c: &mut Criterion) {
}

fn wavefront_bench_l10000_e10(c: &mut Criterion) {
}

fn wavefront_bench_l10000_e30(c: &mut Criterion) {
}

criterion_group!(benches,
                 wavefront_bench_l100_e1,
                 wavefront_bench_l100_e10,
                 wavefront_bench_l100_e30,
                 wavefront_bench_l1000_e1,
                 wavefront_bench_l1000_e10,
                 wavefront_bench_l1000_e30,
                 wavefront_bench_l10000_e1,
                 wavefront_bench_l10000_e10,
                 wavefront_bench_l10000_e30);
criterion_main!(benches);
