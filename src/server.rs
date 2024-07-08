use std::{
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::common::{
    deserialize_message, serialize_message, ClientMessage, FileStream, ServerMessage,
};

#[derive(Debug, Clone)]
pub struct Server {}

impl Server {
    pub async fn new() -> Self {
        Server {}
    }

    pub async fn listen(self) {
        let this = Arc::new(self);

        // cria servidor na porta 8080
        let listener = TcpListener::bind("localhost:8080").await.unwrap();

        // loop para aceitar conexões
        loop {
            // aceita conexão
            let (socket, addr) = listener.accept().await.unwrap();
            let addr = addr.to_string();

            let this = Arc::clone(&this);

            // cria thread para lidar com a conexão
            tokio::spawn(async move {
                this.handle_connection(socket, addr).await;
            });
        }
    }

    async fn handle_connection(&self, mut stream: TcpStream, addr: String) {
        let (sock_reader, mut writter) = stream.split();

        let mut reader = BufReader::new(sock_reader);

        let mut buffer = Vec::new();

        loop {
            reader.read_until(b'\n', &mut buffer).await.unwrap();

            if buffer.len() == 0 {
                break;
            }

            let message = deserialize_message(&buffer).unwrap();

            match message {
                ClientMessage::Hello => {
                    println!("Recebido hello de {}", addr);
                }
                ClientMessage::InitFileUpload { nome, tamanho } => {
                    println!("Recebido init file upload de {}, size {}", addr, tamanho);

                    let path_str = format!("./files/{}", nome);

                    let path = Path::new(&path_str);

                    let mut buffer = Vec::new();

                    let file = OpenOptions::new().append(true).create(true).open(path);

                    let file = file.unwrap();

                    println!(
                        "Criando arquivo e entrando no loop pra receber os dados {}",
                        path_str
                    );

                    loop {
                        reader.read_until(b'\n', &mut buffer).await.unwrap();

                        if buffer.len() == 0 {
                            break;
                        }

                        let message = deserialize_message(&buffer).unwrap();

                        match message {
                            ClientMessage::ContinueFileUpload(data) => {
                                println!("Recebido continue file upload de {}", addr);

                                let mut file = file.try_clone().unwrap();

                                file.write_all(&data).unwrap();
                            }
                            ClientMessage::FinalizeUpload => {
                                println!("Recebido finalize upload de {}", addr);
                                break;
                            }
                            _ => {
                                println!("Mensagem não reconhecida: {:?}", message);
                                break;
                            }
                        }

                        buffer.clear();
                    }

                    println!("Upload finalizado de {}", addr);
                }
                ClientMessage::RequestFileDownload { nome } => {
                    println!("Recebido request file download de {}", addr);

                    let path_str = format!("./files/{}", nome);

                    let file_length = fs::metadata(&path_str).unwrap().len();

                    let mut chunks = (file_length / 1024) + 1;

                    let message = ServerMessage::AcceptFileDownload {
                        tamanho: file_length,
                        chunks,
                    };

                    let message = serialize_message(message);

                    writter.write_all(message.as_bytes()).await.unwrap();

                    let file = fs::File::open(path_str).unwrap();

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

                        let message =
                            serialize_message(ServerMessage::ContinueFileDownload(buffer.clone()));

                        // manda o chunk para o cliente
                        writter.write_all(message.as_bytes()).await.unwrap();

                        println!("Enviando chunk de tamanho: {}", chunk_size);

                        chunks -= 1;
                        amount_read += chunk_size as u64;
                        // std::thread::sleep(std::time::Duration::from_secs(1));
                    }

                    println!("Finalizando Envio");

                    writter
                        .write_all(serialize_message(ServerMessage::FinalizeDownload).as_bytes())
                        .await
                        .unwrap();
                }
                ClientMessage::Disconnect => {}
                _ => {}
            }

            buffer.clear();
        }
    }
}
