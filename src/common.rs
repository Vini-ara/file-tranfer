use std::{fs, io::Read, sync::MutexGuard};
use tokio::{
    io::AsyncWriteExt,
    net::tcp::WriteHalf,
};

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

        println!("File length: {}", file_length);
        println!("Chunks: {}", chunks);

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

            println!("Enviando chunk de tamanho: {}", chunk_size);

            chunks -= 1;
            amount_read += chunk_size as u64;
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }

        println!("Finalizando upload");

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

        println!("File length: {}", file_length);
        println!("Chunks: {}", chunks);

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

            println!("Enviando chunk de tamanho: {}", chunk_size);

            chunks -= 1;
            amount_read += chunk_size as u64;
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }

        println!("Finalizando Envio");

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
