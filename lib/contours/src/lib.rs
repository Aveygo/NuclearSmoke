use std::f64::consts::PI;


#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64
}

impl Point {
    fn new(x:f64, y:f64) -> Self {
        Self {
            x, y
        }
    }

    fn distance(&self, p: &Point) -> f64 {
        return (  (self.x - p.x).powf(2.0) + (self.y - p.y).powf(2.0)  ).sqrt()
    }


}

fn ray(mut p1:Point, dx:f64, dy:f64, threshold: f64, iter:usize, func: &dyn Fn(&Point) -> f64) -> Point {
    /*
        The ray travels in one direction to find function contour 
     */

    let origin = Point::new(0.0, 0.0);

    for i in 0..iter {

        if (func(&p1) - threshold).abs() < threshold {
            return p1;
        }

        let p2 = Point::new(p1.x+dx, p1.x+dy);
        let d = (func(&p1) - func(&p2)) / (dx.powf(2.0) + dy.powf(2.0)).sqrt();
        let q = origin.distance(&p1) - (func(&p1) / d);

        println!("{:?}, {:?}", q, d);
        

        let m = dy / dx;
        let q = Point::new(
            q * m.atan().cos(),
            q * m.atan().sin(),
        );

        p1 = q;

    }

    return p1;
}


pub fn find_contours(threshold: f64, func: &dyn Fn(f64, f64) -> f64) -> Vec<Point> {
    /*
        func is assumed to be a vertically symmetrical ~ellipsoid blur centered around (0, 0)
     */

    
    let nfunc = |point:&Point| (func(point.x, point.y)-threshold).abs();

    let mut contour: Vec<Point> = vec![];

    for degree in (-90..90).step_by(180 / 100) {
        
        let dx = (degree as f64 * PI / 180.0).cos();
        let dy = (degree as f64 * PI / 180.0).sin();

        let found = ray(Point::new(0.0, 0.0), dx, dy, 1.0, 500, &nfunc);
        contour.push(found);
    }

    contour

}