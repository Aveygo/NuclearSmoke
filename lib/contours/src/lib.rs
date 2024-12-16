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

fn ray(mut p1:Point, mut p2:Point, min_step:f64, iter:usize, func: &dyn Fn(&Point) -> f64) -> Point {
    /*
        The ray travels in one direction to find function contour 
     */

    let origin = Point::new(0.0, 0.0);

    for i in 0..iter {

        if p1.distance(&p2) < min_step {
            return p1
        }

        let m = (func(&p2) - func(&p1)) / (origin.distance(&p2) - origin.distance(&p1));
        


        let b = func(&p2) - m * (origin.distance(&p2));
        let q = -b/m; // q is the distance to the origin
        println!("{:?} {:?} {:?} {:?}", p1.distance(&p2), m, b, q);

        let m = p2.y / p2.x;
        let q = Point::new(
            q * m.atan().cos(),
            q * m.atan().sin(),
        );

        let p2_clone = p2.clone();
        p2 = q;
        p1 = p2_clone;
    }

    return p2;
}


pub fn find_contours(threshold: f64, n_rays:usize, func: &dyn Fn(f64, f64) -> f64) -> Vec<Point> {
    /*
        func is assumed to be a vertically symmetrical ~ellipsoid blur centered around (0, 0)
     */
    let nfunc = |point:&Point| (func(point.x, point.y)-threshold).abs();

    let mut contour: Vec<Point> = vec![];

    for degree in (-90..90).step_by(180 / n_rays) {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(
            (degree as f64 * PI / 180.0).cos(),
            (degree as f64 * PI / 180.0).sin()
        );

        println!("{:?}, {:?}, {:?}", degree, degree as f64 * PI / 180.0, p2);

        let found = ray(p1, p2, 0.5, 50, &nfunc);
        contour.push(found);
        contour.push(Point::new(
            -1.0 * found.x,
            found.y
        ));
    }

    contour

}