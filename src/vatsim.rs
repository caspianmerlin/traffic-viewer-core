
use serde::Deserialize;



#[derive(Debug, Deserialize, Clone)]
pub struct Details {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub transponder: String,
    pub altitude: i32,
    pub heading: u32,
    pub qnh_i_hg: f32,
    pub flight_plan: Option<FlightPlan>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct FlightPlan {
    pub flight_rules: FlightRules,
    #[serde(rename = "aircraft_faa")]
    pub aircraft_type: String,
    #[serde(rename = "departure")]
    pub departure_icao: String,
    #[serde(rename = "arrival")]
    pub arrival_icao: String,
    #[serde(rename = "alternate")]
    pub alternate_icao: String,
    pub cruise_tas: String,
    altitude: String,
    #[serde(rename = "deptime")]
    departure_time: String,
    enroute_time: String,
    fuel_time: String,
    pub remarks: String,
    pub route: String,
    pub revision_id: usize,
    pub assigned_transponder: String,
}
impl FlightPlan {
    pub fn altitude(&self) -> i32 {
        match self.altitude.parse::<i32>() {
            Ok(alt) => return alt,
            Err(_) if self.altitude.starts_with("FL") && self.altitude.len() > 2 => {
                match self.altitude[2..].parse::<i32>() {
                    Ok(alt) => return alt * 100,
                    Err(_) => return 0,
                }
            },
            Err(_) => return 0,
        }
    }
    pub fn departure_time(&self) -> (u8, u8) {
        let hours = self.departure_time.get(0..2).and_then(|hours| hours.parse::<u8>().ok()).unwrap_or_default();
        let mins = self.departure_time.get(2..2).and_then(|mins| mins.parse::<u8>().ok()).unwrap_or_default();
        (hours, mins)
    }
    pub fn enroute_time(&self) -> (u8, u8) {
        let hours = self.enroute_time.get(0..2).and_then(|hours| hours.parse::<u8>().ok()).unwrap_or_default();
        let mins = self.enroute_time.get(2..2).and_then(|mins| mins.parse::<u8>().ok()).unwrap_or_default();
        (hours, mins)
    }
    pub fn fuel_time(&self) -> (u8, u8) {
        let hours = self.fuel_time.get(0..2).and_then(|hours| hours.parse::<u8>().ok()).unwrap_or_default();
        let mins = self.fuel_time.get(2..2).and_then(|mins| mins.parse::<u8>().ok()).unwrap_or_default();
        (hours, mins)
    }
}

impl From<FlightPlan> for fsd_interface::FlightPlan {
    fn from(value: FlightPlan) -> Self {
        let (hours_enroute, mins_enroute) = value.enroute_time();
        let (hours_fuel, mins_fuel) = value.fuel_time();
        let alt = value.altitude() as u32;
        Self {
            flight_rules: match value.flight_rules {
                FlightRules::DVFR => fsd_interface::FlightRules::DVFR,
                FlightRules::VFR => fsd_interface::FlightRules::VFR,
                FlightRules::SVFR => fsd_interface::FlightRules::SVFR,
                FlightRules::IFR => fsd_interface::FlightRules::IFR,
            },
            ac_type: value.aircraft_type,
            filed_tas: value.cruise_tas.parse().unwrap_or(0),
            origin: value.departure_icao,
            etd: value.departure_time.parse().unwrap_or(0),
            atd: value.departure_time.parse().unwrap_or(0),
            cruise_level: alt,
            destination: value.arrival_icao,
            hours_enroute,
            mins_enroute,
            hours_fuel,
            mins_fuel,
            alternate: value.alternate_icao,
            remarks: value.remarks,
            route: value.route,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum FlightRules {
    #[serde(rename = "D")]
    DVFR,
    #[serde(rename = "V")]
    VFR,
    #[serde(rename = "S")]
    SVFR,
    #[serde(rename = "I")]
    IFR,
}