FROM rust:1.78-slim-bookworm

WORKDIR /app
COPY . .

RUN cargo build --release --all
RUN cargo test --all

EXPOSE 9733 19733

CMD ["./target/release/lugsim"]
