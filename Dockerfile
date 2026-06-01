FROM rust:1-slim AS builder
WORKDIR /src
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /src/target/release/heol /heol
ENTRYPOINT ["/heol"]
CMD ["run"]
