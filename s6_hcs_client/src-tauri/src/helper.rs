use std::net::TcpStream;
use std::sync::{mpsc, mpsc::Sender};
use std::thread::JoinHandle;
use serde::{Serialize, Deserialize};
use tauri::{Window};
use websocket::ClientBuilder;
use websocket::sync::Client;


#[derive(Serialize, Deserialize, Clone)]
pub enum RequestProcessingError {
    NoConnection,
    BadRequest,
    ServerError,
    BadFile,
}


#[derive(Serialize, Deserialize, Clone)]
pub enum OperationProgress {
    Connecting,
    Encrypting(u8),
    Decrypting(u8),
    Uploading,
    Downloading,
    Done,
    Errored(RequestProcessingError),
}


pub fn err<T>(window: Window, event: &str, e: RequestProcessingError) -> Result<T, RequestProcessingError>{
    window.emit(event, OperationProgress::Errored(e.clone())).unwrap();
    Err(e)
}


pub fn connect(url: &str) -> Result<Client<TcpStream>, RequestProcessingError> {
    let client = match ClientBuilder::new(url) {
        Ok(client) => client,
        Err(_) => return Err(RequestProcessingError::NoConnection),
    };
    let client = client
        .add_protocol("rust-websocket")
        .connect_insecure();
    match client {
        Ok(client) => Ok(client),
        Err(_) => return Err(RequestProcessingError::NoConnection),
    }
}


pub fn make_progress_reporter(len: usize, cb: Box<(dyn Fn(u8) + Send)>) -> (Sender<Option<()>>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut cnt = 0;
        loop {
            match rx.recv() {
                Err(_) => return,
                Ok(None) => return,
                Ok(Some(_)) => {
                    cnt += 1;
                    if (cnt * 100 / len) < ((cnt + 1) * 100 / len) {
                        cb((cnt * 100 / len + 1) as u8);
                    }
                    if cnt >= len {
                        return;
                    }
                }
            }
        }
    });
    (tx, handle)
}


// pub fn encrypt(window: Window, event: &str, rx: Receiver<Option<u128>>) {
//     window.emit(event, Progress::Encrypting(0)).unwrap_or_default();
//     let (tx, handle) = make_progress_reporter(
//         contents.len(),
//         progress_callback,
//     );
//     let encrypted = DEALMode::RDH.encrypt(contents, key, tx);
//     tx.send(None).unwrap_or_default();
//     handle.join().unwrap_or_default();
//     window.emit(event, Progress::Encrypting(100)).unwrap_or_default();
//     encrypted
// }