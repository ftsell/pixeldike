FROM docker.io/golang:latest as build

WORKDIR /app/src
ADD ./backend-go/ /app/src
RUN go mod download
RUN go mod verify
RUN GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -o /app/run github.com/ftsell/pixelflut/backend-go

FROM scratch as final
COPY --from=build /app/run /app/run

ENTRYPOINT ["/app/run"]
CMD "--tcp 9876 --udp 9876 --websocket 9875 -x 800 -y 600"
EXPOSE 9876
EXPOSE 9875

