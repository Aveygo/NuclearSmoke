use special::Gamma;
use statrs::distribution::{Normal, ContinuousCDF};

pub struct WSEG10 {
    translation: (f64, f64), mt_yield: f64, wind: f64, shear: f64, h_c: f64, s_0: f64, s_02: f64, s_h: f64, 
    t_c: f64, l_0: f64, l_02: f64, s_x2: f64, s_x: f64, l_2: f64, l: f64, n: f64, a_1: f64,
}

/*

    Blatantly sourced from https://gist.github.com/GOFAI/5e22c14d8a2c9644db4add3829e3bbde
    Applied minor changes:
     - Now only accepts yield, wind strength, and shear (will need to rotate points beforehand)
     - Units in metric

*/

impl WSEG10 {
    pub fn new(mt_yield: f64, wind_km_h: f64, shear_km_h_m: f64) -> Self {

        let wind: f64 = wind_km_h * 0.621371;
        let shear = shear_km_h_m * 0.189394;

        let translation = (-0.0, -0.0);
        let d = mt_yield.ln() + 2.42;
        let h_c = 44.0 + 6.1 * mt_yield.ln() - 0.205 * d.abs() * d;
        let lnyield = mt_yield.ln();
        let s_0 = (0.7 + lnyield / 3.0 - 3.25 / (4.0 + (lnyield + 5.4).powi(2))).exp();
        let s_02 = s_0.powi(2);
        let s_h = 0.18 * h_c;
        let t_c = 1.0573203 * (12.0 * (h_c / 60.0) - 2.5 * (h_c / 60.0).powi(2)) * (1.0 - 0.5 * (-((h_c / 25.0).powi(2))).exp());
        let l_0 = wind * t_c;
        let l_02 = l_0.powi(2);
        let s_x2 = s_02 * (l_02 + 8.0 * s_02) / (l_02 + 2.0 * s_02);
        let s_x = s_x2.sqrt();
        let l_2 = l_02 + 2.0 * s_x2;
        let l = l_2.sqrt();
        let n = (l_02 + s_x2) / (l_02 + 0.5 * s_x2);
        let a_1 = 1.0 / (1.0 + ((0.001 * h_c * wind) / s_0));

        WSEG10 {
            translation, mt_yield, wind, shear, h_c, s_0, s_02, s_h, 
            t_c, l_0, l_02, s_x2, s_x, l_2, l, n, a_1,
        }
    }

    fn g(&self, x: f64) -> f64 {
        let exponent = -(x.abs() / self.l).powf(self.n);
        let gamma_term = (1.0 + 1.0 / self.n).gamma();
        exponent.exp() / (self.l * gamma_term)
    }

    fn phi(&self, x: f64) -> f64 {
        let w = (self.l_0 / self.l) * (x / (self.s_x * self.a_1));
        let normal = Normal::new(0.0, 1.0).unwrap();
        normal.cdf(w)
    }

    pub fn d_hplus1(&self, x: f64, y: f64) -> f64 {
        /*
            Returns dose rate at x, y in R/hr at 1 hour after burst. This value includes dose rate from all activity that WILL be deposited at that location, not just that that has arrived by H+1 hr.
         */
        let (rx, ry) = (x + self.translation.0, y + self.translation.1);
        let f_x = self.mt_yield * 2e6 * self.phi(rx) * self.g(rx);
        let s_y = (
            self.s_02 +
            (8.0 * (rx + 2.0 * self.s_x) * self.s_02 / self.l) +
            (2.0 * (self.s_x * self.t_c * self.s_h * self.shear).powi(2) / self.l_2) +
            ((rx + 2.0 * self.s_x) * self.l_0 * self.t_c * self.s_h * self.shear).powi(2) / self.l.powi(4)
        ).sqrt();
        let a_2 = 1.0 / (1.0 + ((0.001 * self.h_c * self.wind) / self.s_0) * (1.0 - Normal::new(0.0, 1.0).unwrap().cdf(2.0 * x / self.wind)));
        let f_y = (-0.5 * (ry / (a_2 * s_y)).powi(2)).exp() / (2.5066282746310002 * s_y);
        f_x * f_y
    }

    pub fn fallout_toa(&self, x: f64) -> f64 {
        /*
            Average time-of-arrival for fallout along hotline at x. Minimum is 0.5hr for any location.
        */
        (0.25 + (self.l_02 * (x + 2.0 * self.s_x2) * self.t_c.powi(2)) / (self.l_2 * (self.l_02 + 0.5 * self.s_x2)) + ((2.0 * self.s_x2 * (1.0_f64).powi(2)) / (self.l_02 + 0.5 * self.s_x))).sqrt()
    }

    pub fn dose(&self, x_km: f64, y_km: f64) -> f64 {
        /* 
            Estimate of total "Equivalent Residual Dose" (ERD) in R at location x, y from time of fallout arrival to 30 days, including a 90% recovery factor.
        */

        let x = x_km * 1.60934;
        let y = y_km * 1.60934;

        let rx = x + self.translation.0;
        let t_a = self.fallout_toa(rx);
        let bio = (-(
            0.287 + 0.52 * (t_a / 31.6).ln() + 0.04475 * (t_a / 31.6).ln().powi(2)
        )).exp();
        let r = self.d_hplus1(x, y) * bio;
        if r.is_nan() {
            0.0
        } else {
            r 
        }

    }
}
