
import requests, threading, time, re, json
from datetime import datetime
from contour import ContourFinder
from database import Fire, Contour, Meta
import getgfs, math

class WatchDog(threading.Thread):
    def __init__(self):
        threading.Thread.__init__(self)
        self.check_every = 60 * 15
        self.last_checked = time.time() - self.check_every - 1
        self.url = "https://prod.dataportal.rfs.nsw.gov.au/majorIncidents.json"
        self.f = getgfs.Forecast("0p25")
        self.working=True

    def wind(self, u, v):
        wind_abs = (u**2 + v**2)**(1/2)
        wind_dir = math.atan2(u/wind_abs, v/wind_abs) 
        wind_dir = wind_dir * 180/math.pi
        wind_dir = wind_dir + 180
        wind_dir = 90 - wind_dir
        return float(wind_abs), float(wind_dir)

    def get_weather(self, long, lat, size_ha):
        if size_ha == 0:
            # Skip weather collection - contours will be NA anyways 
            return {
                "shear": 0,
                "speed": 0,
                "direction": 0,
                "temp": 0,
            }
        
        time.sleep(2) # Delay to prevent spamming

        now = datetime.now()
        current_time = f"{now.year}{now.month:02}{now.day:02} {now.hour:02}:{now.minute:02}"
        res = self.f.get(["ugrd10m", "vgrd10m", "vgrd100m", "ugrd100m", "tmax2m"],current_time, lat, long)
        
        u_10, v_10 = res.variables["ugrd10m"].data[0][0][0], res.variables["vgrd10m"].data[0][0][0]         # Wind at 10m above ground
        u_100, v_100 = res.variables["ugrd100m"].data[0][0][0], res.variables["vgrd100m"].data[0][0][0]     # Wind at 100m above ground
        wind_abs_10, wind_dir_10    = self.wind(u_10, v_10)
        wind_abs_100, wind_dir_100  = self.wind(u_100, v_100)

        shear = abs(wind_abs_10 - wind_abs_100) / 90 # 90 comes from 100m - 10m

        temp = float(res.variables["tmax2m"].data[0][0][0]) - 273.15 # Tempurature is in K
        
        return {
            "shear": shear,
            "speed": wind_abs_10,
            "direction": wind_dir_10,
            "temp": temp,
        }

    def get_updated(self, description, published) -> int:
        pattern = r"UPDATED:\s(\d{2})\s([A-Za-z]{3})\s(\d{4})\s(\d{2}):(\d{2})"
        match = re.search(pattern, description)

        epoch_time = published
        if match:
            day, month_str, year, hour, minute = match.groups()            
            month = datetime.strptime(month_str, "%b").month
            date_time = datetime(int(year), month, int(day), int(hour), int(minute))
            epoch_time = int(date_time.timestamp())
            
        return epoch_time 
    
    def handle_fire(self, geometry, feature) -> Fire:
        properties = feature.get("properties", {})
        title = properties.get("title", None)
        if title is None:
            return None # no way to store
        
        coordinates = geometry.get("coordinates", None)
        if coordinates is None:
            return None # Fire location is unknown

        description = properties.get("description", "na")
        published = properties.get("pubDate", 0)
        category = properties.get("category", "na")
        if category is None: category = "na" # I love python

        under_control = "under control" in description.lower()

        match = re.search(r'SIZE:\s*(\d+)\s*ha', description)
        size_ha = float(match.group(1)) if match else 0

        published = int(datetime.strptime(published, "%d/%m/%Y %I:%M:%S %p").timestamp())
        updated = self.get_updated(description, published)

        match = re.search(r"ALERT LEVEL:\s*(\w+ \w+)", description)
        alert_type = match.group(1) if match else "na"

        weather = self.get_weather(coordinates[0], coordinates[1], size_ha)

        # Check if we need to create or update the fire to likely save time computing contours
        query:Fire|None = Fire.get_or_none(Fire.title == title)
        fire = None

        fire = Fire(
            long = coordinates[0],
            lat = coordinates[1],
            title = title,
            published = published,
            category = category,
            updated = updated,
            created = time.time(),
            level = alert_type,
            size_ha = size_ha,
            under_control = under_control,

            temp = weather["temp"],
            wind_speed=weather["speed"],
            wind_direction=weather["direction"],
            wind_shear=weather["shear"]
        )
        
        if query is None:
            return fire 
        
        if (
            updated > query.updated                                     # If RFS updated the fire
            or time.time() + 60*60 < query.created                      # OR exisiting record expired
            or abs(query.wind_speed - weather["speed"]) > 5             # OR the wind changed speed
            or abs(query.wind_direction - weather["direction"]) > 40    # OR the wind changed direction
        ):
            print(query.created)
            print(updated > query.updated,time.time() + 60*60 < query.created,abs(query.wind_speed - weather["speed"]) > 3,  abs(query.wind_direction - weather["direction"]) > 20)
            query.delete_instance()                                     # Delete existing fire and create new one
            return fire
        
        return None

        
    def add_contour(self, fire:Fire):
        # Remove old contours
        for c in fire.contours:
            c.delete_instance()

        if fire.size_ha > 0:
            c10, c80, c250 = ContourFinder(fire)()
            fire.save()
            if not c10 is None: Contour.create(data = json.dumps(c10), thresh = 10, owner = fire)
            if not c80 is None: Contour.create(data = json.dumps(c80), thresh = 80, owner = fire)
            if not c250 is None: Contour.create(data = json.dumps(c250), thresh = 250, owner = fire)
            print(f"Contours added for fire: {fire.title}")

    def run(self):

        while True:
            self.working=False
            sleep_for = max(0, self.last_checked + self.check_every - time.time())
            print(f"Sleeping for {sleep_for} seconds")
            time.sleep(sleep_for)
            self.working=True
            self.last_checked = time.time()

            data = requests.get(self.url).json()
            num_updated_fires = 0
            for feature in data.get("features", []):
                if feature.get("type", "") == "Feature":
                    for geometry in feature.get("geometry", {}).get("geometries", []):
                        if geometry.get("type", "") == "Point":

                            fire:Fire = self.handle_fire(geometry, feature)
                            if not fire is None:
                                num_updated_fires += 1
                                self.add_contour(fire)

            if num_updated_fires > 0:                
                print(f"Added contours for {num_updated_fires} fires...")
            else:
                print("Did not update db")