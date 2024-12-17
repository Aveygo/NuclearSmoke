#[derive(Debug, Clone, Copy, PartialEq)]
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

}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ComputedPoint {
    point: Point,
    value: f64
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub a: Point,
    pub b: Point
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComputedEdge {
    pub a: ComputedPoint,
    pub b: ComputedPoint
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point,
    pub max: Point
}

fn interp(edge: ComputedEdge) -> Point {
    if (edge.b.value - edge.a.value).abs() == 0.0 {
        return edge.a.point;
    }

    let t = (0.0 - edge.a.value) / (edge.b.value - edge.a.value);

    Point {
        x: edge.a.point.x + t * (edge.b.point.x - edge.a.point.x),
        y: edge.a.point.y + t * (edge.b.point.y - edge.a.point.y),
    }
}



fn marching_squares(func: &dyn Fn(f64, f64) -> f64, bounds: Bounds, resolution: usize, threshold: f64) -> Vec<Point> {
    let mut result: Vec<Point> = vec![];
    let mut map: Vec<ComputedPoint> = vec![];

    for y in (bounds.min.y as usize..=bounds.max.y as usize).step_by(resolution) {
        for x in (bounds.min.x as usize..=bounds.max.x as usize).step_by(resolution) {
            let value = func(x as f64, y as f64);
            map.push(ComputedPoint{
                point: Point {
                    x: x as f64,
                    y: y as f64
                },
                value: value
            });
        }
    }

    let width = (bounds.max.x as usize - bounds.min.x as usize) / resolution + 1;
    let height = (bounds.max.y as usize - bounds.min.y as usize) / resolution + 1;

    let mut contour:Vec<Edge> = vec![];

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let i = y * width + x;
            let I = map[i];
            let PT = I.value >= threshold;

            let N = map[i - width];                     
            let E = map[i + 1];                 
            let S = map[i - width];         
            let W = map[i - 1];             

            
            let NT = N.value >= threshold;
            let ET = E.value >= threshold;
            let ST = S.value >= threshold;
            let WT = W.value >= threshold;
            
            if (NT != PT && ET != PT) {
                contour.push(Edge{a: interp(ComputedEdge{a: I, b: N}), b: interp(ComputedEdge{a: I, b: E})});
            }

            if (ET != PT && ST != PT) {
                contour.push(Edge{a: interp(ComputedEdge{a: I, b: E}), b: interp(ComputedEdge{a: I, b: S})});
            }

            if (ST != PT && WT != PT) {
                contour.push(Edge{a: interp(ComputedEdge{a: I, b: S}), b: interp(ComputedEdge{a: I, b: W})});
            }

            if (WT != PT && NT != PT) {
                contour.push(Edge{a: interp(ComputedEdge{a: I, b: W}), b: interp(ComputedEdge{a: I, b: N})});
            }
        }
    }

    for c in contour {
        result.push(c.a);
    }

    result
}



pub fn find_contours(threshold: f64, func: &dyn Fn(f64, f64) -> f64) -> Vec<Point> {
    /*
        func is assumed to be a vertically symmetrical ~ellipsoid blur centered around (0, 0)
     */

    let bounds = Bounds{
        min: Point { x: -1000.0, y: -1000.0 },
        max: Point { x: 1000.0, y: 1000.0 },
    };
    
    return marching_squares(func, bounds, 1, threshold);
}