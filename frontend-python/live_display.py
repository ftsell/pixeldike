#!/usr/bin/env python3

import argparse
import time
import threading

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("GdkPixbuf", "2.0")
from gi.repository import Gtk, GdkPixbuf, GLib, GObject

from pixelflut_client import Client


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument("server_hostname")
    parser.add_argument("server_port")

    return parser.parse_args()


def get_new_pixbuf():
    global client

    print("Receiving canvas", end="")
    start = time.time()
    pixels = client.receive_binary()
    end = time.time()
    print(f"     [{end - start}s]")

    print("Creating new pixbuf", end="")
    start = time.time()

    pixbuf = GdkPixbuf.Pixbuf.new_from_bytes(GLib.Bytes.new(pixels), GdkPixbuf.Colorspace.RGB, False, 8, client.x_size, client.y_size, client.x_size * 3)

    end = time.time()
    print(f"    [{end - start}s]")

    return pixbuf


def display_pixbuf(pixbuf):
    global image
    image.set_from_pixbuf(pixbuf)


def update():
    while True:
        pixbuf = get_new_pixbuf()
        GLib.idle_add(display_pixbuf, pixbuf)


if __name__ == "__main__":
    args = parse_args()

    client = Client()
    client.connect(args.server_hostname, args.server_port)

    window = Gtk.Window(title=f"Pixelflut remote canvas ({args.server_hostname}:{args.server_port})")
    window.set_default_size(client.x_size, client.y_size)
    window.connect("destroy", Gtk.main_quit)

    image = Gtk.Image.new()
    window.add(image)
    window.show_all()

    worker = threading.Thread(target=update)
    worker.daemon = True
    worker.start()

    Gtk.main()

    #update()
