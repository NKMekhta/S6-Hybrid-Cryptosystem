// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            server_calls::get_files,
            server_calls::upload,
            server_calls::download,
            server_calls::delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



pub mod server_calls {
    use std::fs;
    use std::net::TcpStream;
    use std::path::PathBuf;
    use s6_hcs_lib_crypto::crypto::{
		padding::{PaddingAlgorithm, PaddingPKSC7},
	 	symmetric_algs::{DEAL128, DEALMode},
 	};
    use websocket::{ClientBuilder};
    use websocket::sync::Client;
    use s6_hcs_lib_transfer::{
        aux::*,
        key_exchange,
        messages::*,
    };
    use serde::{Deserialize, Serialize};
    use tauri::Manager;
    use crate::server_calls::Error::{BadFile, BadRequest, NoConnection, ServerError};
    use crate::server_calls::Progress::{Connecting, Decrypting, Done, Downloading, Encrypting, Uploading};


    #[derive(Serialize, Deserialize, Clone)]
    pub enum Error {
        NoConnection,
        BadRequest,
        ServerError,
        BadFile,
    }


    #[derive(Serialize, Deserialize, Clone)]
    pub enum Progress {
        Connecting,
        Encrypting(u8),
        Decrypting(u8),
        Uploading,
        Downloading,
        Done,
        Errored(Error),
    }


    fn err<T>(app: tauri::AppHandle, event: &str, e: Error) -> Result<T, Error>{
        let window = app.get_window("main").unwrap();
        window.emit(event, Progress::Errored(e.clone())).unwrap();
        Err(e)
    }

    fn connect(url: &str) -> Result<Client<TcpStream>, Error> {
        let client = match ClientBuilder::new(url) {
            Ok(client) => client,
            Err(_) => return Err(NoConnection),
        };
        let client = client
            .add_protocol("rust-websocket")
            .connect_insecure();
        match client {
            Ok(client) => Ok(client),
            Err(_) => return Err(NoConnection),
        }
    }

    #[tauri::command]
    pub async fn get_files(url: &str) -> Result<Vec<(String, String, String)>, Error> {
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return Err(e),
        };
        request(&mut client, Request::GetFiles);
        match deserialize(client.recv_message()) {
            Response::Success => {},
            Response::FSFail => return Err(ServerError),
            Response::CommFail => return Err(BadRequest),
        }
        let list: FileList = deserialize(client.recv_message());
        let list = list
            .iter()
            .map(|e| (
                e.clone().0.to_string(),
                e.clone().1.to_string(),
                e.clone().2
            ))
            .collect();
        Ok(list)
    }


    #[tauri::command]
    pub async fn upload(app: tauri::AppHandle, url: &str, file: &str, event: &str) -> Result<(), Error> {
        let window = app.get_window("main").unwrap();
        window.emit(event, Connecting).unwrap();

        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return err(app, event, e),
        };

        let path = PathBuf::from(file);
        let name = match path.clone().file_name() {
            None => return err(app, event, BadFile),
            Some(name) => name.to_owned().into_string(),
        };
        let name = match name {
            Err(_) => return err(app, event, BadFile),
            Ok(name) => name,
        };
        if !path.exists() {
            return err(app, event, BadFile)
        }

        let mut contents = match fs::read(path) {
            Ok(c) => c,
            Err(_) => return err(app, event, BadFile),
        };

        PaddingPKSC7::with_block_size(16).apply_padding(&mut contents);
        let contents = u8_to_u128(contents);
        let key = DEAL128::generate_key();
        let cb = |i| {
            window.emit(event, Encrypting(i)).unwrap();
        };
        window.emit(event, Encrypting(0)).unwrap();
        let enc = DEALMode::RDH.encrypt(contents, key, &cb);
        window.emit(event, Encrypting(100)).unwrap();

        window.emit(event, Uploading).unwrap();
        if let Err(_) = client.send_message(&serialize(Request::Upload)) {
            return err(app, event, NoConnection)
        }
        key_exchange::client_send(&mut client, key);
        if let Err(_) = client.send_message(&serialize(name)) {
            return err(app, event, NoConnection)
        }
        if let Err(_) = client.send_message(&serialize(&enc)) {
            return err(app, event, NoConnection)
        }
        if let Ok(msg) = client.recv_message() {
            return match deserialize(Ok(msg)) {
                Response::Success => { window.emit(event, Done).unwrap(); Ok(()) },
                Response::FSFail => err(app, event, ServerError),
                Response::CommFail => err(app, event, BadRequest),
            }
        } else {
            return err(app, event, NoConnection)
        }
    }



    #[tauri::command]
    pub async fn download(app: tauri::AppHandle, url: &str, id: &str, file: &str, event: &str) -> Result<(), Error> {
        let window = app.get_window("main").unwrap();
        window.emit(event, Connecting).unwrap();

        let id: u128 = match id.parse() {
            Err(_) => return Err(BadRequest),
            Ok(id) => id,
        };
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return Err(e),
        };

        window.emit(event, Downloading).unwrap();
        let path = PathBuf::from(file);
        if let Err(_) = client.send_message(&serialize(Request::Download(id))) {
            return err(app, event, NoConnection)
        }
        match deserialize(client.recv_message()) {
            Response::Success => { window.emit(event, Done).unwrap(); },
            Response::FSFail => return err(app, event, ServerError),
            Response::CommFail => return err(app, event, BadRequest),
        }

        let key = key_exchange::client_receive(&mut client);
        let contents: Vec<u128> = deserialize(client.recv_message());
        let cb = |i| {
            window.emit(event, Decrypting(i)).unwrap();
        };
        window.emit(event, Decrypting(0)).unwrap();
        let dec = match DEALMode::RDH.decrypt(contents, key, &cb) {
            Ok(dec) => dec,
            Err(_) => return err(app, event, BadFile),
        };

        let mut dec = u128_to_u8(dec);
        PaddingPKSC7::with_block_size(16).remove_padding(&mut dec);
        window.emit(event, Decrypting(100)).unwrap();

        match fs::write(path, dec) {
            Err(_) => Err(BadFile),
            Ok(_) => Ok(()),
        }
    }


    #[tauri::command]
    pub async fn delete(url: &str, id: &str) -> Result<(), Error> {
        let id: u128 = match id.parse() {
            Err(_) => return Err(BadFile),
            Ok(id) => id,
        };
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return Err(e),
        };
        request(&mut client, Request::Delete(id));
        match deserialize(client.recv_message()) {
            Response::Success => {},
            Response::FSFail => return Err(ServerError),
            Response::CommFail => return Err(BadRequest),
        }
        Ok(())
    }
}
