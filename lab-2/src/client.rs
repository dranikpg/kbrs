use aes::{Aes256, BlockDecrypt, BlockEncrypt, NewBlockCipher};
use rand::Rng;
use rsa::{pkcs8::EncodePublicKey, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const SERVER_ADDRESS: &str = "127.0.0.1:7878";

pub async fn run() {
    // Generate RSA keys
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let rsa_private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let rsa_public_key = RsaPublicKey::from(&rsa_private_key);

    // Connect to the server
    let mut stream = TcpStream::connect(SERVER_ADDRESS).await.unwrap();

    // Send the RSA public key to the server
    let public_key_pem = rsa_public_key
        .to_public_key_pem(Default::default())
        .unwrap();
    stream.write_all(&public_key_pem.as_bytes()).await.unwrap();

    println!("Written public key: {}", public_key_pem);

    // Receive the encrypted session key from the server
    let mut encrypted_session_key = vec![0u8; 256];
    stream.read_exact(&mut encrypted_session_key).await.unwrap();

    // Decrypt the session key
    let session_key = rsa_private_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_session_key)
        .unwrap();

    println!("Decrypted session key: {:?}", session_key);

    // Request a file from the server
    println!("Enter the name of the file you want to retrieve:");

    let mut filename = String::new();
    io::stdin().read_line(&mut filename).unwrap();
    let filename = filename.trim();

    // Send the filename to the server
    stream.write_all(filename.as_bytes()).await.unwrap();

    // Receive the encrypted file content from the server
    let mut encrypted_file_content = vec![];
    stream
        .read_to_end(&mut encrypted_file_content)
        .await
        .unwrap();

    let aes_key = aes::cipher::generic_array::GenericArray::<
        u8,
        aes::cipher::generic_array::typenum::U32,
    >::from_slice(&session_key);
    let cipher = aes::Aes256::new(&aes_key);

    let mut out = Vec::new();

    for block_slice in encrypted_file_content.chunks_exact(16) {
        let mut block = aes::cipher::generic_array::GenericArray::<
            u8,
            aes::cipher::generic_array::typenum::U16,
        >::from_iter(block_slice.iter().cloned());
        cipher.decrypt_block(&mut block);
        out.extend(block);
    }

    // Display the decrypted file content
    println!(
        "Decrypted file content:\n\n====\n{}\n====\n\n",
        String::from_utf8_lossy(&out)
    );
}
