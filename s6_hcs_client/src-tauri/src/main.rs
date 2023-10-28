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
    use s6_hcs_lib_transfer::{aux::*, key_exchange, messages::*, file_exchange};
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

        window.emit(event, Connecting(0)).unwrap();
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return Err(e),
        };

        let file_path = PathBuf::from(file);
        let file_name = match file_path.clone().file_name() {
            None => return Err(BadFile),
            Some(n) => match n.to_owned().into_string() {
                Err(_) => return Err(BadFile),
                Ok(name) => name,
            },
        };
        if !file_path.exists() {
            return Err(BadFile);
        }
        let mut contents_dec = match fs::read(file_path) {
            Ok(c) => c,
            Err(_) => return Err(BadFile),
        };

        PaddingPKSC7::with_block_size(16).apply_padding(&mut contents_dec);
        let contents_dec = u8_to_u128(contents_dec);
        let key = DEAL128::generate_key();

        let contents_enc = {
            window.emit(event, Encrypting(0)).unwrap_or_default();
            let w = window.clone();
            let e = event.to_owned();
            let (tx, handle) = make_progress_reporter(
                contents_dec.len(),
                Box::new(move |i| {
                    w.emit(e.as_str(), Encrypting(i)).unwrap_or_default()
                }),
            );
            let contents_enc = DEALMode::RDH.encrypt(contents_dec, key, tx.clone());
            tx.send(None).unwrap_or_default();
            handle.join().unwrap_or_default();
            window.emit(event, Encrypting(100)).unwrap_or_default();
            contents_enc
        };

        if let Err(_) = client.send_message(&serialize(Request::Upload)) {
            return Err(NoConnection);
        }
        key_exchange::client_send(&mut client, key);
        if let Err(_) = client.send_message(&serialize(file_name)) {
            return Err(NoConnection);
        }

        {
            let contents_enc = u128_to_u8(contents_enc);
            window.emit(event, Uploading(0)).unwrap_or_default();
            let w = window.clone();
            let e = event.to_owned();
            let (tx, handle) = make_progress_reporter(
                file_exchange::count_dataframes(&contents_enc),
                Box::new(move |i| {
                    w.emit(e.as_str(), Uploading(i)).unwrap_or_default()
                }),
            );
            if let Err(_) = file_exchange::send_file(
                &mut client,
                contents_enc,
                Some(tx.clone()),
            ) {
                return Err(NoConnection);
            }
            tx.send(None).unwrap_or_default();
            handle.join().unwrap_or_default();
            window.emit(event, Uploading(100)).unwrap_or_default();
        }

        if let Ok(msg) = client.recv_message() {
            match deserialize(Ok(msg)) {
                Success => Ok(()),
                FSFail => Err(ServerError),
                CommFail => Err(BadRequest),
            }
        } else {
            Err(NoConnection)
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

        window.emit(event, Connecting(0)).unwrap();
        let id: u128 = match id.parse() {
            Err(_) => return Err(BadRequest),
            Ok(id) => id,
        };
        let mut client = match connect(url) {
            Ok(client) => client,
            Err(e) => return Err(e),
        };

        let path = PathBuf::from(file);
        if let Err(_) = client.send_message(&serialize(Request::Download(id))) {
            return Err(NoConnection);
        }
        match deserialize(client.recv_message()) {
            Success => {}
            FSFail => return Err(ServerError),
            CommFail => return Err(BadRequest),
        }

        let key = key_exchange::client_receive(&mut client);

        let contents_enc = {
            window.emit(event, Downloading(0)).unwrap_or_default();
            let e = event.to_owned();
            let w = window.clone();
            let size = match file_exchange::recv_file_len(&mut client) {
                Ok(s) => s,
                Err(_) => return Err(NoConnection),
            };
            let (tx, handle) = make_progress_reporter(
                size,
                Box::new(move |i| {
                    w.emit(e.as_str(), Downloading(i)).unwrap_or_default()
                }),
            );
            let contents = match file_exchange::recv_file(
                &mut client,
                size,
                Some(tx.clone())
            ) {
                Ok(c) => c,
                Err(_) => return Err(NoConnection),
            };
            tx.send(None).unwrap_or_default();
            handle.join().unwrap_or_default();
            window.emit(event, Downloading(100)).unwrap_or_default();
            u8_to_u128(contents)
        };

        let contents_dec = {
            window.emit(event, Decrypting(0)).unwrap_or_default();
            let e = event.to_owned();
            let w = window.clone();
            let (tx, handle) = make_progress_reporter(
                contents_enc.len(),
                Box::new(move |i: u8| {
                    w.emit(e.as_str(), Decrypting(i)).unwrap_or_default()
                }),
            );
            let decrypted = match DEALMode::RDH.decrypt(contents_enc, key, tx.clone()) {
                Ok(dec) => dec,
                Err(_) => return Err(BadFile),
            };
            tx.send(None).unwrap_or_default();
            handle.join().unwrap_or_default();
            window.emit(event, Decrypting(100)).unwrap_or_default();
            decrypted
        };

        let mut dec = u128_to_u8(contents_dec);
        PaddingPKSC7::with_block_size(16).remove_padding(&mut dec);

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
            Success => Ok(()),
            FSFail => Err(ServerError),
            CommFail => Err(BadRequest),
        }
    }
}
