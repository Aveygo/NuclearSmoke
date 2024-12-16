use wseg10::WSEG10;
use contours::find_contours;
use charming::{component::{Axis, Title}, element::AxisType, series::Scatter, Chart};
use std::iter;
use charming::{ImageRenderer, ImageFormat};

fn main() {

    let w = WSEG10::new(0.0, 0.0, 3000.0, 1.0, 50.0, 0.0, 5.0, 0.0);
    let contour = find_contours(10.0, 10, &|x:f64, y:f64| w.dose(x, y));
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
