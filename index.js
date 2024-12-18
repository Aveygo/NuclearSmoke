import init, { smoke } from "./pkg/nuclearsmoke.js";

const runWasm = async (map, L) => {
  
  await init();
  console.time('test');
  const result = JSON.parse(smoke(-33.8688, 151.2093, 1000.0, 20.0, 90, 0.2, 10));
  console.timeEnd('test');
  console.log(result);

  var polygon = L.polygon(result.map(point => [point.x, point.y]), {color: 'green'}).addTo(map)
  
};

var map = L.map('map').setView([ -33.8521, 151.1917], 10);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a> <span>|</span> Data from <a href="https://www.rfs.nsw.gov.au/fire-information/fires-near-me">RFS</a>'
}).addTo(map);

runWasm(map, L);
