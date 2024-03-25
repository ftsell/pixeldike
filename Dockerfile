FROM docker.io/rust:1.77 as build

RUN apt update && apt install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup toolchain install nightly --target=x86_64-unknown-linux-musl --profile=minimal

WORKDIR /usr/local/src/pixelflut/
ADD Cargo.toml Cargo.lock /usr/local/src/pixelflut/
RUN cargo fetch --locked
ADD . /usr/local/src/pixelflut/

ARG target_cpu=x86-64
RUN cargo build --offline --frozen --locked --target=x86_64-unknown-linux-musl --features=cli,tcp,udp,ws --release --bin=pixelflut


#
# final image
#
FROM docker.io/alpine as final
RUN apk add --no-cache tini ffmpeg
WORKDIR /app
RUN adduser -h /usr/local/src/pixelflut -s /bin/sh -D -u 10001 -g 10001 pixelflut

COPY --from=build /usr/local/src/pixelflut/target/x86_64-unknown-linux-musl/release/pixelflut /usr/local/bin/pixelflut

ENTRYPOINT ["/sbin/tini", "--", "pixelflut"]
CMD ["--help"]
