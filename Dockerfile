FROM rust:1.67

WORKDIR /usr/src/langkeeper
COPY . .

RUN cargo install --path .
RUN cargo build --release

COPY .env /usr/src/langkeeper/.env

RUN apt-get update -y
RUN apt-get install -y sqlite3 libsqlite3-dev libssl-dev
RUN cargo install diesel_cli --no-default-features --features sqlite
RUN diesel migration run

CMD ["/usr/src/langkeeper/target/release/langkeeper", "serve"]
