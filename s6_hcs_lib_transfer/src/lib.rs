pub mod key_exchange {
    use super::aux::*;
    use s6_hcs_lib_crypto::crypto::asymmetric_algs::XTR;

    use std::net::TcpStream;
    use websocket::sync::Client;


    pub fn client_send(client: &mut Client<TcpStream>, key: u128) {
        let xtr = XTR::new_at_client(deserialize(client.recv_message()));
        client.send_message(&serialize(xtr.share_trace_with_server())).unwrap();
        client.send_message(&serialize(xtr.encrypt_deal128_key(key))).unwrap();
    }

    pub fn server_receive(client: &mut Client<TcpStream>) -> u128 {
        let mut xtr = XTR::new_at_server();
        client
            .send_message(&serialize(xtr.share_public_key_with_client()))
            .unwrap();
        xtr.derive_sym_key_at_server(deserialize(client.recv_message()));
        xtr.encrypt_deal128_key(deserialize(client.recv_message()))
    }

    pub fn server_send(client: &mut Client<TcpStream>, key: u128) {
        let mut xtr = XTR::new_at_server();
        client.send_message(&serialize(xtr.share_public_key_with_client())).unwrap();
        xtr.derive_sym_key_at_server(deserialize(client.recv_message()));
        client.send_message(&serialize(xtr.encrypt_deal128_key(key))).unwrap();
    }

    pub fn client_receive(client: &mut Client<TcpStream>) -> u128 {
        let xtr = XTR::new_at_client(deserialize(client.recv_message()));
        client.send_message(&serialize(xtr.share_trace_with_server())).unwrap();
        xtr.encrypt_deal128_key(deserialize(client.recv_message()))
    }
}


pub mod messages {
    use super::aux::serialize;
    use std::net::TcpStream;
    use websocket::sync::Client;
    use serde::{Serialize, Deserialize};


    #[derive(Serialize, Deserialize)]
    pub enum Request {
        Upload,
        GetFiles,
        Download(u128),
        Delete(u128),
    }

    #[derive(Serialize, Deserialize)]
    pub enum Response {
        Success,
        FSFail,
        CommFail,
    }

    pub fn request(client: &mut Client<TcpStream>, req: Request) {
        client.send_message(&serialize(req)).unwrap();
    }

    pub fn respond(client: &mut Client<TcpStream>, res: Response) {
        client.send_message(&serialize(res)).unwrap();
    }
}


pub mod aux {
    use serde::{Serialize, Deserialize};
    use websocket::{
        OwnedMessage,
        WebSocketResult,
    };


    pub type FileList = Vec<(u128, usize, String)>;


    pub fn deserialize<T>(msg: WebSocketResult<OwnedMessage>) -> T
        where T: for<'a> Deserialize<'a>
    {
        let msg = msg.unwrap();
        if let OwnedMessage::Binary(data) = msg {
            serde_json::from_slice::<T>(&data).unwrap()
        } else {
            panic!()
        }
    }

    pub fn serialize<T>(data: T) -> OwnedMessage
        where T: Serialize
    {
        OwnedMessage::Binary(serde_json::to_vec(&data).unwrap())
    }

    pub fn u128_to_u8(value: Vec<u128>) -> Vec<u8> {
        value
            .iter()
            .flat_map(|num| num.to_be_bytes().to_vec())
            .collect()
    }

    pub fn u8_to_u128(value: Vec<u8>) -> Vec<u128> {
        value
            .chunks(16)
            .map(|chunk| {
                u128::from_be_bytes(chunk.try_into().unwrap())
            })
            .collect()
    }
}
