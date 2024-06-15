use criterion::{criterion_group, criterion_main, Criterion};
use wlrs::{set_from_memory, set_from_path, CropMode};

fn benchmark_crop(c: &mut Criterion) {
    c.bench_function("crop_set_jpg_from_path", |b| {
        b.iter(|| {
            set_from_path("tests/test_images/test1.jpg", vec![], CropMode::Crop).unwrap();
        })
    });
    c.bench_function("crop_set_jpg_from_memory", |b| {
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            set_from_memory(image.clone(), vec![], CropMode::Crop).unwrap();
        })
    });
}

fn benchmark_fit(c: &mut Criterion) {
    c.bench_function("fit_set_jpg_from_path", |b| {
        b.iter(|| {
            set_from_path("tests/test_images/test1.jpg", vec![], CropMode::Fit(None)).unwrap();
        })
    });

    c.bench_function("fit_set_jpg_from_memory", |b| {
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            set_from_memory(image.clone(), vec![], CropMode::Fit(None)).unwrap();
        })
    });
}

fn benchmark_no(c: &mut Criterion) {
    c.bench_function("no_set_jpg_from_path", |b| {
        b.iter(|| {
            set_from_path("tests/test_images/test1.jpg", vec![], CropMode::No(None)).unwrap();
        })
    });

    c.bench_function("no_set_jpg_from_memory", |b| {
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            set_from_memory(image.clone(), vec![], CropMode::No(None)).unwrap();
        })
    });
}

criterion_group!(crop, benchmark_crop);
criterion_group!(fit, benchmark_fit);
criterion_group!(no, benchmark_no);
criterion_main!(crop, no, fit);
