/*

  If you are looking at this, then you probably want to know how to calculate the smoke contours yourself.
  Unfortunately, there is no API here, and is instead all calculated using WASM with the "smoke" function.

  Please see the repo for more details: https://github.com/Aveygo/NuclearSmoke

*/

import init, { smoke } from "./pkg/nuclearsmoke.js";

function panic() {
  alert("Panic state!")
}


async function fetch_json(url) {
  let report = await fetch(url)
  return await report.json()
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

async function handle_fire(fire) {
  /*
    Asynchronously fetch weather data & calculate contours 
  */
  
  // Hectares may not be present, so we assume size = 0
  const match = fire.properties.description.match(/SIZE:\s*(\d+)\s*ha/);
  let hectares = 0;
  if (match) {
    hectares = parseInt(match[1], 10);
  }
  
  // Only compute for fires with a "center" position
  let maybe_point = fire.geometry.geometries.at(-1)
  if (maybe_point.type == "Point" && hectares > 0) {

    // Assumed as center of fire
    let lat = maybe_point.coordinates[1]
    let lon = maybe_point.coordinates[0]

    let weather = await fetch_json("https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lon + "&hourly=temperature_2m,wind_speed_10m,wind_speed_120m,wind_direction_10m,wind_direction_120m&forecast_days=1")

    // Get current hour as weather returns hourly data
    const d = new Date();
    let hour = d.getUTCHours();

    // Weather data
    let wind = (weather.hourly.wind_speed_10m[hour] + weather.hourly.wind_speed_120m[hour]) / 2
    let wind_direction = (weather.hourly.wind_direction_10m[hour] + weather.hourly.wind_direction_120m[hour]) / 2
    let shear = Math.abs(weather.hourly.wind_speed_10m[hour] - weather.hourly.wind_speed_120m[hour]) / 110
    let temp = weather.hourly.temperature_2m[hour]

    // Contour data
    let result_10   = JSON.parse(smoke(lat, lon, hectares/1000*temp, wind/5, wind_direction, shear*5, 10));
    let result_100  = JSON.parse(smoke(lat, lon, hectares/1000*temp, wind/5, wind_direction, shear*5, 100));
    let result_1000 = JSON.parse(smoke(lat, lon, hectares/1000*temp, wind/5, wind_direction, shear*5, 1000));
    
    // Return found data
    return {"fire": fire, "contours": [result_10, result_100, result_1000]}
  }
}

async function build_contours() {
  
  // Fetch the fires-near-me data
  let fires = await fetch_json("https://prod.dataportal.rfs.nsw.gov.au/majorIncidents.json");

  // Async each fire to spread out weather requests
  let jobs = [];
  for (let i=0; i<fires.features.length;i++) {
    jobs.push(handle_fire(fires.features[i]));
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
  await init();
  var map = L.map('map').setView([ -33.8521, 151.1917], 10); // Sydney

  L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a> <span>|</span> Data from <a href="https://www.rfs.nsw.gov.au/fire-information/fires-near-me">RFS</a>'
  }).addTo(map);

  // Add each contour for each "risk level"
  let contours = await cached_contours()
  contours.forEach(function (contours, index) {
    L.polygon(contours.contours[0].map(point => [point.x, point.y]), {color: 'green'}).addTo(map)
    L.polygon(contours.contours[1].map(point => [point.x, point.y]), {color: 'orange'}).addTo(map)
    L.polygon(contours.contours[2].map(point => [point.x, point.y]), {color: 'red'}).addTo(map)
  })

};

main()