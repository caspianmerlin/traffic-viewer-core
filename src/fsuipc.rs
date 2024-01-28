
use std::{ffi::{c_void, CStr}, mem::{self, MaybeUninit}, thread};


#[link(name = "User32", kind="dylib")]
extern {}

#[allow(unused)]
#[link(name = "FSUIPCuser64")]
extern "C" {
    pub fn FSUIPC_Open(fs_req: u32, result: *mut u32) -> i32;
    pub fn FSUIPC_Close();
    pub fn FSUIPC_Read(offset: u32, size: u32, destination: *mut c_void, result: *mut u32) -> i32;
    pub fn FSUIPC_ReadSpecial(offset: u32, size: u32, destination: *mut c_void, result: *mut u32) -> i32;
    pub fn FSUIPC_Write(offset: u32, size: u32, source: *const c_void, result: *mut u32) -> i32;
    pub fn FSUIPC_Process(result: *mut u32) -> i32;
    pub static FSUIPC_Version: u32;
    pub static FSUIPC_FS_Version: FlightSimVersion;
    pub static FSUIPC_Lib_Version: u32;
}

pub fn read<T>(offset: u32) -> Result<T, Error> {
    unsafe {
        let mut result = 0;
        let mut destination: MaybeUninit<T> = MaybeUninit::uninit(); 
        let size = mem::size_of::<T>() as u32;
        if FSUIPC_Read(offset, size, destination.as_mut_ptr() as *mut _, &mut result) != 1 || FSUIPC_Process(&mut result) != 1 {
            let result = mem::transmute(result);
            return Err(result);
        }
        return Ok(destination.assume_init());
    }
}

#[derive(Debug, Clone)]
pub struct Versions {
    pub fsuipc_version: String,
    pub fs_version: FlightSimVersion,
    pub fsuipc_lib_version: u32,
}

pub fn link(required_fs_version: Option<FlightSimVersion>) -> Result<Versions, Error> {
    unsafe {
        let mut result = 0;
        let fs_req = match required_fs_version {
            Some(fs_req) => fs_req as u32,
            None => 0,
        };
        if FSUIPC_Open(fs_req, &mut result) != 1 {
            return Err(mem::transmute(result));
        }

        let v_a: u8 = 48 + (0x0f & (FSUIPC_Version >> 28)) as u8;
        let v_b: u8 = 48 + (0x0f & (FSUIPC_Version >> 24)) as u8;
        let v_c: u8 = 48 + (0x0f & (FSUIPC_Version >> 20)) as u8;
        let v_d: u8 = 48 + (0x0f & (FSUIPC_Version >> 16)) as u8;
        let v_e: u8 = if (FSUIPC_Version & 0xffff) > 0 { 97 + (FSUIPC_Version & 0xff) as u8 - 1 } else { 32 };
        let mut fsuipc_version = format!("{}.{}{}{}",
            char::from_u32_unchecked(v_a as u32),
            char::from_u32_unchecked(v_b as u32),
            char::from_u32_unchecked(v_c as u32),
            char::from_u32_unchecked(v_d as u32),
        );
        if v_e != 32 {
            fsuipc_version.push(char::from_u32_unchecked(v_e as u32));
        }
        set_preferences();
        return Ok(Versions { fsuipc_version: fsuipc_version, fs_version: FSUIPC_FS_Version, fsuipc_lib_version: FSUIPC_Lib_Version })
    }
}


#[allow(unused)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum FlightSimVersion {
    FS98 = 1,
    FS2000,
    CFS2,
    CFS1,
    Fly,
    FS2002,
    FS2004,
    FSX,
    ESP,
    P3D,
    FSX64,
    P3D64,
    MSFS,
}

