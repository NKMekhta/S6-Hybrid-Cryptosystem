// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod helper;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            server_calls::get_files,
            server_calls::upload,
            server_calls::download,
            server_calls::delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod server_calls {
    use crate::helper::*;
    use s6_hcs_lib_crypto::crypto::{
        padding::{PaddingAlgorithm, PaddingPKSC7},
        symmetric_algs::{DEALMode, DEAL128},
    };
    use s6_hcs_lib_transfer::{aux::*, key_exchange, messages::*};
    use std::fs;
    use std::path::PathBuf;
    use tauri::Manager;

    use OperationProgress::*;
    use RequestProcessingError::*;
    use Response::*;

    #[tauri::command]
    pub async fn get_files(
        url: &str,
    ) -> Result<Vec<(String, String, String)>, RequestProcessingError> {
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return Err(e),
        };
        request(&mut client, Request::GetFiles);
        match deserialize(client.recv_message()) {
            Success => {}
            FSFail => return Err(ServerError),
            CommFail => return Err(BadRequest),
        }
        let list: FileList = deserialize(client.recv_message());
        let list = list
            .iter()
            .map(|e| {
                (
                    e.clone().0.to_string(),
                    e.clone().1.to_string(),
                    e.clone().2,
                )
            })
            .collect();
        Ok(list)
    }

    #[tauri::command]
    pub async fn upload(
        app: tauri::AppHandle,
        url: &str,
        file: &str,
        event: &str,
    ) -> Result<(), RequestProcessingError> {
        let window = app.get_window("main").unwrap();

        window.emit(event, Connecting).unwrap();
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return err(window, event, e),
        };

        let path = PathBuf::from(file);
        let name = match path.clone().file_name() {
            None => return err(window, event, BadFile),
            Some(name) => name.to_owned().into_string(),
        };
        let name = match name {
            Err(_) => return err(window, event, BadFile),
            Ok(name) => name,
        };
        if !path.exists() {
            return err(window, event, BadFile);
        }

        let mut contents = match fs::read(path) {
            Ok(c) => c,
            Err(_) => return err(window, event, BadFile),
        };

        PaddingPKSC7::with_block_size(16).apply_padding(&mut contents);
        let contents = u8_to_u128(contents);
        let key = DEAL128::generate_key();

        let output = {
            window.emit(event, Encrypting(0)).unwrap_or_default();
            let e = event.to_owned();
            let (tx, handle) = make_progress_reporter(
                contents.len(),
                Box::new(move |i| {
                    app.get_window("main")
                        .unwrap()
                        .emit(e.as_str(), Encrypting(i))
                        .unwrap_or_default()
                }),
            );
            let encrypted = DEALMode::RDH.encrypt(contents, key, tx.clone());
            tx.send(None).unwrap_or_default();
            handle.join().unwrap_or_default();
            window.emit(event, Encrypting(100)).unwrap_or_default();
            encrypted
        };

        window.emit(event, Uploading).unwrap();
        if let Err(_) = client.send_message(&serialize(Request::Upload)) {
            return err(window, event, NoConnection);
        }
        key_exchange::client_send(&mut client, key);
        if let Err(_) = client.send_message(&serialize(name)) {
            return err(window, event, NoConnection);
        }
        if let Err(_) = client.send_message(&serialize(&output)) {
            return err(window, event, NoConnection);
        }
        if let Ok(msg) = client.recv_message() {
            return match deserialize(Ok(msg)) {
                Success => {
                    window.emit(event, Done).unwrap();
                    Ok(())
                }
                FSFail => err(window, event, ServerError),
                CommFail => err(window, event, BadRequest),
            };
        } else {
            return err(window, event, NoConnection);
        }
    }

    #[tauri::command]
    pub async fn download(
        app: tauri::AppHandle,
        url: &str,
        id: &str,
        file: &str,
        event: &str,
    ) -> Result<(), RequestProcessingError> {
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
            return err(window, event, NoConnection);
        }
        match deserialize(client.recv_message()) {
            Success => {
                window.emit(event, Done).unwrap();
            }
            FSFail => return err(window, event, ServerError),
            CommFail => return err(window, event, BadRequest),
        }

        let key = key_exchange::client_receive(&mut client);
        let contents: Vec<u128> = deserialize(client.recv_message());

        let output = {
            window.emit(event, Decrypting(0)).unwrap_or_default();
            let e = event.to_owned();
            let (tx, handle) = make_progress_reporter(
                contents.len(),
                Box::new(move |i: u8| {
                    app.get_window("main")
                        .unwrap()
                        .emit(e.as_str(), Encrypting(i))
                        .unwrap_or_default()
                }),
            );
            let decrypted = match DEALMode::RDH.decrypt(contents, key, tx.clone()) {
                Ok(dec) => dec,
                Err(_) => return err(window, event, BadFile),
            };
            tx.send(None).unwrap_or_default();
            handle.join().unwrap_or_default();
            window.emit(event, Decrypting(100)).unwrap_or_default();
            decrypted
        };

        let mut dec = u128_to_u8(output);
        PaddingPKSC7::with_block_size(16).remove_padding(&mut dec);
        window.emit(event, Decrypting(100)).unwrap();

        match fs::write(path, dec) {
            Err(_) => Err(BadFile),
            Ok(_) => Ok(()),
        }
    }

    #[tauri::command]
    pub async fn delete(url: &str, id: &str) -> Result<(), RequestProcessingError> {
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
            Response::Success => {}
            Response::FSFail => return Err(ServerError),
            Response::CommFail => return Err(BadRequest),
        }
        Ok(())
    }
}
