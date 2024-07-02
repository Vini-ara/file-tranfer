use std::{env, fs, io::Read};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{tcp::WriteHalf, TcpListener}};
use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use rand::{rngs::OsRng, RngCore};
use std::convert::TryFrom; // Importa TryFrom para lidar com conversão de tipos

// Type alias for AES encryption
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

// Encryption key (must be kept secret)
const KEY: &[u8; 16] = b"anexamplekey1234"; // 16 bytes key for AES-128

#[tokio::main]
async fn main() {
    // cria servidor na porta 8080
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    // loop para aceitar conexões
    loop {
        // aceita conexão
        let (mut socket, _) = listener.accept().await.unwrap();

        // cria thread para lidar com a conexão
        tokio::spawn(async move {
            // separa leitura e escrita
            let (reader, mut writer) = socket.split();

            // cria buffer para leitura
            let mut reader = BufReader::new(reader);

            // cria string para armazenar a linha
            let mut line = String::new();

            loop {
                // lê linha
                let byte_read = reader.read_line(&mut line).await.unwrap();

                if byte_read == 0 {
                    break;
                }

                // pega o diretório atual
                let mut current_path = env::current_dir().unwrap();

                // adiciona o caminho da requisição
                current_path.push(&line.trim());

                println!("Request: {}", current_path.display());

                // faz o stream do arquivo criptografado
                file_stream(&mut writer, current_path.to_str().unwrap()).await;
            };
        });
    }
}

async fn file_stream(writer: &mut WriteHalf<'_>, path: &str) {
    // gera um IV aleatório para cada conexão
    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    // envia o IV para o cliente
    let _ = writer.write_all(&iv).await.unwrap();

    // pega o tamanho do arquivo
    let file_length = fs::metadata(path).unwrap().len();

    // calcula a quantidade de chunks
    let mut chunks = (file_length / 1024) + 1;

    // abre o arquivo
    let mut file = fs::File::open(path).unwrap();

    println!("File length: {}", file_length);
    println!("Chunks: {}", chunks);

    // quantidade de bytes lidos
    let mut amount_read = 0;

    // loop para ler o arquivo
    while chunks > 0 {
        // calcula o tamanho do chunk (1024 bytes ou o restante do arquivo)
        let chunk_size: usize = if file_length - amount_read > 1024 {
            1024
        } else {
            usize::try_from(file_length - amount_read).unwrap() // Convertendo para usize
        };

        // cria buffer para chunk
        let mut buffer = vec![0; chunk_size];

        // lê o chunk
        file.read_exact(&mut buffer).unwrap();

        // criptografa o chunk
        let cipher = Aes128Cbc::new_var(KEY, &iv).unwrap(); // Criando o cifrador com new_var
        let ciphertext = cipher.encrypt_vec(&buffer);

        println!("Buffer: {:?}", ciphertext);

        // manda o chunk criptografado para o cliente
        let _ = writer.write_all(&ciphertext).await.unwrap();

        chunks -= 1;
        amount_read += chunk_size as u64;
    }
}
