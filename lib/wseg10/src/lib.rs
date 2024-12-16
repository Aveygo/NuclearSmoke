use ndarray::{Array, Array1, Array2};
use special::Gamma;
use statrs::distribution::{Normal, ContinuousCDF};
use std::f64::consts::PI;

pub struct WSEG10 {
    translation: (f64, f64),
    wd: f64,
    yld: f64,
    ff: f64,
    wind: f64,
    shear: f64,
    tob: f64,
    h_c: f64,
    s_0: f64,
    s_02: f64,
    s_h: f64,
    t_c: f64,
    l_0: f64,
    l_02: f64,
    s_x2: f64,
    s_x: f64,
    l_2: f64,
    l: f64,
    n: f64,
    a_1: f64,
}

impl WSEG10 {
    pub fn new(gzx: f64, gzy: f64, yld: f64, ff: f64, wind: f64, wd: f64, shear: f64, tob: f64) -> Self {
        let translation = (-gzx, -gzy);
        let d = yld.ln() + 2.42;
        let h_c = 44.0 + 6.1 * yld.ln() - 0.205 * d.abs() * d;
        let lnyield = yld.ln();
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
        let n = (ff * l_02 + s_x2) / (l_02 + 0.5 * s_x2);
        let a_1 = 1.0 / (1.0 + ((0.001 * h_c * wind) / s_0));

        WSEG10 {
            translation, wd, yld, ff, wind, shear, tob, h_c, s_0, 
            s_02, s_h, t_c, l_0, l_02, s_x2, s_x, l_2, l, n, a_1,
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
        let (rx, ry) = (x + self.translation.0, y + self.translation.1);
        let f_x = self.yld * 2e6 * self.phi(rx) * self.g(rx) * self.ff;
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
        let t_1: f64 = 1.0;
        (0.25 + (self.l_02 * (x + 2.0 * self.s_x2) * self.t_c.powi(2)) / (self.l_2 * (self.l_02 + 0.5 * self.s_x2)) + ((2.0 * self.s_x2 * t_1.powi(2)) / (self.l_02 + 0.5 * self.s_x))).sqrt()
    }

    pub fn dose(&self, x: f64, y: f64) -> f64 {
        let rx = x + self.translation.0;
        let t_a = self.fallout_toa(rx);
        let bio = (-(
            0.287 + 0.52 * (t_a / 31.6).ln() + 0.04475 * (t_a / 31.6).ln().powi(2)
        )).exp();
        self.d_hplus1(x, y) * bio
    }
}
