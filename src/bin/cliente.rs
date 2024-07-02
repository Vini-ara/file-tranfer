use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use hex_literal::hex;
use std::io::Write;
use rand::Rng;

// Tipo alias para criptografia AES
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// Chave de criptografia
const KEY: &[u8; 16] = b"anexamplekey1234"; // A chave deve ser de 16 bytes para AES-128

fn main() {
    // Gera um IV aleatorio
    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);

    // Plain text data to be encrypted
    let plain_text = b"exampleplaintext"; // This must be a multiple of the block size (16 bytes)
    println!("Plain text: {:?}", plain_text);

    // Criptografa os dados
    let cipher = Aes128Cbc::new_var(KEY, &iv).unwrap();
    let encrypted_data = cipher.encrypt_vec(plain_text);
    println!("Encrypted data: {:?}", encrypted_data);
    println!("IV: {:?}", iv);

    // Descriptografa os dados
    let cipher = Aes128Cbc::new_var(KEY, &iv).unwrap();
    let decrypted_data = cipher.decrypt_vec(&encrypted_data).unwrap();
    println!("Decrypted data: {:?}", decrypted_data);

    // Salva os dados descriptografados
    let mut file = std::fs::File::create("received_file.txt").unwrap();
    file.write_all(&decrypted_data).unwrap();
}
