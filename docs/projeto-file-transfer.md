---
title: File Transfer
parent: Projeto
has_children: false
nav_order: 2
---

# File Transfer

O file transfer é um projeto em rust para transferir arquivos entre dois clientes, o projeto se baseia em um servidor e dois tipos de cliente se comunicando com esse servidor, um cliente que envia um arquivo e outro que faz o download desse arquivo. A comunicação entre cliente e servidor é feita através de um protocolo TCP, e para fins de segurança os arquivos são salvos de forma criptografada no servidor utilizando criptografia XChaCha20.

## Compilando o projeto

Para compilar o projeto basta clonar esse repositório e rodar o comando ```cargo build --release```, assim um executável é gerado na pasta ```./target/release/```.

## Servidor

Para rodar o servidor é necessário primeiramente um banco de dados ```mongodb``` rodando, e um arquivo .env no mesmo diretório do executável contendo as variáveis ```MONGO_URL``` e ```DATABASE_NAME```. Feito isso basta rodar o executável da seguinte forma ```file-transfer server```.

## Enviando um arquivo

Para enviar um arquivo é muito simples, basta rodar o comando ```file-transfer send``` com a flag ```--file PATH_DO_ARQUIVO```, o comando completo para manda um arquivo ```exemplo.txt``` seria ```file-transfer send --file exemplo.txt```, caso obtenha sucesso no envio o usuário receberá uma mensagem contendo o código para realizar o download do arquivo.

## Baixando um arquivo

Para baixar um arquivo basta possuir um código (gerado ao enviar o arquvio). O comando para baixar ẽ ```file-transfer get``` junto com a flag ```--secret CODIGO_AQUI```. Com isso o arquivo deverá ser baixado no diretório de **Downloads** do usuário com o mesmo nome do arquivo original mandado.
