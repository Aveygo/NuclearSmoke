use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64
}

fn ray(center:Point, direction:f64, tolerance: f64, iter:usize, func: &dyn Fn(&Point) -> f64) -> Option<Point> {
    /*
        Casts a ray from p1 into direction (dx, dy) to estimate where func(x, y) = 0
        Ray travels in discrete steps, initially doubling until it crosses the boundary, where it
        then goes back halving it's steps to refine the estimation.

        Only returns one point where the ray lands if within tolerance.
        If the ray shoots off / takes too many iterations to converge, then it returns None.

        Can be optimized by somehow matching the step size to the gradient of ray position? Newtons method?
     */
    
    let mut step_size    = 0.1;              // Initial step size of the ray
    let mut state       = false;            // false -> doubling, true -> halving
    let mut p1         = center.clone();   // Initial ray position

    // Convert the direction into meaningful changes in x/y for the ray
    let dx = (direction * PI / 180.0).cos();
    let dy = (direction * PI / 180.0).sin();

    for _i in 0..iter {

        if func(&p1).abs() < tolerance { return Some(p1) };     // Close enough to boundary, early exit

        if func(&p1) <= 0.0 {
            if step_size > 0.0 { step_size *= -1.0; }           // Reverse direction if stepped outside of boundary
            if !state { state = true; }                         // We passed the boundary, start the halving approach
        } else {
            if step_size < 0.0 { step_size *= -1.0; }           // Reverse direction if stepped inside of boundary
        }

        step_size *= if state {0.5} else {2.0};                 // Double step if finding boundary, half if refining 
        p1.x += dx*step_size; p1.y += dy*step_size;             // Progress ray in direction (dx, dy)

    }

    return None;
}


pub fn find_contours(num_points: usize, threshold: f64, func: &dyn Fn(f64, f64) -> f64) -> Vec<Point> {
    /*
        Attempts to find points where func(point) = threshold by shooting rays until they hit the boundary
        We assume that Func is an ellipsoid-ish continuous field, with func(0, 0) > threshold
     */

    
    let nfunc = |point:&Point| func(point.x, point.y) - threshold;

    let mut contours: Vec<Point> = vec![];
    let tol = 1.0;
    let iter = 32; // Arbitrary, larger than earth given one unit of x/y ~= 1km

    // Because ~highest should be at center, if lower, quit early to save compute
    let center = Point{x: 0.0, y: 0.0};
    if nfunc(&center) < threshold {
        return contours;
    }

    // Cast rays outwards evenly in a circular pattern
    for degree in (0..361).step_by(360 / num_points) {     
        
        // Ray may not hit boundary if bad center pos or func != threshold
        match ray(center, degree.into(), tol, iter, &nfunc) {
            Some(found) => {
                contours.push(found);
            }
            None => {}
        };

    }

    contours

}