use clap::Parser;
use wseg10::WSEG10;
use contours::{find_contours, Point};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    #[arg(long)]
    fire_mt: f64,

    #[arg(long)]
    wind_speed: f64,

    #[arg(long)]
    wind_shear: f64,
    
    #[arg(long)]
    threshold: f64,

}

fn main() {
    let args = Args::parse();

    let w: WSEG10 = WSEG10::new(args.fire_mt, args.wind_speed, args.wind_shear);
    let contour = find_contours(100, args.threshold, &|x:f64, y:f64| w.dose(x, y));

    let (mut max_x, mut max_y) = (0.0, 0.0);

    for point in contour {
        if point.x > max_x { max_x = point.x;}

        if point.y > max_y { max_y = point.y }
    }

    println!("{:.2},{:.2}", max_x, max_y);
}