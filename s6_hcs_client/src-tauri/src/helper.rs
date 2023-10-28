use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::sync::{mpsc, mpsc::Sender};
use std::thread::JoinHandle;
use websocket::sync::Client;
use websocket::ClientBuilder;

#[derive(Serialize, Deserialize, Clone)]
pub enum RequestProcessingError {
    NoConnection,
    BadRequest,
    ServerError,
    BadFile,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OperationProgress {
    Connecting(u8),
    Encrypting(u8),
    Decrypting(u8),
    Uploading(u8),
    Downloading(u8),
}

pub fn connect(url: &str) -> Result<Client<TcpStream>, RequestProcessingError> {
    let client = match ClientBuilder::new(url) {
        Ok(client) => client,
        Err(_) => return Err(RequestProcessingError::NoConnection),
    };
    let client = client.add_protocol("rust-websocket").connect_insecure();
    match client {
        Ok(client) => Ok(client),
        Err(_) => return Err(RequestProcessingError::NoConnection),
    }
}

pub fn make_progress_reporter(
    len: usize,
    cb: Box<(dyn Fn(u8) + Send)>,
) -> (Sender<Option<()>>, JoinHandle<()>) {
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
