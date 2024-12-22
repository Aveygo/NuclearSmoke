/*

  If you are looking at this, then you probably want to know how to calculate the smoke contours yourself.
  Unfortunately, there is no easy to use API here, and is instead all calculated using WASM with the "smoke" function.

  Please see the repo for more details: https://github.com/Aveygo/NuclearSmoke

*/

import init, { smoke } from "./pkg/nuclearsmoke.js";

async function fetch_json(url) {
  /*
    Light wrapper for boilerplate stuff TODO errors
  */
  let report = await fetch(url)
  return await report.json()
}

async function cached_fetch_json(url, ttl) {
  /*
    Mainly for dev purposes to limit number of weather requests
  */

  if (ttl == null) {
    ttl = 60 * 15 * 1000;
  }
  let data = localStorage.getItem(url);

  // Check if already in cache
  if (data) { 
    data = JSON.parse(data);
    
    // Check if too old
    if (Date.now() < data.fetched + ttl) {
      console.log("Returning cache")
      return data.data
    }
  }

  // Set cache and return results
  console.log("Refreshing/building cache")
  data = {data: await fetch_json(url), fetched: Date.now()}
  localStorage.setItem(url, JSON.stringify(data));
  return data.data;
}

async function cached_contours() {
  /* 
    Store the calculated contours into local storage
    This is mainly because fetching the weather data takes a while 
    to process for multiple fires.

    Cache can only store strings, hence the json parsing
  */

  let ttl = 60 * 15 * 1000;
  let contours = localStorage.getItem("data");

  // Check if already in cache
  if (contours) { 
    contours = JSON.parse(contours);
    
    // Check if too old
    if (Date.now() < contours.fetched + ttl) {
      console.log("Returning cache")
      return contours.data
    }
  }

  console.log("Refreshing/building cache")

  // Show loading screen
  let loading = document.getElementById("loading");
  loading.classList.remove("hidden")

  // Create all the contours
  contours = {data: await build_contours(), fetched: Date.now()}
  
  // Hide the loading screen
  loading.classList.add("hidden")

  // Set cache and return results
  localStorage.setItem("data", JSON.stringify(contours));
  return contours.data;
}

function bounding_box(points) {
  let min_lat = points[0][0];
  let min_lon = points[0][1];
  let max_lat = points[0][0];
  let max_lon = points[0][1];

  for (let point of points) {
    if (point[0] < min_lat) min_lat = point[0];
    if (point[0] > max_lat) max_lat = point[0];
    if (point[1] < min_lon) min_lon = point[1];
    if (point[1] > max_lon) max_lon = point[1];
  }

  return [min_lat, min_lon, max_lat, max_lon]
}

function bounding_box_size(points) {
  let bounds = bounding_box(points);
  let min_lat = bounds[1];
  let min_lon = bounds[0];
  let max_lat = bounds[3];
  let max_lon = bounds[2];

  // Approximation of distances in meters for lat/lon differences
  const earthRadius = 6371000; // in meters
  const latDist = ((max_lat - min_lat) * Math.PI / 180) * earthRadius;
  const avgLat = (min_lat + max_lat) / 2; // Average latitude for accurate longitude scaling
  const lonDist = ((max_lon - min_lon) * Math.PI / 180) * earthRadius * Math.cos(avgLat * Math.PI / 180);

  return Math.abs((latDist * lonDist) / 10000); // hectares
}

function get_size(fire) {
  // Description is mainly only within NSW data, but is the best outcome
  // cause they set it to 0 when the fire is out 
  if (fire.properties.description != null) {
    const match = fire.properties.description.match(/SIZE:\s*(\d+)\s*ha/);
    if (match) { return parseInt(match[1], 10); }
  }

  
  // Sum of bounding boxes, kinda inaccurate as bush fires don't
  // burn in squares, but it's easy to calculate
  let sizes = [];
  if (fire.geometry != null) {
    if (fire.geometry.type === "Polygon") {
      sizes.push(bounding_box_size(fire.geometry.coordinates[0]));
      
    }
  }
  let size = sizes.reduce((a, b) => a + b, 0);

  if (size != null) {return size}
  return 0
}

