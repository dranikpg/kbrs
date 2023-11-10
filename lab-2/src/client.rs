use aes::{Aes256, BlockDecrypt, BlockEncrypt, NewBlockCipher};
use rand::Rng;
use rsa::{pkcs8::EncodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::common::AesWrapper;

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

    println!("Enter the name of the file you want to retrieve:");

    let filename = {
        let mut filename = String::new();
        io::stdin().read_line(&mut filename).unwrap();
        filename.trim().to_owned()
    };
    stream.write_all(filename.as_bytes()).await.unwrap();

    let mut encrypted_file_content = vec![];
    stream
        .read_to_end(&mut encrypted_file_content)
        .await
        .unwrap();

    let cipher = AesWrapper::new(&session_key);
    cipher.decrypt(&mut encrypted_file_content);

    println!(
        "Decrypted file content:\n\n====\n{}\n====\n\n",
        String::from_utf8_lossy(&encrypted_file_content)
    );
}
