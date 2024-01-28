use std::{io::{BufRead, BufReader, LineWriter, Write}, net::TcpStream, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc}, thread::{self, JoinHandle}, time::Duration};

use fsd_interface::FsdMessageType;




pub struct Server {
    recv_thread: Option<JoinHandle<()>>,
    writer: LineWriter<TcpStream>,
    should_stop: Arc<AtomicBool>,
    receiver: Receiver<FsdMessageType>,
}
impl Server {
    pub fn new(tcp_stream: TcpStream) -> Server {
        tcp_stream.set_write_timeout(Some(Duration::from_secs(2))).ok();
        let (tx, rx) = mpsc::channel();
        let should_stop = Arc::new(AtomicBool::new(false));
        let recv_thread = Some(recv_thread(Arc::clone(&should_stop), tcp_stream.try_clone().unwrap(), tx));
        Server {
            recv_thread,
            writer: LineWriter::new(tcp_stream),
            should_stop,
            receiver: rx,
        }
    }

    pub fn poll(&mut self) -> Vec<FsdMessageType> {
        let mut vec = vec![];
        while let Ok(msg) = self.receiver.try_recv() {
            vec.push(msg);
        }
        return vec;
    }

    pub fn send_packet(&mut self, message: &str) -> bool {
        match self.writer.write(&string_to_byte_slice(&format!("{message}\r\n"))) {
            Ok(0) | Err(_) => return false,
            Ok(_) => return true,
        }
    }

}
impl Drop for Server {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.recv_thread.take() {
            thread.join().ok();
        }
    }
}


fn recv_thread(should_stop: Arc<AtomicBool>, tcp_stream: TcpStream, sender: Sender<FsdMessageType>) -> JoinHandle<()> {
    thread::Builder::new().name(String::from("TrafficViewerRecvThread")).spawn(move|| {
        let mut reader = BufReader::new(tcp_stream);
        

        while !should_stop.load(Ordering::Relaxed) {
            let mut buffer = Vec::with_capacity(512);
            match reader.read_until(b'\n', &mut buffer) {
                Ok(0) => {
                    println!("Connection to controller client ended");
                    break;
                },
                Ok(_) => {
                    let message = byte_slice_to_string(&buffer);
                    println!("RECV: {}", message.trim());
                    if let Ok(fsd_message) = fsd_interface::parse_message(message.trim()) {
                        if let Err(e) = sender.send(fsd_message) {
                            println!("{:?}", e);
                            break;
                        }
                    }
                },
                Err(e) => {
                    println!("{:?}", e);
                            break;
                },
            }
        }
        println!("RECV THREAD ENDING");
    }).unwrap()
}



#[inline]
fn byte_slice_to_string(slice: &[u8]) -> String {
    slice.iter().map(|c| *c as char).collect()
}

#[inline]
fn string_to_byte_slice(string: &str) -> Vec<u8> {
    string.chars().map(|c| c as u8).collect()
}