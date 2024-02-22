use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::{ImageBuffer, RgbImage};
use wlrs::helpers::resize_image;

fn bench_resize(c: &mut Criterion) {
    let image: RgbImage = black_box(ImageBuffer::new(3840, 2160));
    c.bench_function("enlarge_smol", |b| {
        b.iter(|| resize_image(&image, 3940, 2260))
    });

    c.bench_function("shrink_smol", |b| {
        b.iter(|| resize_image(&image, 3740, 2060))
    });

    c.bench_function("enlarge_big", |b| {
        b.iter(|| resize_image(&image, 4840, 3160))
    });

    c.bench_function("shrink_big", |b| {
        b.iter(|| resize_image(&image, 1940, 1060))
    });

    c.bench_function("resize_to_current_size", |b| {
        b.iter(|| resize_image(&image, 3840, 2160))
    });
}

criterion_group!(benches, bench_resize);
criterion_main!(benches);
