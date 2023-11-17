use core::slice::SlicePattern;

use serde::{Deserialize, Serialize};

use aes::{BlockDecrypt, BlockEncrypt, NewBlockCipher};

type Block = aes::cipher::generic_array::GenericArray<u8, aes::cipher::generic_array::typenum::U16>;

pub struct AesWrapper {
    aes: aes::Aes256,
}

impl AesWrapper {
    pub fn new(key: &[u8; 32]) -> Self {
        let aes_key = aes::cipher::generic_array::GenericArray::from(*key);
        AesWrapper {
            aes: aes::Aes256::new(&aes_key),
        }
    }

    pub fn encrypt(&self, data: &mut Vec<u8>) {
        data.extend(std::iter::repeat(0).take(16 - data.len() % 16));

        let mut block = Block::default();
        for chunk in data.chunks_exact_mut(16) {
            block.copy_from_slice(chunk.as_slice());
            self.aes.encrypt_block(&mut block);
            chunk.copy_from_slice(block.as_slice())
        }
    }

    pub fn decrypt(&self, data: &mut Vec<u8>) {
        let mut block = Block::default();
        for chunk in data.chunks_exact_mut(16) {
            block.copy_from_slice(chunk.as_slice());
            self.aes.decrypt_block(&mut block);
            chunk.copy_from_slice(block.as_slice());
        }
        while let Some(0) = data.last() {
            data.pop();
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { file: String, data: String },
    Get { file: String },
    Del { file: String },
    List {},
}

impl Command {
    pub fn parse(s: &str) -> Option<Command> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.get(0).map(|s| s.to_uppercase()).as_deref() {
            None => None,
            Some("SET") => parts.get(1).zip(parts.get(2)).map(|(k, v)| Command::Set {
                file: k.to_string(),
                data: v.to_string(),
            }),
            Some("GET") => parts.get(1).map(|f| Command::Get {
                file: f.to_string(),
            }),
            Some("DEL") => parts.get(1).map(|f| Command::Del {
                file: f.to_string(),
            }),
            Some("LIST") => Some(Command::List {}),
            _ => None,
        }
    }

    pub fn execute(&self) -> String {
        match self {
            Command::Set { file, data } => std::fs::write("./data/".to_owned() + &file, data)
                .map(|_| "OK".to_owned())
                .unwrap(),
            Command::Get { file } => std::fs::read_to_string("./data/".to_owned() + &file).unwrap(),
            Command::List {} => std::fs::read_dir("./data")
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .map(|n| format!("{:?}", n))
                .collect::<Vec<_>>()
                .join("\n"),
            Command::Del { file } => std::fs::remove_file("./data/".to_owned() + &file)
                .map(|_| "OK".to_owned())
                .unwrap(),
        }
    }
}
