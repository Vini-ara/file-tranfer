use std::{fs::{self, OpenOptions}, io::Write, path::Path};

use dirs;

use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpStream};

use crate::common::{deserialize_message, serialize_message, ClientMessage, FileStream, ServerMessage};

pub struct SendingClient {
    stream: TcpStream,
    file: String,
}

impl SendingClient {
    pub async fn new(file: String) -> Self {
        let stream = TcpStream::connect("localhost:8080").await.unwrap();

        SendingClient {
            stream,
            file,
        }
    }

    pub async fn connect(mut self) {
        let (reader, mut writer) = self.stream.split();

        // aqui que ficaria a logica de autenticação e troca de chaves
        let message = serialize_message(ClientMessage::Hello);

        writer.write(message.as_bytes()).await.unwrap();

        let message = serialize_message(ClientMessage::RequestFileUpload {
            nome: self.file.clone(),
        });

        writer.write(message.as_bytes()).await.unwrap();

        let mut reader = BufReader::new(reader);

        let mut buffer = Vec::new();

        loop {
            reader.read_until(b'\n', &mut buffer).await.unwrap();

            if buffer.len() == 0 {
                break;
            }

            let message = deserialize_message(&buffer).unwrap();

            match message {
                ServerMessage::AcceptFileUpload { secret } => {
                    let file_name = self.file.clone();

                    let file_length = fs::metadata(&file_name).unwrap().len();

                    println!("Enviando arquivo: {} de tamanho: {}", file_name, file_length);

                    let message = serialize_message(ClientMessage::InitFileUpload {
                        secret: secret.clone(),
                    });

                    writer.write(message.as_bytes()).await.unwrap();

                    FileStream::upload_file(&mut writer, &self.file).await.unwrap();

                    println!("Finalizando Envio");
                    println!("O arquivo pode ser recuperado pelo código {}", secret);

                    break;
                },
                ServerMessage::Error(mensagem) => {
                    println!("Erro: {}", mensagem);
                    break;
                },
                _ => {
                    println!("Mensagem inesperada: {:?}", message);
                }
            }

            buffer.clear();
        }
    }

    pub async fn send_file(mut self, file: &String) {
        let (_, mut writer) = self.stream.split();

        FileStream::upload_file(&mut writer, file).await.unwrap(); 
    }
}

pub struct FetchingClient {
    stream: TcpStream,
    secret: String,
}

impl FetchingClient {
    pub async fn new(secret: String) -> Self {
        let stream = TcpStream::connect("localhost:8080").await.unwrap();

        FetchingClient {
            stream,
            secret,
        }
    }

    pub async fn connect(mut self) {
        let (reader, mut writer) = self.stream.split();

        // iniciar logica de criptografia 
        let message = serialize_message(ClientMessage::Hello);

        writer.write(message.as_bytes()).await.unwrap();

        let message = serialize_message(ClientMessage::RequestFileDownload {
            secret: self.secret.clone(),
        });

        writer.write(message.as_bytes()).await.unwrap();

        let mut reader = BufReader::new(reader);

        let mut buffer = Vec::new();

        loop {
            reader.read_until(b'\n', &mut buffer).await.unwrap();

            if buffer.len() == 0 {
                break;
            }

            let message = deserialize_message(&buffer).unwrap();

            match message {
                ServerMessage::AcceptFileDownload { nome, tamanho, chunks } => {
                    let file_name = nome.clone();

                    let home = dirs::home_dir().unwrap();

                    let path_str = format!("{}/Downloads/{}", home.to_str().unwrap(), file_name);

                    let path = Path::new(path_str.as_str());

                    let file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .unwrap();
                    
                    let mut buffer = Vec::new();

                    loop {
                        reader.read_until(b'\n', &mut buffer).await.unwrap();

                        if buffer.len() == 0 {
                            break;
                        }

                        let message = deserialize_message(&buffer).unwrap();

                        match message {
                            ServerMessage::ContinueFileDownload(data) => {
                                let mut file = file.try_clone().unwrap();

                                file.write_all(&data).unwrap();
                            }
                            ServerMessage::FinalizeDownload => {
                                break;
                            },
                            ServerMessage::Error(mensagem) => {
                                println!("Erro: {}", mensagem);
                                break;
                            },
                            _ =>{ 
                                println!("Mensagem não reconhecida: {:?}", message);
                                break;
                            }
                        }

                        buffer.clear();
                    }

                    println!("Download finalizado, arquivo salvo em: {}", path_str);
                    break;
                },
                ServerMessage::FinalizeDownload => {
                    break;
                },
                ServerMessage::Error(mensagem) => {
                    println!("Erro: {}", mensagem);
                    break;
                },
                _ => {
                    println!("Mensagem inesperada: {:?}", message);
                }
            }

            buffer.clear();
        }

        println!("Download finalizado");
    }
}