function get_position(fire) {

  // Could be given exact point immediately
  if (fire.geometry.geometries != null) {
    let maybe_point = fire.geometry.geometries.at(-1)
    if (maybe_point.type == "Point") {
      return maybe_point.coordinates
    }
  }

  // Estimate based on first given bounds
  
  if (fire.geometry.type == "Polygon") {
    let bounds = bounding_box(fire.geometry.coordinates[0]);

    let min_lat = bounds[0];
    let min_lon = bounds[1];
    let max_lat = bounds[2];
    let max_lon = bounds[3];
    
    return [(max_lat + min_lat)/2, (max_lon + min_lon)/2]
  }
  
  return null
}

function get_age(fire) {
  let date = null;

  if (fire.properties.pubDate) { // NSW
    const [day, month, year, time] = fire.properties.pubDate.split(/[\/\s]/);
    date = new Date(`${year}-${month}-${day}T${time}`);
  }

  if (fire.properties.updated) { // VIC
    date = new Date(fire.properties.updated);
  } 

  if (date != null) {
    return Math.floor((new Date() - date) / 1000);
  }

  return null
}

async function handle_fire(fire) {
  /*
    Asynchronously fetch weather data & calculate contours 
  */
  
  let hectares = get_size(fire);
  let center = get_position(fire);
  let age = get_age(fire);

  if ((center!=null) && hectares > 0 && (age != null) && (age < 60 * 60 * 24 * 1)) {

    // Assumed as center of fire
    let lat = center[1]
    let lon = center[0]

    let weather = await cached_fetch_json("https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lon + "&hourly=temperature_2m,wind_speed_10m,wind_speed_120m,wind_direction_10m,wind_direction_120m&forecast_days=1", 6 * 60 * 60 * 1000)

    // Get current hour as weather returns hourly data
    const d = new Date();
    let hour = d.getUTCHours();

    // Weather data
    let wind = (weather.hourly.wind_speed_10m[hour] + weather.hourly.wind_speed_120m[hour]) / 2
    let wind_direction = (weather.hourly.wind_direction_10m[hour] + weather.hourly.wind_direction_120m[hour]) / 2
    let shear = Math.abs(weather.hourly.wind_speed_10m[hour] - weather.hourly.wind_speed_120m[hour]) / 110
    let temp = weather.hourly.temperature_2m[hour]

    // Contour data
    let result_10   = JSON.parse(smoke(lat, lon, hectares/10000*temp, wind/5, wind_direction, shear*10, 10));
    let result_100  = JSON.parse(smoke(lat, lon, hectares/10000*temp, wind/5, wind_direction, shear*10, 1000));
    let result_1000 = JSON.parse(smoke(lat, lon, hectares/10000*temp, wind/5, wind_direction, shear*10, 100000));
    
    // Return found data
    return {"fire": fire, "contours": [result_10, result_100, result_1000]}
  }
}

async function build_contours() {
  
  // Fetch NSW fires
  let fires = (await cached_fetch_json("https://prod.dataportal.rfs.nsw.gov.au/majorIncidents.json", 15 * 60 * 1000)).features;

  // Fetch VIV fires
  fires.push(...(await cached_fetch_json("https://www.emergency.vic.gov.au/public/impact-areas-geojson.json", 15 * 60 * 1000)).features);

  // Async each fire to spread out weather requests
  let jobs = [];
  for (let i=0; i<fires.length;i++) {
    jobs.push(handle_fire(fires[i]));
  }

  // Wait for all the jobs to be completed
  let contours = [];
  await Promise.all(jobs).then((values) => {
    for (let i=0; i<values.length; i++) {    
      if (values[i]) { contours.push(values[i]) }
    }
  });

  // Return found contours for each fire
  return contours
} 

async function main() {
  
  // Init wasm object
  await init(); 

  // Set view onto Sydney
  var map = L.map('map').setView([ -33.8521, 151.1917], 10); 

  // Initialize Leaflet 
  L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a> <span>|</span> Data from <a href="https://www.rfs.nsw.gov.au/fire-information/fires-near-me">RFS</a>'
  }).addTo(map);

  // Add each contour. Changed colors for each "risk level"
  let contours = await cached_contours()
  contours.forEach(function (contours, index) {
    L.polygon(contours.contours[0].map(point => [point.x, point.y]), {color: 'green'}).addTo(map)
    L.polygon(contours.contours[1].map(point => [point.x, point.y]), {color: 'orange'}).addTo(map)
    L.polygon(contours.contours[2].map(point => [point.x, point.y]), {color: 'red'}).addTo(map)
  })

};

main()