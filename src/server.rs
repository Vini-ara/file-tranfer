use std::{
    env,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{Read, Write},
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use rand::{distributions::Alphanumeric, Rng};
use dirs;
use mongodb::{
    bson::doc,
    Client, Collection,
};

use crate::common::{
    deserialize_message, encrypt_large_file, decrypt_large_file, serialize_message, ClientMessage, FileData, ServerMessage
};

#[derive(Debug, Clone)]
pub struct Server {}

impl Server {
    pub async fn new() -> Self {
        let file_transfer_dir = dirs::public_dir().unwrap().join("file_transfer");

        if !file_transfer_dir.exists() {
            fs::create_dir(&file_transfer_dir).unwrap();
        }

        let tmp_dir = file_transfer_dir.join("tmp");

        if !tmp_dir.exists() {
            fs::create_dir(&tmp_dir).unwrap();
        }

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

        let connection_string = env::var("MONGO_URL").unwrap();
        let db_name = env::var("DATABASE_NAME").unwrap();

        let client = Client::with_uri_str(connection_string.as_str())
            .await
            .unwrap();

        let database = client.database(db_name.as_str());

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
                ClientMessage::RequestFileUpload { nome } => {
                    println!("Recebido request file upload de {}", addr);

                    let secret: String = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(7)
                        .map(char::from)
                        .collect();

                    let collection = database.collection("files");

                    let is_secret_valid = collection
                        .find_one(doc! { "secret": &secret }, None)
                        .await
                        .unwrap();

                    if is_secret_valid != None {
                        println!("Erro ao criar arquivo");
                        let message = serialize_message(ServerMessage::Error(
                            "Erro ao criar arquivo".to_string(),
                        ));
                        writter.write_all(message.as_bytes()).await.unwrap();
                        break;
                    }

                    let files_dir = dirs::public_dir().unwrap().join("file_transfer").join(&secret);

                    collection
                        .insert_one(
                            doc! { "path": files_dir.to_str().unwrap(), "secret": &secret, "fileName": nome },
                            None,
                        )
                        .await
                        .unwrap();

                    let message = serialize_message(ServerMessage::AcceptFileUpload {
                        secret: secret.clone(),
                    });

                    writter.write_all(message.as_bytes()).await.unwrap();
                }
                ClientMessage::InitFileUpload { secret } => {
                    println!("Recebido init file upload de {}", addr);

                    let path = dirs::public_dir().unwrap().join("file_transfer").join("tmp").join(&secret);

                    let path_str = &path.to_str().unwrap();

                    let mut buffer = Vec::new();

                    println!(
                        "Criando arquivo e entrando no loop pra receber os dados {}",
                        &path_str
                    );

                    let file = OpenOptions::new().append(true).create(true).open(&path);

                    let file = file.unwrap();

                    loop {
                        reader.read_until(b'\n', &mut buffer).await.unwrap();

                        if buffer.len() == 0 {
                            break;
                        }

                        let message = deserialize_message(&buffer).unwrap();

                        match message {
                            ClientMessage::ContinueFileUpload(data) => {
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

                    let files_dir = dirs::public_dir().unwrap().join("file_transfer").join(&secret);

                    encrypt_large_file(&path_str, files_dir.to_str().unwrap(), secret).unwrap();

                    fs::remove_file(&path_str).unwrap();

                    println!("Upload finalizado de {}", addr);
                }
                ClientMessage::RequestFileDownload { secret } => {
                    println!("Recebido request file download de {}", addr);

                    let collection: Collection<FileData> = database.collection("files");

                    let file_data = collection
                        .find_one(doc! { "secret": &secret }, None)
                        .await
                        .unwrap();

                    if file_data.is_none() {
                        println!("Erro ao achar arquivo");
                        let message = serialize_message(ServerMessage::Error(
                            "Erro ao achar arquivo".to_string(),
                        ));
                        writter.write_all(message.as_bytes()).await.unwrap();
                        break;
                    }

                    let file_data = file_data.unwrap();

                    let path_str = dirs::public_dir().unwrap().join("file_transfer").join(&secret);

                    let tmp_dir = dirs::public_dir().unwrap().join("file_transfer").join("tmp").join(&secret);

                    decrypt_large_file(&path_str.to_str().unwrap(), &tmp_dir.to_str().unwrap(), secret).unwrap();

                    let file_length = fs::metadata(&tmp_dir).unwrap().len();

                    let mut chunks = (file_length / 1024) + 1;

                    let message = ServerMessage::AcceptFileDownload {
                        nome: file_data.fileName.clone(),
                        tamanho: file_length,
                        chunks,
                    };

                    let message = serialize_message(message);

                    writter.write_all(message.as_bytes()).await.unwrap();

                    let file = fs::File::open(&tmp_dir).unwrap();

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
                            serialize_message(ServerMessage::ContinueFileDownload(buffer));

                        writter.write_all(message.as_bytes()).await.unwrap();

                        chunks -= 1;
                        amount_read += chunk_size as u64;
                    }

                    fs::remove_file(tmp_dir).unwrap();

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
