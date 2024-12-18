 fn index_of_min(v: &[f64]) -> Option<usize> {
    v.iter()
        .enumerate()
        .fold(None, |min, (i, &val)| match min {
            Some((min_idx, min_val)) if min_val <= val => Some((min_idx, min_val)),
            _ => Some((i, val)),
        })
        .map(|(min_idx, _)| min_idx)
}

fn main() {
    // https://prod.dataportal.rfs.nsw.gov.au/majorIncidents.json
    // https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&hourly=temperature_2m,wind_speed_10m,wind_speed_120m,wind_direction_10m,wind_direction_120m&forecast_days=1

    // grads_size = 129, grads_step= 3hr
    // 

    let url = "";






}