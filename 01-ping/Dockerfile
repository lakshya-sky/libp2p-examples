FROM rust:slim
COPY . /usr/src/app
WORKDIR /usr/src/app
RUN cargo build --release
EXPOSE 3000
VOLUME ["/usr/local/cargo"]
ENTRYPOINT ["target/release/ping_example"]
