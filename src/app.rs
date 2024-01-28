use std::{ffi::CStr, net::TcpListener, ops::{Div, Mul}, thread, time::Duration};

use fsd_interface::{messages::{FlightPlanMessage, MetarRequestMessage, PilotPositionUpdateMessage, TextMessage}, FsdMessageType, PilotRating, TransponderCode, TransponderMode};

use crate::{fsuipc, server::Server, worker::Worker};

const SERVER_CALLSIGN: &str = "SERVER";


pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Traffic Viewer starting");

    // Establish a connection to FSUIPC
    println!("Attempting to connect to simulator");
    let versions = loop {
        match fsuipc::link(None) {
            Ok(versions) => break versions,
            Err(error) => {
                if let fsuipc::Error::NoSimConnection = error {
                    thread::sleep(Duration::from_secs(3));
                    continue;
                } else {
                    return Err(Box::new(error));
                }
            },
        }
    };
    println!("Connected to {} via FSUIPC {}", versions.fs_version, versions.fsuipc_version);

    // Start the worker thread which periodically downloads METARs and aircraft data from VATSIM
    let mut worker = Worker::start();

    // Wait for a connection from a controller client
    let tcp_listener = TcpListener::bind("127.0.0.1:6809").unwrap();
    println!("Waiting for ATC client connection...");
    let (tcp_stream, _) = tcp_listener.accept().unwrap();

    println!("Incoming connection request", );

    // Once we have a connection from a controller client, we can refresh the metars and vatsim data
    worker.refresh_metars();
    worker.refresh_vatsim_data();

    let mut count = 0;
    let mut server = Server::new(tcp_stream);

    let mut client_cs = None;

    loop {
        worker.tick();
        for message in server.poll() {
            match message {
                FsdMessageType::AtcRegisterMessage(msg) => {
                    client_cs = Some(msg.from.clone());
                    server.send_packet(&TextMessage::new(SERVER_CALLSIGN, msg.from, "Connected to Traffic Viewer. Welcome!").to_string());
                },
                FsdMessageType::MetarRequestMessage(msg) => {
                    if let Some(metar) = worker.get_metar(&msg.station) {
                        let res = server.send_packet(&MetarRequestMessage::new("server", msg.from, metar).to_string());
                        println!("Sent METAR message: {}", res);
                    }
                },
                _ => {},
            }
        }

        // Aircraft
        if count == 0 {
            let gnd_aircraft = fsuipc::get_aircraft(true);
            let air_aircraft = fsuipc::get_aircraft(false);

            for tcas_data in gnd_aircraft.iter().chain(air_aircraft.iter()) {
                let callsign = CStr::from_bytes_until_nul(&tcas_data.atc_id).unwrap();
                let callsign = match callsign.to_str() {
                    Ok(cs) => cs,
                    Err(_) => continue,
                };
                if let Some((details, dirty)) = worker.get_aircraft_details(callsign) {
                    if dirty {
                        if let Some(flight_plan) = details.flight_plan {
                            let flight_plan = fsd_interface::FlightPlan::from(flight_plan);
                            let to = client_cs.clone().unwrap_or(String::from("A*"));
                            server.send_packet(&FlightPlanMessage::new(to, callsign, flight_plan).to_string());
                        }
                    }

                    let alt_diff = ((details.qnh_i_hg - 29.92).mul(100.0).round().div(100.0) * 1000.0) as f64;

                    let position = PilotPositionUpdateMessage::new(callsign, TransponderMode::ModeC, TransponderCode::try_from(details.transponder.parse::<u16>().unwrap_or_default()).unwrap_or(TransponderCode::try_from(0).unwrap()), PilotRating::Student, tcas_data.lat as f64, tcas_data.lon as f64, tcas_data.alt as f64, tcas_data.alt as f64 - alt_diff, tcas_data.gs as u32, 0.0, 0.0, (tcas_data.hdg as f64 / HDG_FACTOR as f64).floor(), false);
                    
                    server.send_packet(&position.to_string());
                }
            };


            // Own aircraft
        if let Ok(own_aircraft_data) = fsuipc::get_own_aircraft_data() {
            if let Some(callsign) = &client_cs {
                if let Some((details, dirty)) = worker.get_aircraft_details(callsign) {
                    if dirty {
                        if let Some(flight_plan) = details.flight_plan {
                            let flight_plan = fsd_interface::FlightPlan::from(flight_plan);
                            let to = client_cs.clone().unwrap_or(String::from("A*"));
                            server.send_packet(&FlightPlanMessage::new(to, callsign, flight_plan).to_string());
                        }
                    }
                    let alt_diff = ((details.qnh_i_hg - 29.92).mul(100.0).round().div(100.0) * 1000.0) as f64;
                    let position = PilotPositionUpdateMessage::new(callsign, TransponderMode::ModeC, TransponderCode::try_from(own_aircraft_data.xpdr_str.parse::<u16>().unwrap_or_default()).unwrap_or_else(|_| TransponderCode::try_from(0).unwrap()), PilotRating::Student, own_aircraft_data.lat, own_aircraft_data.lon, own_aircraft_data.alt, own_aircraft_data.alt - alt_diff, own_aircraft_data.gs.floor() as u32, 0.0, 0.0, own_aircraft_data.true_hdg, false);
                    server.send_packet(&position.to_string());
                }
            }
        }


        }

        


        count += 1;
        if count == 4 {
            count = 0;
        }
        thread::sleep(Duration::from_secs(1));
    }

}








const HDG_FACTOR: f32 = 182.044444444;