impl std::fmt::Display for FlightSimVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
    FlightSimVersion::FS98 => "Microsoft Flight Simulator 98",
    FlightSimVersion::FS2000 => "Microsoft Flight Simulator 2000",
    FlightSimVersion::CFS2 => "Microsoft Combat Flight Simulator 2",
    FlightSimVersion::CFS1 => "Microsoft Combat Flight Simulator",
    FlightSimVersion::Fly => "Fly",
    FlightSimVersion::FS2002 => "Microsoft Flight Simulator 2002",
    FlightSimVersion::FS2004 => "Microsoft Flight Simulator 2004",
    FlightSimVersion::FSX => "Microsoft Flight Simulator X",
    FlightSimVersion::ESP => "Microsoft ESP",
    FlightSimVersion::P3D => "Prepar3D",
    FlightSimVersion::FSX64 => "Microsoft Flight Simulator X x64",
    FlightSimVersion::P3D64 => "Prepar3D x64",
    FlightSimVersion::MSFS => "Microsoft Flight Simulator 2020",
})
    }
}

#[allow(unused)]
#[derive(Debug)]
#[repr(u32)]
pub enum Error {
    /// Attempt to Open when already Open
    Open = 1,
    /// Cannot link to FSUIPC or WideClient
    NoSimConnection,
    /// Failed to Register common message with Windows
    RegisterMessage,
    /// Failed to create Atom for mapping filename
    Atom,
    /// Failed to create a file mapping object
    Map,
    /// Failed to open a view to the file map
    View,
    /// Incorrect version of FSUIPC, or not FSUIPC
    FSUIPCVersion,
    /// Sim is not version requested
    WrongFSVersion,
    /// Call cannot execute, link not Open
    NotOpen,
    /// Call cannot execute: no requests accumulated
    NoData,
    /// IPC timed out all retries
    TimedOut,
    /// IPC sendmessage failed all retries
    SendMessage,
    /// IPC request contains bad data
    BadData,
    /// Maybe running on WideClient, but FS not running on Server, or wrong FSUIPC
    Running,
    /// Read or Write request cannot be added, memory for Process is full
    Size,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::Open => "Already connected to simulator",
            Error::NoSimConnection => "Unable to connect to simulator",
            Error::RegisterMessage => "Failed to register common message with Windows",
            Error::Atom => "Failed to create Atom for mapping filename",
            Error::Map => "Failed to create a file mapping object",
            Error::View => "Failed to open a view to the file map",
            Error::FSUIPCVersion => "Incorrect version of FSUIPC, or not FSUIPC",
            Error::WrongFSVersion => "Simulator is not version requested",
            Error::NotOpen => "Call cannot execute, link not open",
            Error::NoData => "Call cannot execute: no requests accumulated",
            Error::TimedOut => "IPC timed out all retries",
            Error::SendMessage => "IPC SendMessage failed all retries",
            Error::BadData => "IPC request contains bad data",
            Error::Running => "Maybe running on WideClient, but FS not running on server, or wrong FSUIPC",
            Error::Size => "Read or Write request cannot be added, memory for Process is full",
        })
    }
}

impl std::error::Error for Error {}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TcasData {
    pub id: u32,
    pub lat: f32,
    pub lon: f32,
    pub alt: f32,
    pub hdg: u16,
    pub gs: u16,
    pub vs: i16,
    pub atc_id: [u8; 15],
    pub state: u8,
    pub com1: u16,
}

#[allow(unused)]
#[repr(u8)]
enum State {
    Initialising = 128,
    Sleeping,
    FilingFlightPlan,
    ObtainingClearance,
    PushBack,
    PushBackTurning,
    StartingUp,
    PreparingToTaxi,
    TaxiingOut,
    LiningUp,
    TakingOff,
    Departing,
    Enroute,
    InCircuit,
    Landing,
    RollingOut,
    GoingAround,
    TaxiingIn,
    ShuttingDown,
}

