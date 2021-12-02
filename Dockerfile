FROM rust

RUN rustup default nightly && rustup update
RUN apt install libssl-dev pkg-config

# Make sure to have a settings.toml
COPY ./ ./
CMD cargo build --release

RUN chmod +x ./target/release/aocleaderboard
ENTRYPOINT [ "target/release/aocleaderboard" ]