use std::num::NonZeroU32;

use criterion::{criterion_group, criterion_main, Criterion};
use wlrs::{CropMode, Image, SetType, Wlrs};

fn benchmark_crop(c: &mut Criterion) {
    let wlrs = Wlrs::new().unwrap();
    c.bench_function("crop_set_jpg_from_path", |b| {
        b.iter(|| {
            wlrs.set(
                SetType::Path("tests/test_images/test1.jpg"),
                &[],
                CropMode::Crop,
            )
            .unwrap();
        })
    });
    c.bench_function("crop_set_jpg_from_memory", |b| {
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            let image_data = Image::new(
                &image.to_rgba8(),
                NonZeroU32::new(image.width()).unwrap(),
                NonZeroU32::new(image.height()).unwrap(),
            )
            .unwrap();
            wlrs.set(SetType::Img(image_data), &[], CropMode::Crop)
                .unwrap();
        })
    });
}

fn benchmark_fit(c: &mut Criterion) {
    let wlrs = Wlrs::new().unwrap();
    c.bench_function("fit_set_jpg_from_path", |b| {
        b.iter(|| {
            wlrs.set(
                SetType::Path("tests/test_images/test1.jpg"),
                &[],
                CropMode::Fit(None),
            )
            .unwrap();
        })
    });

    c.bench_function("fit_set_jpg_from_memory", |b| {
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            let image_data = Image::new(
                &image.to_rgba8(),
                NonZeroU32::new(image.width()).unwrap(),
                NonZeroU32::new(image.height()).unwrap(),
            )
            .unwrap();
            wlrs.set(SetType::Img(image_data), &[], CropMode::Fit(None))
                .unwrap();
        })
    });
}

fn benchmark_no(c: &mut Criterion) {
    let wlrs = Wlrs::new().unwrap();
    c.bench_function("no_set_jpg_from_path", |b| {
        b.iter(|| {
            wlrs.set(
                SetType::Path("tests/test_images/test1.jpg"),
                &[],
                CropMode::No(None),
            )
            .unwrap();
        })
    });

    c.bench_function("no_set_jpg_from_memory", |b| {
        let wlrs = Wlrs::new().unwrap();
        let image = image::open("tests/test_images/test1.jpg").unwrap();
        b.iter(|| {
            let image_data = Image::new(
                &image.to_rgba8(),
                NonZeroU32::new(image.width()).unwrap(),
                NonZeroU32::new(image.height()).unwrap(),
            )
            .unwrap();
            wlrs.set(SetType::Img(image_data), &[], CropMode::No(None))
                .unwrap();
        })
    });
}

criterion_group!(crop, benchmark_crop);
criterion_group!(fit, benchmark_fit);
criterion_group!(no, benchmark_no);
criterion_main!(crop, no, fit);
