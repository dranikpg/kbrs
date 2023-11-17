use rand::{thread_rng, Rng};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::common::AesWrapper;
use crate::common::Command;

const ADDRESS: &str = "127.0.0.1:7878";

pub struct Server {}

impl Server {
    pub async fn run(&self) {
        let mut rng = thread_rng();
        let bits = 2048;
        let rsa_private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();

        let listener = TcpListener::bind(ADDRESS).await.unwrap();
        println!("Server listening on {}", ADDRESS);

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(Self::handle_client(stream, rsa_private_key.clone()));
        }
    }

    async fn handle_client(mut stream: TcpStream, _: RsaPrivateKey) {
        let mut buffer = [0; 1024];

        println!("New client {:?}", stream.peer_addr());

        let client_public_key = {
            let bytes_read = stream.read(&mut buffer).await.unwrap();
            let public_key_slice = std::str::from_utf8(&buffer[..bytes_read]).unwrap();

            println!("Got public key: {}...", &public_key_slice[27..50]);
            RsaPublicKey::from_public_key_pem(public_key_slice).unwrap()
        };

        let session_key: [u8; 32] = thread_rng().gen(); // AES 256 bit
        println!("Generated session key: {:?}", session_key);

        let encrypted_session_key = client_public_key
            .encrypt(&mut thread_rng(), Pkcs1v15Encrypt, &session_key)
            .unwrap();
        stream.write_all(&encrypted_session_key).await.unwrap();

        let cipher = AesWrapper::new(&session_key);

        loop {
            // Read command
            let cmd: Command = {
                let size = stream.read_u32().await.unwrap();
                let mut cmd_buf = vec![0u8; size as usize];
                stream.read_exact(&mut cmd_buf).await.unwrap();

                cipher.decrypt(&mut cmd_buf);

                let cmd_str = String::from_utf8_lossy(&cmd_buf);
                serde_json::from_str(&cmd_str).unwrap()
            };
            println!("Got: {:?}", cmd);

            // Execute command
            let mut resp = cmd.execute().into_bytes();
            cipher.encrypt(&mut resp);
            
            stream.write_u32(resp.len() as u32).await.unwrap();
            stream.write_all(&resp).await.unwrap();
        }
    }
}
