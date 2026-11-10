# De HTTP-API van de NTA 8800-rekenkern (crates/nta8800-service, binary `api`) als container, voor de OpenAEC SuperCloud
# (de dienst `energie` per omgeving, zie docs/nta8800-api.md). Bouwen vanuit de root van de repo:
#   docker build -f deploy/cloud-api/Dockerfile --build-arg OES_BUILD_COMMIT=$(git rev-parse HEAD) .
# De container luistert op 0.0.0.0:8080 en vraagt een API-sleutel als OES_API_KEY_SHA256 gezet is (de SHA-256 van de sleutel).
FROM rust:1.99-bookworm AS bouw
ARG OES_BUILD_COMMIT=""
ENV OES_BUILD_COMMIT=${OES_BUILD_COMMIT}
WORKDIR /src
# De kern leest tijdens het bouwen ook docs/ en training-data/ (include_str!).
COPY crates crates
COPY docs docs
COPY training-data training-data
RUN cargo build --release --locked --manifest-path crates/nta8800-service/Cargo.toml --bin api \
 && install -m 755 crates/nta8800-service/target/release/api /usr/local/bin/oes-api

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --no-create-home oes
COPY --from=bouw /usr/local/bin/oes-api /usr/local/bin/oes-api
USER oes
ENV OES_API_BIND=0.0.0.0 OES_API_PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/oes-api"]
