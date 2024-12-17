use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64
}

impl Point {
    fn new(x:f64, y:f64) -> Self {
        Self { x, y }
    }
}

fn ray(mut p1:Point, dx:f64, dy:f64, min_error: f64, iter:usize, func: &dyn Fn(&Point) -> f64) -> Point {
    let mut step_size = 0.1;
    let mut state = false; // false -> doubling, true -> halving

    for i in 0..iter {

        if func(&p1).abs() < min_error {
            return p1
        };

        if func(&p1) <= 0.0 {
            if step_size > 0.0 { step_size *= -1.0; }
            if !state { state = true; }
        } else {
            if step_size < 0.0 { step_size *= -1.0; }
        }


        step_size *= if state {0.5} else {2.0};
        p1.x += dx*step_size; p1.y += dy*step_size;

    }

    return p1;
}


pub fn find_contours(threshold: f64, func: &dyn Fn(f64, f64) -> f64) -> Vec<Point> {
    /*
        func is assumed to be a vertically symmetrical ~ellipsoid blur centered around (0, 0)
     */

    
    let nfunc = |point:&Point| func(point.x, point.y) - threshold;

    
    let mut contour: Vec<Point> = vec![];

    for degree in (0..361).step_by(360 / 100) {
        
        let dx = (degree as f64 * PI / 180.0).cos();
        let dy = (degree as f64 * PI / 180.0).sin();

        println!("{:?} {:?}", dx, dy);

        let found = ray(Point::new(0.0, 0.0), dx, dy, 1.0, 100, &nfunc);
        contour.push(found);
    }

    contour

}