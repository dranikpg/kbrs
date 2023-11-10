use core::slice::SlicePattern;

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

    pub fn decrypt(&self, data: &mut [u8]) {
        let mut block = Block::default();
        for chunk in data.chunks_exact_mut(16) {
            block.copy_from_slice(chunk.as_slice());
            self.aes.decrypt_block(&mut block);
            chunk.copy_from_slice(block.as_slice());
        }
    }
}
