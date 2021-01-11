FROM docker.io/rust:1.49 as build

RUN apt update && apt install -y gettext musl-tools
RUN rustup install nightly
RUN rustup target add x86_64-unknown-linux-musl --toolchain=nightly

WORKDIR /app/src
ADD . /app/src/
RUN cargo +nightly build --target=x86_64-unknown-linux-musl --all-features --release --bin pixelflut_server


#
#
# final image
#
#
FROM docker.io/alpine:3.12 as final
RUN apk add --no-cache tini
WORKDIR /app
RUN adduser -h /app -s /bin/sh -D -u 10001 -g 10001 pixelflut

COPY --from=build /app/src/target/x86_64-unknown-linux-musl/release/pixelflut_server /bin/pixelflut_server

ENV RUST_LOG=info
ENTRYPOINT ["/sbin/tini", "--", "/bin/pixelflut_server"]
CMD ["--help"]
EXPOSE 1234/udp
EXPOSE 1234/tcp
