use aes::{Aes256, BlockEncrypt, NewBlockCipher};
use rand::{thread_rng, Rng};
use rsa::{Pkcs1v15Encrypt, pkcs8::DecodePublicKey, RsaPrivateKey, RsaPublicKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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

        // Receive the RSA public key from the client
        let bytes_read = stream.read(&mut buffer).await.unwrap();
        let public_key_slice = std::str::from_utf8(&buffer[..bytes_read]).unwrap();

        println!("Got public key: {}", public_key_slice);

        let client_public_key = RsaPublicKey::from_public_key_pem(public_key_slice).unwrap();

        // Generate a random session key
        let session_key: [u8; 32] = thread_rng().gen(); // AES 256-bit key

        println!("Generated session key: {:?}", session_key);

        // Encrypt the session key with the client's RSA public key
        let encrypted_session_key = client_public_key
            .encrypt(&mut thread_rng(), Pkcs1v15Encrypt, &session_key)
            .unwrap();

        // Send the encrypted session key to the client
        stream.write_all(&encrypted_session_key).await.unwrap();

        // Wait for the client to send a filename
        let bytes_read = stream.read(&mut buffer).await.unwrap();
        let filename = String::from_utf8_lossy(&buffer[..bytes_read]);

        println!("Requested filename: {}", filename);

        // Read the file content
        let mut file_content = std::fs::read(filename.to_string()).unwrap();

        println!("File content: {:?}", String::from_utf8_lossy(&file_content));

        // Encrypt the file content with the session key
        let aes_key = aes::cipher::generic_array::GenericArray::from(session_key);
        let cipher = aes::Aes256::new(&aes_key);

        file_content.extend(std::iter::repeat(0).take(16 - file_content.len() % 16));

        for chunk in file_content.chunks_exact(16) {
            let mut block = aes::cipher::generic_array::GenericArray::<
                u8,
                aes::cipher::generic_array::typenum::U16,
            >::from_iter(chunk.iter().cloned());
            cipher.encrypt_block(&mut block);
            stream.write_all(&block).await.unwrap();
        }
    }
}
