FROM docker.io/rust:1.66 as build

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app/src
ADD . /app/src/
RUN cargo build --target x86_64-unknown-linux-musl --features cli --release --locked --bin pixelflut


#
# final image
#
FROM docker.io/alpine:3 as final
RUN apk add --no-cache tini
WORKDIR /app
RUN adduser -h /app -s /bin/sh -D -u 10001 -g 10001 pixelflut

COPY --from=build /app/src/target/x86_64-unknown-linux-musl/release/pixelflut /bin/pixelflut

ENV RUST_LOG=info
ENTRYPOINT ["/sbin/tini", "--", "/bin/pixelflut"]
CMD ["--help"]
