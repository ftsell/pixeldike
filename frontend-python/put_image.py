#!/usr/bin/env python3
import time

from PIL import Image
import argparse

from pixelflut_client import Client


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument("server_hostname")
    parser.add_argument("server_port")
    parser.add_argument("image")

    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()

    client = Client()
    client.connect(args.server_hostname, args.server_port)
    im = Image.open(args.image)
    im = im.resize((client.x_size, client.y_size))

    print(f"Uploading image [0/{client.x_size * client.y_size}]", end="")
    for ix in range(0, client.x_size):
        print(f"\rUploading image [{ix * client.y_size}/{client.x_size * client.y_size}]", end="")
        #time.sleep(1)
        for iy in range(0, client.y_size):
            r, g, b = im.getpixel((ix, iy))
            color = "%0.2X%0.2X%0.2X" % (r, g, b)
            client.set_pixel(ix, iy, color)

    print(f"\rUploading image [{client.x_size * client.y_size}/{client.x_size * client.y_size}]", end="")
