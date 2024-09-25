"""
This really should be a class. Just import find_bounds.
"""

import numpy as np

THRESH = 10

def _test_point(x, y, dose_calc):
    z = dose_calc(x, y)
    return np.max(z)

def _test_slice(x, y_min, y_max, dose_calc):
    y = np.arange(y_min, y_max, 0.1)
    z = dose_calc(x, y)
    return np.max(z)

def _find_upper(dose_calc):
    # Upwind
    y = 0.1
    last_y = y
    while True:
        current_score = _test_point(0, y, dose_calc)
        if current_score >= THRESH:
            last_y = y
            y *= 2
        else:
            tmp_last_y = y 
            y = y - (y-last_y)/2
            last_y = tmp_last_y
        
        if abs(y - last_y) < 0.1:
            return y

def _find_lower(dose_calc):
    # downwind
    y = -0.1
    last_y = y
    while True:
        current_score = _test_point(0, y, dose_calc)
        if current_score >= THRESH * 10:
            last_y = y
            y *= 2
        else:
            tmp_last_y = y 
            y = y - abs(y-last_y)/2
            last_y = tmp_last_y
        
        if abs(y - last_y) < 0.01:
            return y

def _find_x_max(min_y, max_y, dose_calc):
    # Much tricker! We need to take an entire vertical slice
    x = 0.1
    last_x = x
    while True:
        
        current_score = _test_slice(x, min_y, max_y, dose_calc)
        if current_score >= THRESH:
            last_x = x
            x *= 2
        else:
            tmp_last_x = x 
            x = x - (x-last_x)/2
            last_x = tmp_last_x
        
        if abs(x - last_x) < 0.1:
            return x
        
def find_bounds(dose_calc):
    """
    Find the min_x, min_y, max_x and max_y of ContourFinder.dose_calc
    """
    min_y, max_y = _find_lower(dose_calc), _find_upper(dose_calc)
    max_x = _find_x_max(min_y, max_y, dose_calc)
    min_x = -1 * max_x
    return round(min_x, 3), round(min_y, 3), round(max_x, 3), round(max_y, 3)
    