fn set_preferences() {
    unsafe {
        let (range_a_offset, range_b_offset, tcas_id_option_offset) = (0xE068, 0xE069, 0xE06A);
        let range = 0_u8;
        let mut result = 0;
        if FSUIPC_Write(range_a_offset, mem::size_of::<u8>() as u32, &range as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        if FSUIPC_Write(range_b_offset, mem::size_of::<u8>() as u32, &range as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        let tcas_id_option = 0_u8;
        if FSUIPC_Write(tcas_id_option_offset, mem::size_of::<u8>() as u32, &tcas_id_option as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        let (range_a_offset, range_b_offset, tcas_id_option_offset) = (0xF068, 0xF069, 0xF06A);
        let range = 0_u8;
        let mut result = 0;
        if FSUIPC_Write(range_a_offset, mem::size_of::<u8>() as u32, &range as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        if FSUIPC_Write(range_b_offset, mem::size_of::<u8>() as u32, &range as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        let tcas_id_option = 0_u8;
        if FSUIPC_Write(tcas_id_option_offset, mem::size_of::<u8>() as u32, &tcas_id_option as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        if FSUIPC_Process(&mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }
    }
}


pub fn get_aircraft(on_ground: bool) -> Vec<TcasData> {
    unsafe {
        set_preferences();
        let (range_a_offset, range_b_offset, tcas_id_option_offset) = if on_ground {
            (0xE068, 0xE069, 0xE06A)
        } else {
            (0xF068, 0xF069, 0xF06A)
        };
        let range = 0_u8;
        let mut result = 0;
        if FSUIPC_Write(range_a_offset, mem::size_of::<u8>() as u32, &range as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        if FSUIPC_Write(range_b_offset, mem::size_of::<u8>() as u32, &range as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }

        let tcas_id_option = 0_u8;
        if FSUIPC_Write(tcas_id_option_offset, mem::size_of::<u8>() as u32, &tcas_id_option as *const u8 as *const _, &mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }
        if FSUIPC_Process(&mut result) != 1 {
            let result: Error = mem::transmute(result);
            panic!("Error: {:?}", result);
        }
    }


    let offset = if on_ground { 0xE004 } else { 0xF004 };
    let num_aircraft = read::<u16>(offset).unwrap() as usize;
    println!("Number of {} aircraft: {}", if on_ground { "ground" } else { "airborne" }, num_aircraft);
    let mut vec: Vec<MaybeUninit<TcasData>> = vec![MaybeUninit::uninit(); num_aircraft];
    let mut ret_vec = Vec::with_capacity(num_aircraft);
    if num_aircraft == 0 {
        return vec![];
    }
    for i in 0..num_aircraft {
        let offset = if on_ground { 0xE080 } else { 0xF080 }; 
        let vec_offset = i * mem::size_of::<TcasData>();
        let offset = offset + vec_offset;
        unsafe {
            let mut result = 0;
            if FSUIPC_Read(offset as u32, mem::size_of::<TcasData>() as u32, vec.as_mut_ptr().byte_add(vec_offset) as *mut Vec<MaybeUninit<TcasData>> as *mut _, &mut result) != 1 {
                panic!("Read error: {result}");
            }
        }
    }
    let mut result = 0;
    unsafe {
        if FSUIPC_Process(&mut result) != 1 {
            panic!("Read error");
        }


        for aircraft in vec.into_iter() {
            let tcas_data = aircraft.assume_init();
            if tcas_data.id == 0 { continue; }
            ret_vec.push(tcas_data);
        }



        return ret_vec;
    }
    

}

const KNOTS_PER_M_PER_S: f64 = 1.943844;
const FEET_PER_M: f64 = 3.28084;

pub fn get_own_aircraft_data() -> Result<OwnAircraftData, Error> {
    let mut lat: f64 = 0.0;
    let mut lon: f64 = 0.0;
    let mut alt_m: f64 = 0.0;
    let mut true_hdg_radians: f64 = 0.0;
    let mut xpdr: u16 = 0;
    let mut gs_m_per_s: f64 = 0.0;
    let mut res = 0;

    unsafe {
        if FSUIPC_Read(0x6010, mem::size_of::<f64>() as u32, &mut lat as *mut f64 as *mut _, &mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        if FSUIPC_Read(0x6018, mem::size_of::<f64>() as u32, &mut lon as *mut f64 as *mut _, &mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        if FSUIPC_Read(0x6020, mem::size_of::<f64>() as u32, &mut alt_m as *mut f64 as *mut _, &mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        if FSUIPC_Read(0x6038, mem::size_of::<f64>() as u32, &mut true_hdg_radians as *mut f64 as *mut _, &mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        if FSUIPC_Read(0x6030, mem::size_of::<f64>() as u32, &mut gs_m_per_s as *mut f64 as *mut _, &mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        if FSUIPC_Read(0x354, mem::size_of::<u16>() as u32, &mut xpdr as *mut u16 as *mut _, &mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        if FSUIPC_Process(&mut res) != 1 {
            let result: Error = mem::transmute(res);
            return Err(result);
        }
        let alt_ft = alt_m * FEET_PER_M;
        let xpdr_str = format!("{:4X}", xpdr);


        Ok(
            OwnAircraftData {
                lat,
                lon,
                alt: alt_ft,
                true_hdg: true_hdg_radians.to_degrees(),
                gs: gs_m_per_s * KNOTS_PER_M_PER_S,
                xpdr_str,
            }
        )
    }
}

#[derive(Debug)]
pub struct OwnAircraftData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub true_hdg: f64,
    pub gs: f64,
    pub xpdr_str: String,
}

#[allow(unused)]
unsafe fn get_string(ai_ac_id: u32, desired_value: StringType) -> Result<String, Error> {
    let mut result = 0;
    let source_a = desired_value as u32;
    if FSUIPC_Write(0xD004, mem::size_of::<u32>() as u32, &source_a as *const u32 as *const _, &mut result) != 1 {
        let result: Error = mem::transmute(result);
        return Err(result);
    }

    let mut timestamp = 0_u32;
    let mut result = 0;
    if FSUIPC_Read(0xD008, mem::size_of::<u32>() as u32, &mut timestamp as *mut u32 as *mut _, &mut result) != 1 {
        let result: Error = mem::transmute(result);
        return Err(result);
    }

    let mut result = 0;
    let source_b = ai_ac_id;
    if FSUIPC_Write(0xD00C, mem::size_of::<u32>() as u32, &source_b as *const u32 as *const _, &mut result) != 1 {
        let result: Error = mem::transmute(result);
        return Err(result);
    }

    let mut result = 0;
    let source_c = 16_u32;
    if FSUIPC_Write(0xD000, mem::size_of::<u32>() as u32, &source_c as *const u32 as *const _, &mut result) != 1 {
        let result: Error = mem::transmute(result);
        return Err(result);
    }

    let mut result = 0;
    if FSUIPC_Process(&mut result) != 1 {
        let result: Error = mem::transmute(result);
        return Err(result);
    }

    let mut i = 0;
    loop {
        if i == 200 {
            return Err(Error::TimedOut);
        }
        let new_timestamp: u32 = read(0xD008)?;
        if new_timestamp == timestamp {
            thread::sleep(std::time::Duration::from_millis(10));
            i += 1;
            continue;
        }

        let string: [u8; 48] = read(0xD010)?;
        let c_str = CStr::from_bytes_until_nul(&string).unwrap();
        let str = c_str.to_str().unwrap();
        return Ok(str.to_owned());
    }
}


#[allow(unused)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum StringType {
    TailNumber = 1,
    AirlineNameAndFlightNumber,
    AtcAcTypeAndModel,
    AircraftTitle,
    AtcAcTypeAndLastThreeOfTail,
}

#[allow(unused)]
#[derive(Debug)]
struct StringData {
    tail_number: String,
    airline_name_and_flight_number: String,
    atc_ac_type_and_model: String,
    aircraft_title: String,
    atc_ac_type_and_last_three_of_tail: String,
}
impl StringData {
    #[allow(unused)]
    pub fn get(ai_ac_id: u32) -> Result<StringData, Error> {
        unsafe {
            let tail_number = get_string(ai_ac_id, StringType::TailNumber)?;
            let airline_name_and_flight_number = get_string(ai_ac_id, StringType::AirlineNameAndFlightNumber)?;
            let atc_ac_type_and_model = get_string(ai_ac_id, StringType::AtcAcTypeAndModel)?;
            let aircraft_title = get_string(ai_ac_id, StringType::AircraftTitle)?;
            let atc_ac_type_and_last_three_of_tail = get_string(ai_ac_id, StringType::AtcAcTypeAndLastThreeOfTail)?;

            Ok(
                StringData {
                    tail_number,
                    airline_name_and_flight_number,
                    atc_ac_type_and_model,
                    aircraft_title,
                    atc_ac_type_and_last_three_of_tail
                }
            )
        }
    }
}