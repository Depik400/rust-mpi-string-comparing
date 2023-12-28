FROM ubuntu:20.04

RUN apt-get update 
RUN apt install -y curl
RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh

WORKDIR /same_strings

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./config.toml ./config.toml

CMD [ "cargo", "install" ] 