use std::{
    fs,
    env,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);

            let mut line = String::new();

            loop {
                let byte_read = reader.read_line(&mut line).await.unwrap();

                if byte_read == 0 {
                    break;
                }

                let mut current_path = env::current_dir().unwrap();

                current_path.push(&line.trim());

                println!("Request: {}", current_path.display());

                let content = fs::read_to_string(&current_path).map_or_else(|_| "File not found".to_string(), |content| content);

                writer.write_all(content.as_bytes()).await.unwrap();
                line.clear();
            };
        });
    }
}
