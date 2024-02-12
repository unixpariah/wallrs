use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::{ImageBuffer, RgbaImage};
use wlrs::helpers::resize_image;

fn bench_resize(c: &mut Criterion) {
    let image: RgbaImage = black_box(ImageBuffer::new(3840, 2160));
    c.bench_function("enlarge_smol", |b| {
        b.iter(|| resize_image(image.clone(), 3940, 2260))
    });

    c.bench_function("shrink_smol", |b| {
        b.iter(|| resize_image(image.clone(), 3740, 2060))
    });

    c.bench_function("enlarge_big", |b| {
        b.iter(|| resize_image(image.clone(), 4840, 3160))
    });

    c.bench_function("shrink_big", |b| {
        b.iter(|| resize_image(image.clone(), 1940, 1060))
    });
}

criterion_group!(benches, bench_resize);
criterion_main!(benches);
