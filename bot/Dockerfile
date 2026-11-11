# Kontekst: repo ildizi (git archive): Cargo.toml + crates/bitg-core + bot. deploy/deploy-bot.sh quradi.
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev gcc
WORKDIR /src
COPY Cargo.toml ./
COPY crates/bitg-core crates/bitg-core
COPY bot bot
WORKDIR /src/bot
RUN cargo build --release --locked && cp target/release/bitg-bot /bitg-bot

# Statik musl binary; TLS ildiz sertifikatlari ureq ichida (webpki-roots).
FROM scratch
COPY --from=build /bitg-bot /bitg-bot
USER 65534:65534
ENTRYPOINT ["/bitg-bot"]
