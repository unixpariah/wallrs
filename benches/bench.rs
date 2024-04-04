use criterion::{criterion_group, criterion_main, Criterion};
use wlrs::{set_from_memory, set_from_path};

fn bench_resize(c: &mut Criterion) {
    c.bench_function("set_jpg_from_path", |b| {
        b.iter(|| {
            set_from_path("tests/test_images/test1.jpg", vec![]).unwrap();
        })
    });

    c.bench_function("set_png_from_path", |b| {
        b.iter(|| {
            set_from_path("tests/test_images/test2.png", vec![]).unwrap();
        })
    });

    c.bench_function("set_bmp_from_path", |b| {
        b.iter(|| {
            set_from_path("tests/test_images/test3.bmp", vec![]).unwrap();
        })
    });

    c.bench_function("set_jpg_from_memory", |b| {
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            set_from_memory(image.clone(), vec![]).unwrap();
        })
    });

    c.bench_function("set_png_from_memory", |b| {
        let image = image::open("tests/test_images/test2.png").unwrap();
        b.iter(|| {
            set_from_memory(image.clone(), vec![]).unwrap();
        })
    });

    c.bench_function("set_bmp_from_memory", |b| {
        let image = image::open("tests/test_images/test3.bmp").unwrap();
        b.iter(|| {
            set_from_memory(image.clone(), vec![]).unwrap();
        })
    });
}

criterion_group!(benches, bench_resize);
criterion_main!(benches);
