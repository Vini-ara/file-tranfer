use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use hex_literal::hex;
use std::io::Write;
use rand::Rng;

// Type alias for AES encryption
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// Encryption key (must be the same as used for encryption)
const KEY: &[u8; 16] = b"anexamplekey1234"; // 16 bytes key for AES-128

fn main() {
    // Generate a random IV (must be 16 bytes for AES-128)
    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);

    // Plain text data to be encrypted
    let plain_text = b"exampleplaintext"; // This must be a multiple of the block size (16 bytes)
    println!("Plain text: {:?}", plain_text);

    // Encrypt the data
    let cipher = Aes128Cbc::new_var(KEY, &iv).unwrap();
    let encrypted_data = cipher.encrypt_vec(plain_text);
    println!("Encrypted data: {:?}", encrypted_data);
    println!("IV: {:?}", iv);

    // Decrypt the data
    let cipher = Aes128Cbc::new_var(KEY, &iv).unwrap();
    let decrypted_data = cipher.decrypt_vec(&encrypted_data).unwrap();
    println!("Decrypted data: {:?}", decrypted_data);

    // Save to file or process the decrypted data
    let mut file = std::fs::File::create("received_file.txt").unwrap();
    file.write_all(&decrypted_data).unwrap();
}
