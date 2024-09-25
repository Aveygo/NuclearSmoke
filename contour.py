import numpy as np, sys, time, math
sys.path.append('glasstone/glasstone')
from fallout import WSEG10 # LINT will complain
from estimate_bounds import find_bounds
import numpy as np
import cv2 as cv2
from scipy.ndimage import gaussian_filter1d

from database import Fire

class ContourFinder:
    def __init__(self, fire:Fire):
        self.fire = fire

    def smoothing(self, polygon, sigma=1):
        polygon = np.array(polygon)
        x, y = polygon[:, 0], polygon[:, 1]
        x_smooth = gaussian_filter1d(x, sigma=sigma)
        y_smooth = gaussian_filter1d(y, sigma=sigma)
        return np.column_stack((x_smooth, y_smooth))

    def dose_calc(self, x, y, full_calc=False):
        """
        Very broken units - But in summary, please use metrics for self.fire
        3300 hectare is equiv. to ~MT, very rough estimation, assumes perfect burning.
        Contour is scaled down to mitigate this.
        """
        energy_multiplier = 1 if self.fire.under_control else 5
        energy_multiplier *= 1.01 ** (self.fire.temp - 25)
        
        if full_calc:
            print("Calculating contours:", {
                "name": self.fire.title,
                "under_control": self.fire.under_control,
                "temp": self.fire.temp,
                "mul": energy_multiplier,
                "size": self.fire.size_ha,
                "wind_speed": self.fire.wind_speed,
                "shear (metric)": self.fire.wind_shear,
                "shear (imperial)": abs(self.fire.wind_shear) * 190
            })

        X, Y = np.meshgrid(x, y)
        w = WSEG10(
            0, 0, max(self.fire.size_ha, 0.1) / 3300 * energy_multiplier, 1.0, abs(self.fire.wind_speed/1.609), 0, abs(self.fire.wind_shear) * 190, 
            dunits='mi', wunits='mph', yunits='MT', shearunits='mph/kilofoot'
        )

        dose = np.vectorize(w.D_Hplus1)
        return dose(X, Y, dunits='mi', doseunits='Roentgen') * energy_multiplier
        
    def rotate_polygon(self, polygon, angle_degrees):
        # Convert the angle from degrees to radians
        angle_radians = math.radians(angle_degrees)
        
        # Rotation matrix components
        cos_theta = math.cos(angle_radians)
        sin_theta = math.sin(angle_radians)
        
        rotated_polygon = []
        
        # Apply the rotation matrix to each point
        for x, y in polygon:
            new_x = x * cos_theta - y * sin_theta
            new_y = x * sin_theta + y * cos_theta
            rotated_polygon.append((new_x, new_y))
        
        return np.array(rotated_polygon)

    def contours_to_map(self, c, min_x, min_y, max_x, max_y, lat, long, direction, h_scale, v_scale):
        c = [i[0] for i in c]
        c = np.array([[i[0]-max_x*(1/h_scale) , -1 * (i[1] - max_y*(1/v_scale)+min_y*(1/v_scale))] for i in c])
        c = self.rotate_polygon(c, direction)
        c = c * 1.61 # To km
        c = c / 100 # To lat/long (assume earth is flat)
        c = c / 30 # ¯\_(ツ)_/¯ approximate scaling based on personal observations (helps later with uint8 too)
        
        # cv2 can give jagged edged due to low resolution
        c = self.smoothing(c)

        c = np.array([[round(i[0]+lat, 6), round(i[1]+long, 6)] for i in c])
        return c.tolist()

    def calc_contour(self, gray, thresh, min_x, min_y, max_x, max_y, h_scale, v_scale):
        ret, thresh = cv2.threshold(gray, thresh, 255, 0)
        contours, hierarchy = cv2.findContours(thresh, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

        if len(contours):
            return self.contours_to_map(
                contours[0], min_x, min_y, max_x, max_y, 
                self.fire.lat, self.fire.long, self.fire.wind_direction,
                h_scale, v_scale
            )

        return None

    def __call__(self):
        min_x, min_y, max_x, max_y = find_bounds(self.dose_calc)
        v_scale = (abs(max_y) + abs(min_y)) / 200 # 200 is the image width  & height
        h_scale = (abs(max_x) + abs(min_x)) / 200 # 

        x = np.arange(min_x, max_x, h_scale)
        y = np.arange(min_y, max_y, v_scale)
        
        z = self.dose_calc(x, y, full_calc=True)

        gray = ((z/5000)*255).clip(0, 255).astype(np.uint8)

        c10  = self.calc_contour(gray, 10,  min_x, min_y, max_x, max_y, h_scale, v_scale) # "Nearby smoker" - 0.295 mSv
        c80  = self.calc_contour(gray, 80,  min_x, min_y, max_x, max_y, h_scale, v_scale) # "Asthma risk" - 2.365 mSv
        c250 = self.calc_contour(gray, 250, min_x, min_y, max_x, max_y, h_scale, v_scale) # "Why does the air hurt" - 5.913 mSv

        return c10, c80, c250
    
if __name__ == "__main__":
    import matplotlib.pyplot as plt

    c = ContourFinder(Fire(
        lat=0,
        long=0,
        title="",
        published=0,
        category="",
        updated=0,
        level=0,
        size_ha=1,
        temp=26,
        under_control=True,
        wind_speed=2.8,
        wind_direction=0,
        wind_shear=0.0135
    ))

    min_x, min_y, max_x, max_y = find_bounds(c.dose_calc)
    print(min_x, min_y, max_x, max_y)

    v_scale = (abs(max_y) + abs(min_y)) / 200 # Max height
    h_scale = (abs(max_x) + abs(min_x)) / 200# Max width
    
    x = np.arange(min_x, max_x, h_scale)
    y = np.arange(min_y, max_y, v_scale)
    z = c.dose_calc(x, y, full_calc=True)

    gray = ((z/5000) *255).clip(0, 255).astype(np.uint8)
    image = np.expand_dims(gray, 2)
    image = np.repeat(image, 3, axis=2)
    ret, thresh = cv2.threshold(gray, 10, 255, 0)
    contours, hierarchy = cv2.findContours(thresh, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)

    if len(contours):
        r = c.contours_to_map(
            contours[0], min_x, min_y, max_x, max_y, 
            c.fire.lat, c.fire.long, c.fire.wind_direction,
            h_scale, v_scale
        )
        plt.scatter([i[0] for i in r], [i[1] for i in r])
        plt.savefig("plot2.png")

    cv2.drawContours(image, contours, -1, (0, 255, 0), 1) 
    cv2.imwrite("plot.png", image)

