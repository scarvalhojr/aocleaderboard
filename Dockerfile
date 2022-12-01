FROM rust

RUN rustup default nightly && rustup update
RUN apt install libssl-dev pkg-config

WORKDIR /app
COPY Cargo.lock Cargo.toml ./
COPY src ./src
COPY templates ./templates
RUN cargo build --release

RUN chmod +x ./target/release/aocleaderboard
ENTRYPOINT [ "/app/target/release/aocleaderboard" ]
