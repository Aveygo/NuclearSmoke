use std::f64::consts::PI;

use wseg10::WSEG10;
use contours::{find_contours, Point};

use wasm_bindgen::prelude::*;
use web_sys::console;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct WASMPoint {
    x: f64,
    y: f64
}

impl WASMPoint {
    fn new(from: &Point) -> Self {
        Self {
            x: from.x,
            y: from.y
        }
    }
}

fn sphere_coords(lat: f64, lon: f64, x_km: f64, y_km: f64) -> (f64, f64) {
    /* Haversine formula */
    const EARTH_RADIUS_KM: f64 = 6371.0;
    let lat_rad = lat.to_radians();
    let new_lat = lat + (y_km / EARTH_RADIUS_KM) * (180.0 / PI);
    let new_lon = lon + (x_km / (EARTH_RADIUS_KM * lat_rad.cos())) * (180.0 / PI);
    (new_lat, new_lon)
}

fn apply_contour(contour :Vec<Point>, rotation_deg: f64, lat:f64, lon:f64) -> Vec<Point> {
    // Convert the angle from degrees to radians
    let angle_radians = rotation_deg * PI / 180.0;

    // Rotation matrix components
    let cos_theta = angle_radians.cos();
    let sin_theta = angle_radians.sin();

    // Apply the rotation matrix to each point
    contour
        .iter()
        .map(|p| {

            let (new_x, new_y) = sphere_coords(
                lat, 
                lon, 
                p.x * cos_theta - p.y * sin_theta, 
                p.x * sin_theta + p.y * cos_theta
            );

            Point {
                x: new_x,
                y: new_y
            }
        })
        .collect()
}

#[wasm_bindgen]
pub fn smoke(lat:f64, lon:f64, mt_yield:f64, wind_km_h:f64, wind_direction_deg:f64, wind_shear_km_h_m: f64, threshold:f64) -> String {
    let w: WSEG10 = WSEG10::new(mt_yield, wind_km_h, wind_shear_km_h_m);
    let contour = find_contours(100, threshold, &|x:f64, y:f64| w.dose(x, y));
    let contour = apply_contour(contour, wind_direction_deg, lat, lon); 
    let contour: Vec<WASMPoint> = contour.iter().map(|p| WASMPoint::new(p)).collect();
    
    return serde_json::to_string(&contour).unwrap();

}
