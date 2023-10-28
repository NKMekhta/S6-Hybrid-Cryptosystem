mod file_manager;

use file_manager::FileManager;
use s6_hcs_lib_transfer::{aux::*, file_exchange, key_exchange, messages::*};

use std::sync::Arc;
use websocket::sync::Server;
use log::{Level, log};

fn main() {
    use Request::*;
    use Response::*;

    let server = Server::bind("127.0.0.1:2794").unwrap();
    let mgr = Arc::new(FileManager::new("./test/storage").unwrap());

    for connection in server.filter_map(Result::ok) {
        let mgr = Arc::clone(&mgr);
        std::thread::spawn(move || {
            log!(Level::Info, "Client connected");
            let mut client = connection.accept().unwrap();
            match deserialize(client.recv_message()) {

                GetFiles => {
                    if let Ok(list) = mgr.get_file_list() {
                        respond(&mut client, Success);
                        log!(Level::Info, "Sending list of {}", list.len());
                        client.send_message(&serialize(list)).unwrap();
                    } else {
                        respond(&mut client, FSFail);
                    }
                }

                Upload => {
                    let key = key_exchange::server_receive(&mut client);
                    let name = deserialize(client.recv_message());
                    log!(Level::Info, "Receiving of {}", name);
                    let size = file_exchange::recv_file_len(&mut client).unwrap();
                    let contents = file_exchange::recv_file(&mut client, size, None).unwrap();
                    if let Ok(()) = mgr.save_file(name, key, contents) {
                        respond(&mut client, Success);
                    } else {
                        respond(&mut client, FSFail);
                    }
                }

                Download(id) => {
                    let (contents, key) = match mgr.get_file(id) {
                        Ok(data) => {
                            respond(&mut client, Success);
                            data
                        }
                        Err(_) => {
                            respond(&mut client, FSFail);
                            return;
                        }
                    };
                    key_exchange::server_send(&mut client, key);
                    file_exchange::send_file(&mut client, contents, None).unwrap();
                }

                Delete(id) => {
                    match mgr.delete_file(id) {
                        Ok(_) => respond(&mut client, Success),
                        Err(_) => respond(&mut client, FSFail),
                    };
                }

            };
        });
    }
}
