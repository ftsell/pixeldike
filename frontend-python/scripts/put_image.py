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
    client.connect(args.server_hostname, int(args.server_port))
    im = Image.open(args.image)
    im = im.resize((client.size[0], client.size[1]))

    print(f"Uploading image [0/{client.size[0] * client.size[0]}]", end="")
    for ix in range(0, client.size[0]):
        print(f"\rUploading image [{ix * client.size[1]}/{client.size[0] * client.size[1]}]", end="")
        time.sleep(0.001)
        for iy in range(0, client.size[1]):
            r, g, b = im.getpixel((ix, iy))
            color = "%0.2X%0.2X%0.2X" % (r, g, b)
            client.set_pixel(ix, iy, color)

    print(f"\rUploading image [{client.size[0] * client.size[1]}/{client.size[0] * client.size[1]}]", end="")
