# Projeto: File Transfer

O file transfer é um projeto em rust para transferir arquivos entre dois clientes, o projeto se baseia em um servidor e dois tipos de cliente se comunicando com esse servidor, um cliente que envia um arquivo e outro que faz o download desse arquivo. A comunicação entre cliente e servidor é feita através de um protocolo TCP, e para fins de segurança os arquivos são salvos de forma criptografada no servidor utilizando criptografia XChaCha20.

Transferir arquivos é uma tarefa comum do dia a dia de milhares de pessoas, e muitas vezes recorremos por mandar algo por whatsapp, email, alguma plataforma de cloud storage ou então até mesmo por um pendrive. Mas cada plataforma dessa tem seus prós e contras, algumas trazem custo para armazenamento, outras são mais lentas e por aí vai.

A proposta do projeto é ser uma ferramenta de linha de comando útil para se transferir arquivos de forma simples. Com apenas um comando é possível enviar um arquivo para um servidor, o que gera um código que pode ser utilizado para resgatar e fazer o download desse arquivo.

## Servidor

Para rodar o servidor é necessário primeiramente um banco de dados ```mongodb``` rodando, e um arquivo .env no mesmo diretório do executável contendo as variáveis ```MONGO_URL``` e ```DATABASE_NAME```. Feito isso basta ter uma pasta com o nomes ```files``` na raiz do diretório e rodar o executável da seguinte forma ```file-transfer server```.

## Enviando um arquivo

Para enviar um arquivo é muito simples, basta rodar o comando ```file-transfer send``` com a flag ```--file PATH_DO_ARQUIVO```, o comando completo para manda um arquivo ```exemplo.txt``` seria ```file-transfer send --file exemplo.txt```, caso obtenha sucesso no envio o usuário receberá uma mensagem contendo o código para realizar o download do arquivo.

## Baixando um arquivo

Para baixar um arquivo basta possuir um código (gerado ao enviar o arquvio). O comando para baixar ẽ ```file-transfer get``` junto com a flag ```--secret CODIGO_AQUI```. Com isso o arquivo deverá ser baixado no diretório de **Downloads** do usuário com o mesmo nome do arquivo original mandado.

## Autores

### - Filipe Abadi Marcelino - 190087161
### - Tiago Leão Buson - 200034162
### - Vinícius da Silva Araujo - 221001981

## Links Importantes

### [Github do projeto](https://github.com/Vini-ara/file-tranfer)
### [Slides da Apresentação](https://docs.google.com/presentation/d/1JFkpZs8guHQA5i69HfbKa1Gs663aFrb7sgugSTw96yY/edit?usp=sharing)
### [Vídeo da Apresentação]()

