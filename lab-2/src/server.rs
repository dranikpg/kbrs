use aes::{Aes256, BlockEncrypt, NewBlockCipher};
use rand::{thread_rng, Rng};
use rsa::{Pkcs1v15Encrypt, pkcs8::DecodePublicKey, RsaPrivateKey, RsaPublicKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::common::AesWrapper;

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

    async fn handle_client(mut stream: TcpStream, rsa_private_key: RsaPrivateKey) {
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

        let filename = {
            let bytes_read = stream.read(&mut buffer).await.unwrap();
            String::from_utf8_lossy(&buffer[..bytes_read])
        };
        println!("Requested filename: {}", filename);

        let mut file_content = std::fs::read(filename.to_string()).unwrap();
        let cipher = AesWrapper::new(&session_key);
        cipher.encrypt(&mut file_content);

        stream.write_all(&file_content).await.ok();
    }
}
