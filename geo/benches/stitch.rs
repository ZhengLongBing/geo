// 这需要对stitchtriangles trait进行公有导出。目前我们决定将其设为私有
// 因此此基准测试被注释掉。如果你需要它且trait仍未公有，
// 你需要临时更改它以使此基准测试再次工作。
//
// use criterion::{criterion_group, criterion_main, criterion};
// use geo::stitch::StitchTriangles;
// use geo::TriangulateSpade;
//
// fn criterion_benchmark(c: &mut criterion) {
//     c.bench_function("stitch", |bencher| {
//         let p = geo_test_fixtures::east_baton_rouge::<f32>();
//         let tris = p.unconstrained_triangulation().unwrap();
//
//         bencher.iter(|| {
//             criterion::black_box(criterion::black_box(&tris).stitch_triangulation().unwrap());
//         });
//     });
// }
//
// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
fn main() {
    println!("Placeholder");
}
