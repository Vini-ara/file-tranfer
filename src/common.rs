use std::{fs::{self, File}, io::{Read, Write}, sync::MutexGuard};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::WriteHalf,
};

use orion::hazardous::{
    aead::xchacha20poly1305::{seal, open, Nonce, SecretKey},
    mac::poly1305::POLY1305_OUTSIZE,
    stream::xchacha20::XCHACHA_NONCESIZE,
};

use orion::hazardous::stream::chacha20::CHACHA_KEYSIZE;
use orion::kdf::{derive_key, Password, Salt};
use rand_core::{OsRng, RngCore};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub struct FileStream {}

impl FileStream {
    pub async fn upload_file(writer: &mut WriteHalf<'_>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // pega o tamanho do arquivo
        let file_length = fs::metadata(path).unwrap().len();

        // calcula a quantidade de chunks
        let mut chunks = (file_length / 1024) + 1;

        // abre o arquivo
        let file = fs::File::open(path).unwrap();

        // quantidade de bytes lidos
        let mut amount_read = 0;

        // loop para ler o arquivo
        while chunks > 0 {
            // clona o arquivo (coisa do rust)
            let mut file = file.try_clone().unwrap();

            // calcula o tamanho do chunk (1024 bytes ou o restante do arquivo)
            let chunk_size: usize = if file_length - amount_read > 1024 {
                1024
            } else {
                usize::try_from(file_length - amount_read).unwrap()
            };

            // cria buffer para chunk
            let mut buffer = vec![0; chunk_size];

            // lê o chunk
            file.read(&mut buffer).unwrap();

            let message = serialize_message(ClientMessage::ContinueFileUpload(buffer.clone()));

            // manda o chunk para o cliente
            writer.write_all(message.as_bytes()).await.unwrap();

            chunks -= 1;
            amount_read += chunk_size as u64;
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }

        writer.write_all(serialize_message(ClientMessage::FinalizeUpload).as_bytes()).await.unwrap();

        Ok(())
    }
    pub async fn download_file(writer: &mut MutexGuard<'_, &mut WriteHalf<'_>>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // pega o tamanho do arquivo
        let file_length = fs::metadata(path).unwrap().len();

        // calcula a quantidade de chunks
        let mut chunks = (file_length / 1024) + 1;

        // abre o arquivo
        let file = fs::File::open(path).unwrap();

        // quantidade de bytes lidos
        let mut amount_read = 0;

        // loop para ler o arquivo
        while chunks > 0 {
            // clona o arquivo (coisa do rust)
            let mut file = file.try_clone().unwrap();

            // calcula o tamanho do chunk (1024 bytes ou o restante do arquivo)
            let chunk_size: usize = if file_length - amount_read > 1024 {
                1024
            } else {
                usize::try_from(file_length - amount_read).unwrap()
            };

            // cria buffer para chunk
            let mut buffer = vec![0; chunk_size];

            // lê o chunk
            file.read(&mut buffer).unwrap();

            let message = serialize_message(ServerMessage::ContinueFileDownload(buffer.clone()));

            // manda o chunk para o cliente
            writer.write_all(message.as_bytes()).await.unwrap();

            chunks -= 1;
            amount_read += chunk_size as u64;
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }

        writer.write_all(serialize_message(ServerMessage::FinalizeDownload).as_bytes()).await.unwrap();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    // inicia conexão com o servidor
    Hello,

    RequestFileDownload { secret: String },

    RequestFileUpload { nome: String },

    // Inicia upload de arquivo 
    // parâmetros: nome do arquivo, tamanho do arquivo
    InitFileUpload { secret: String },

    // Continua upload de arquivo
    // parâmetros: chunk de arquivo
    ContinueFileUpload(Vec<u8>),

    // Continua dowload de arquivo
    // parâmetros: chunk de arquivo
    ContinueFileDownload(Vec<u8>),

    FinalizeUpload,

    Disconnect
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    AcceptFileDownload { nome: String, tamanho: u64, chunks: u64 },

    // Continua dowload de arquivo
    // parâmetros: chunk de arquivo
    ContinueFileDownload(Vec<u8>),

    FinalizeDownload,

    AcceptFileUpload { secret: String },

    Error(String),

    Disconnect
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileData {
    pub secret: String,
    pub fileName: String,
    pub path: String,
}

pub fn serialize_message<T: Serialize>(message: T) -> String {
    let delimiter = '\n';

    let message = serde_json::to_string(&message).unwrap();

    message + &delimiter.to_string()
}

pub fn deserialize_message<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    serde_json::from_slice(bytes).unwrap()
}

fn get_random(dest: &mut [u8]) {
    RngCore::fill_bytes(&mut OsRng, dest);
}

fn nonce() -> Vec<u8> {
    let mut randoms: [u8; 24] = [0; 24];
    get_random(&mut randoms);
    return randoms.to_vec();
}

fn auth_tag() -> Vec<u8> {
    let mut randoms: [u8; 32] = [0; 32];
    get_random(&mut randoms);
    return randoms.to_vec();
}

fn simple_split_encrypted(cipher_text: &[u8]) -> (Vec<u8>, Vec<u8>) {
    return (
        cipher_text[..CHACHA_KEYSIZE].to_vec(),
        cipher_text[CHACHA_KEYSIZE..].to_vec(),
    )
}

fn create_key(password: String, nonce: Vec<u8>) -> SecretKey {
    let password = Password::from_slice(password.as_bytes()).unwrap();
    let salt = Salt::from_slice(nonce.as_slice()).unwrap();
    let kdf_key = derive_key(&password, &salt, 15, 1024, CHACHA_KEYSIZE as u32).unwrap();
    let key = SecretKey::from_slice(kdf_key.unprotected_as_bytes()).unwrap();
    return key;
}

fn encrypt_core(
    dist: &mut File,
    contents: Vec<u8>,
    key: &SecretKey,
    nonce: Nonce,
) {
    let ad = auth_tag();
    let output_len = match contents.len().checked_add(POLY1305_OUTSIZE + ad.len()) {
        Some(min_output_len) => min_output_len,
        None => panic!("Plaintext is too long"),
    };

    let mut output = vec![0u8; output_len];
    output[..CHACHA_KEYSIZE].copy_from_slice(ad.as_ref());
    seal(&key, &nonce, contents.as_slice(), Some(ad.clone().as_slice()), &mut output[CHACHA_KEYSIZE..]).unwrap();
    dist.write(&output.as_slice()).unwrap();
}

fn decrypt_core(
    dist: &mut File,
    contents: Vec<u8>,
    key: &SecretKey,
    nonce: Nonce
) {
    let split = simple_split_encrypted(contents.as_slice());
    let mut output = vec![0u8; split.1.len() - POLY1305_OUTSIZE];

    open(&key, &nonce, split.1.as_slice(), Some(split.0.as_slice()), &mut output).unwrap();
    dist.write(&output.as_slice()).unwrap();
}


const CHUNK_SIZE: usize = 128; // The size of the chunks you wish to split the stream into.

pub fn encrypt_large_file(
    file_path: &str,
    output_path: &str,
    password: String
) -> Result<(), orion::errors::UnknownCryptoError> {
    let mut source_file = File::open(file_path).expect("Failed to open input file");
    let mut dist = File::create(output_path).expect("Failed to create output file");

    let mut src = Vec::new();
    source_file.read_to_end(&mut src).expect("Failed to read input file");

    let nonce = nonce();

    dist.write(nonce.as_slice()).unwrap();
    let key = create_key(password, nonce.clone());
    let nonce = Nonce::from_slice(nonce.as_slice()).unwrap();

    for (n_chunk, src_chunk) in src.chunks(CHUNK_SIZE).enumerate() {
        encrypt_core(&mut dist, src_chunk.to_vec(), &key, nonce)
    }

    Ok(())
}

pub fn decrypt_large_file(
    file_path: &str, 
    output_path: &str,
    password: String
) -> Result<(), orion::errors::UnknownCryptoError> {
    let mut input_file = File::open(file_path).expect("Failed to open input file");
    let mut output_file = File::create(output_path).expect("Failed to create output file");

    let mut src: Vec<u8> = Vec::new();
    input_file.read_to_end(&mut src).expect("Failed to read input file");

    let nonce = src[..XCHACHA_NONCESIZE].to_vec();

    src = src[XCHACHA_NONCESIZE..].to_vec();

    let key = create_key(password, nonce.clone());
    let nonce = Nonce::from_slice(nonce.as_slice()).unwrap();

    for (n_chunk, src_chunk) in src.chunks(CHUNK_SIZE + CHACHA_KEYSIZE + POLY1305_OUTSIZE).enumerate() {
        decrypt_core(&mut output_file, src_chunk.to_vec(), &key, nonce);
    }

    Ok(())
}
