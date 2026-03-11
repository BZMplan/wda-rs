use chrono::Utc;
use round::round;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
    pub timestamp: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ElemGet {
    pub station_id: i32,
    pub station_name: Option<String>,
    pub station_height: Option<f64>,
    pub station_lat: Option<f64>,
    pub station_lon: Option<f64>,
    pub timestamp: Option<i64>,
    pub t: Option<f64>,
    pub p: Option<f64>,
    pub rh: Option<f64>,
    pub dp: Option<f64>,
    pub slp: Option<f64>,
    pub ws: Option<f64>,
    pub wd: Option<i32>,
}

impl ElemUpload {
    pub fn build(&mut self) {
        if let (Some(t), Some(p), Some(rh), Some(h)) = (
            self.weather.t,
            self.weather.p,
            self.weather.rh,
            self.station.station_height,
        ) {
            let slp = _calc_slp(t, p, rh, h);
            let dp = _calc_dp(t, rh);
            self.weather.slp = Some(slp);
            self.weather.dp = Some(dp);
        }
        self.timestamp.get_or_insert_with(|| Utc::now().timestamp());
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

    round(slp, 1)
}

fn _calc_dp(_t: f64, _rh: f64) -> f64 {
    let mut rh = _rh.clamp(0.1, 100.0);
    rh /= 100.0;

    let a: f64 = 17.27;
    let b: f64 = 237.7;

    let gamma = (a * _t / (b + _t)) + rh.ln();

    let dp = (b * gamma) / (a - gamma);

    round(dp, 1)
}

#[cfg(test)]
mod tests {
    use super::{ElemUpload, Station, Weather};

    fn sample_upload() -> ElemUpload {
        ElemUpload {
            station: Station {
                station_id: 54511,
                station_name: Some("Test".to_string()),
                station_height: Some(50.0),
                station_lat: Some(39.9),
                station_lon: Some(116.4),
            },
            weather: Weather {
                t: Some(25.0),
                p: Some(1000.0),
                rh: Some(60.0),
                dp: None,
                slp: None,
                ws: Some(3.0),
                wd: Some(180),
            },
            timestamp: None,
        }
    }

    #[test]
    fn build_sets_timestamp_when_missing() {
        let mut upload = sample_upload();
        upload.build();
        assert!(upload.timestamp.is_some());
    }

    #[test]
    fn build_calculates_dp_and_slp_when_fields_are_complete() {
        let mut upload = sample_upload();
        upload.build();
        assert!(upload.weather.dp.is_some());
        assert!(upload.weather.slp.is_some());
    }

    #[test]
    fn build_keeps_dp_and_slp_empty_when_required_fields_missing() {
        let mut upload = sample_upload();
        upload.station.station_height = None;
        upload.build();
        assert!(upload.weather.dp.is_none());
        assert!(upload.weather.slp.is_none());
        assert!(upload.timestamp.is_some());
    }
}
