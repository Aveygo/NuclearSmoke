use wseg10::WSEG10;
use contours::find_contours;
use charming::{component::{Axis, Title}, element::AxisType, series::Scatter, Chart};
use std::iter;
use charming::{ImageRenderer, ImageFormat};
use image::{ImageBuffer, Luma};

fn save_grayscale_image(data: Vec<Vec<f64>>, width: u32, height: u32, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Flatten the Vec<Vec<f32>> into Vec<u8>
    let flat_data: Vec<u8> = data
        .into_iter()
        .flat_map(|row| {
            row.into_iter()
                .map(|value| {
                    // Scale f32 value from [0.0, 1.0] to [0, 255]
                    let clamped = value.clamp(0.0, 1.0); // Ensure value is in [0.0, 1.0]
                    (clamped * 255.0).round() as u8
                })
        })
        .collect();

    // Create an ImageBuffer
    let img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_vec(width, height, flat_data)
        .ok_or("Failed to create ImageBuffer. Ensure width * height matches data length.")?;

    // Save the image as PNG
    img.save(filename)?;

    Ok(())
}

fn main() {

    let w = WSEG10::new(0.0, 0.0, 0.01, 1.0, 3.0, 90.0, 0.2, 0.0);

    let mut image = vec![];
    for x in -1000..1000 {
        let mut row = vec![];
        for y in -1000..1000 {
            row.push(w.dose(x as f64 / 10.0, y as f64 / 10.0))
        }
        image.push(row);
    }

    save_grayscale_image(image, 2000, 2000, "plot.png");


    let contour = find_contours(10.0, &|x:f64, y:f64| w.dose(x, y));
    println!("{:?}", contour);
    println!("Hello, world!");

    let chart = Chart::new()
        .x_axis(Axis::new().type_(AxisType::Value))
        .y_axis(Axis::new().type_(AxisType::Value))
        .series(
            Scatter::new().data(
                contour.into_iter().map(|point| vec![point.x, point.y])
                    .collect(),
            ),
        );

    let mut renderer = ImageRenderer::new(1000, 800);
    renderer.save_format(ImageFormat::Png, &chart, "chart.png");

}
