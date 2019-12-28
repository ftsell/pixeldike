package network

import (
	"fmt"
	"github.com/gorilla/websocket"
	. "net"
	"net/http"
	"pixelflut/protocol"
	"sync"
)

var upgrader = websocket.Upgrader{
	Subprotocols: []string{"pixelflut"},
	CheckOrigin: func(_ *http.Request) bool {
		return true
	},
}

func handleWsConnection(conn *websocket.Conn, pixmap *protocol.Pixmap) {
	fmt.Printf("[WS] New connection from %v\n", conn.RemoteAddr())
	defer fmt.Printf("[WS] Connection from %v closed\n", conn.RemoteAddr())

	for {
		if messageType, messageBytes, err := conn.ReadMessage(); err != nil {
			fmt.Printf("[WS] Error receiving message: %v\n", err)
			_ = conn.Close()
			return
		} else {
			if messageType != websocket.TextMessage {
				_ = conn.WriteMessage(websocket.TextMessage, []byte("invalid message received. send a text message"))
			} else {
				message := string(messageBytes)
				response := protocol.ParseAndHandleInput(message, pixmap)
				_ = conn.WriteMessage(websocket.TextMessage, []byte(response))
			}
		}
	}
}

func handleHttpRequest(writer http.ResponseWriter, request *http.Request, pixmap *protocol.Pixmap) {
	if !websocket.IsWebSocketUpgrade(request) {
		writer.WriteHeader(http.StatusUpgradeRequired)
		_, _ = writer.Write([]byte("upgrade to websocket required"))
	} else {
		if conn, err := upgrader.Upgrade(writer, request, nil); err != nil {
			fmt.Printf("[WS] Error upgrading http connection to websocket: %v\n", err)
		} else {
			handleWsConnection(conn, pixmap)
		}
	}
}

func StartWebsocketServer(port string, pixmap *protocol.Pixmap, waitGroup *sync.WaitGroup) {
	defer waitGroup.Done()
	fmt.Printf("[WS] Starting websocket (http) server on port %v\n", port)

	http.HandleFunc("/", func(writer http.ResponseWriter, request *http.Request) {
		handleHttpRequest(writer, request, pixmap)
	})

	if err := http.ListenAndServe(JoinHostPort("", port), nil); err != nil {
		fmt.Printf("[WS] Cannot start http server: %v\n", err)
	}
}
