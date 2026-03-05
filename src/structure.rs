use chrono::{DateTime, Utc};
use round::round;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Weather {
    pub t: Option<f64>,
    pub p: Option<f64>,
    pub rh: Option<f64>,
    pub dp: Option<f64>,
    pub slp: Option<f64>,
    pub ws: Option<f64>,
    pub wd: Option<i32>,
}

struct _WeatherEx {
    pub prec_10min: Option<f64>,
    pub max_ws_10min: Option<f64>,
    pub min_ws_10min: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Station {
    pub station_id: i32,
    pub station_name: Option<String>,
    pub station_height: Option<f64>,
    pub station_lat: Option<f64>,
    pub station_lon: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElemUpload {
    pub station: Station,
    pub weather: Weather,
    pub timestamp: Option<DateTime<Utc>>,
}

impl ElemUpload {
    pub fn build(&mut self) {
        if (self.weather.t != None)
            && (self.weather.p != None)
            && (self.weather.rh != None)
            && (self.station.station_height != None)
        {
            let _t: f64 = self.weather.t.unwrap();
            let _p = self.weather.p.unwrap();
            let _rh = self.weather.rh.unwrap();
            let _h = self.station.station_height.unwrap();

            let slp = _calc_slp(_t, _p, _rh, _h);
            let dp = _calc_dp(_t, _rh);
            self.weather.slp = Some(slp);
            self.weather.dp = Some(dp);
            self.timestamp = Some(Utc::now());
        }
    }
}

fn _calc_slp(_t: f64, _p: f64, _rh: f64, _h: f64) -> f64 {
    const G: f64 = 9.80665;
    const RD: f64 = 287.05;
    const LAPSE: f64 = 0.0065;
    const EPSILON: f64 = 0.622;

    let t = _t + 273.15;
    let rh = _rh / 100.0;

    let es = 6.112 * ((17.67 * _t) / (_t + 243.5)).exp();
    let mut e = rh * es;

    e = e.min(_p * 0.99);
    let r = EPSILON * e / (_p - e);

    let q = r / (1.0 + r);

    let tv = t * (1.0 + 0.61 * q);
    let tv_mean = tv + 0.5 * LAPSE * _h;

    let slp = _p * (G * _h / (RD * tv_mean)).exp();

    round(slp as f64, 1)
}

fn _calc_dp(_t: f64, _rh: f64) -> f64 {
    let mut rh = _rh.max(0.1).min(100.0);
    rh = rh / 100.0;

    let a: f64 = 17.27;
    let b: f64 = 237.7;

    let gamma = (a * _t / (b + _t)) + rh.ln();

    let dp = (b * gamma) / (a - gamma);

    round(dp, 1)
}
