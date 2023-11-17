use rsa::{pkcs8::EncodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::common::AesWrapper;
use crate::common::Command;

const SERVER_ADDRESS: &str = "127.0.0.1:7878";

pub async fn run() {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let rsa_private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let rsa_public_key = RsaPublicKey::from(&rsa_private_key);

    let mut stream = TcpStream::connect(SERVER_ADDRESS).await.unwrap();

    let public_key_pem = rsa_public_key
        .to_public_key_pem(Default::default())
        .unwrap();
    stream.write_all(&public_key_pem.as_bytes()).await.unwrap();

    println!("Written public key: {}...", &public_key_pem[27..50]);

    let mut encrypted_session_key = vec![0u8; 256];
    stream.read_exact(&mut encrypted_session_key).await.unwrap();

    let session_key: [u8; 32] = rsa_private_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_session_key)
        .unwrap()
        .try_into()
        .unwrap();

    println!("Decrypted session key: {:?}", session_key);
    let cipher = AesWrapper::new(&session_key);

    loop {
        print!("> ");
        std::io::stdout().flush().ok();

        let cmd = {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            if let Some(cmd) = Command::parse(&buf) {
                cmd
            } else {
                continue;
            }
        };

        // Send cmd
        {
            let mut cmd_str: Vec<u8> = serde_json::to_string(&cmd).unwrap().into();
            cipher.encrypt(&mut cmd_str);

            stream.write_u32(cmd_str.len() as u32).await.unwrap();
            stream.write_all(&cmd_str).await.unwrap();
        }

        // Read response
        let resp = {
            let size = stream.read_u32().await.unwrap();
            let mut resp = vec![0u8; size as usize];
            stream.read_exact(&mut resp).await.unwrap();
            cipher.decrypt(&mut resp);
            resp
        };

        // Print response
        println!(
            "\n====\n{}\n====\n",
            String::from_utf8_lossy(&resp)
        );
    }
}
