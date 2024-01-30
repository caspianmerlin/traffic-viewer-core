use std::{collections::HashMap, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread::{self, JoinHandle}, time::{Duration, Instant}};

use serde_json::Value;

use crate::vatsim::Details;

const VATSIM_DATA_URL: &str = "https://data.vatsim.net/v3/vatsim-data.json";
const VATSIM_METARS_URL: &str = "https://metar.vatsim.net/metar.php?id=all";

const METAR_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 10);
const VATSIM_DATA_REFRESH_INTERVAL: Duration = Duration::from_secs(20);





pub struct Worker {
    vatsim_data_last_refreshed: Instant,
    metars_last_refreshed: Instant,
    sender: Sender<WorkerCommand>,
    thread: Option<JoinHandle<()>>,
    metars: Arc<Mutex<HashMap<String, String>>>,
    vatsim_data: Arc<Mutex<HashMap<String, (Details, bool)>>>,
}

impl Worker {

    pub fn start() -> Worker {
        let (tx, rx) = mpsc::channel();
        let metars = Arc::new(Mutex::new(HashMap::new()));
        let vatsim_data = Arc::new(Mutex::new(HashMap::new()));

        let mut worker = Worker {
            vatsim_data_last_refreshed: Instant::now(),
            metars_last_refreshed: Instant::now(),
            sender: tx,
            thread: Some(worker_thread(rx, Arc::clone(&metars), Arc::clone(&vatsim_data))),
            metars,
            vatsim_data,
        };
        worker.refresh_metars();
        worker.refresh_vatsim_data();
        worker
    }
    pub fn refresh_metars(&mut self) {
        self.metars_last_refreshed = Instant::now();
        self.sender.send(WorkerCommand::RefreshMetars).ok();
    }

    pub fn refresh_vatsim_data(&mut self) {
        self.vatsim_data_last_refreshed = Instant::now();
        self.sender.send(WorkerCommand::RefreshVatsimData).ok();
    }

    pub fn tick(&mut self) {
        if self.metars_last_refreshed.elapsed() > METAR_REFRESH_INTERVAL {
            self.refresh_metars();
        }
        if self.vatsim_data_last_refreshed.elapsed() > VATSIM_DATA_REFRESH_INTERVAL {
            self.refresh_vatsim_data();
        }
    }

    pub fn get_metar(&self, icao: &str) -> Option<String> {
        let icao = icao.to_uppercase();
        self.metars.lock().unwrap().get(&icao).map(|value| value.clone())
    }

    pub fn get_aircraft_details(&mut self, callsign: &str) -> Option<(Details, bool)> {
        let callsign = callsign.to_uppercase();
        let mut data = self.vatsim_data.lock().unwrap();
        let mut ret_val = None;
        if let Some(value) = data.get_mut(&callsign) {
            ret_val = Some(value.clone());
            value.1 = false;
        }
        return ret_val;
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.sender.send(WorkerCommand::Stop).ok();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}
enum WorkerCommand {
    RefreshVatsimData,
    RefreshMetars,
    Stop,

}

fn worker_thread(receiver: Receiver<WorkerCommand>, metars: Arc<Mutex<HashMap<String, String>>>, vatsim_data: Arc<Mutex<HashMap<String, (Details, bool)>>>,) -> JoinHandle<()> {
    thread::Builder::new().name(String::from("TrafficViewerWorker")).spawn(move|| {
        loop {
            match receiver.recv() {
                Ok(command) => match command {
                    WorkerCommand::RefreshMetars => {
                        let mut count = 0;
                        match ureq::get(VATSIM_METARS_URL).call().ok().and_then(|response| response.into_string().ok()) {
                            Some(metar_file) => {
                                for line in metar_file.lines() {
                                    let icao = match line.split_whitespace().next() {
                                        Some(icao) => icao.to_owned(),
                                        None => continue,
                                    };
                                    if icao.len() < 4 { continue; }
                                    let mut metar_map = metars.lock().unwrap();
                                    metar_map.insert(icao, line.to_owned());
                                    count += 1;
                                }
                            },
                            None => println!("Unable to retrieve METARs from VATSIM"),
                        }
                        println!("{} METARs fetched", count);
                    },

                    WorkerCommand::RefreshVatsimData => {
                        let mut count = 0;
                        let mut new_count = 0;
                        let res = ureq::get(VATSIM_DATA_URL).call().unwrap().into_json::<Value>().unwrap();
                        let pilots = res.get("pilots").unwrap();
                        for value in pilots.as_array().unwrap().iter() {
                            let new = match serde_json::from_value::<Details>(value.clone()) {
                                Ok(details) => details,
                                Err(_) => continue,
                            };

                            let mut details_map = vatsim_data.lock().unwrap();
                            //let mut is_new = true;

                            // If we already have the details for this pilot
                            if let Some(existing) = details_map.get_mut(&new.callsign) {
                                // Set 'dirty' to true
                                existing.1 = true;
                                if let (Some(new_fp), Some(existing_fp)) = (&new.flight_plan, &existing.0.flight_plan) {
                                    if existing_fp.revision_id == new_fp.revision_id {
                                        // Only if there is a new flight plan and an old flight plan and the revision IDs are the same do we set 'dirty' to false
                                        // So basically we set dirty to true if:
                                        // - There was no old flight plan but there is a new one
                                        // - There was an old flight plan but there is no new one
                                        // - There was an old flight plan and a new one and the new one does not have the same revision ID as the old one
                                        existing.1 = false;
                                        //is_new = false;
                                    }
                                }
                                existing.0 = new;
                                count += 1;
                            } else {
                                details_map.insert(new.callsign.clone(), (new, true));
                                count += 1;
                            }
                            // if is_new { 
                            //     new_count += 1;
                            // }
                        };
                        println!("Details of {} aircraft fetched from VATSIM, of which {} were new", count, new_count);
                    },
                    WorkerCommand::Stop => break,
                },
                Err(_) => break,
            }
        }
    }).unwrap()
}