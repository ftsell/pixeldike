#!/usr/bin/env python3

from PIL import Image
import argparse
import time

from pixelflut_client import Client


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument("server_hostname")
    parser.add_argument("server_port")

    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()

    client = Client()
    client.connect(args.server_hostname, args.server_port)
    im = Image.new("RGB", (client.x_size, client.y_size))

    print("Receiving canvas", end="")
    start = time.time()
    pixels = client.receive_binary()
    end = time.time()
    print(f"     [{end - start}s]")

    print("Painting local canvas", end="")
    start = time.time()
    for ix in range(0, client.x_size):
        for iy in range(0, client.y_size):
            pixel_index = (ix * client.y_size + iy) * 3

            r = pixels[pixel_index]
            g = pixels[pixel_index + 1]
            b = pixels[pixel_index + 2]

            im.putpixel((ix, iy), (r, g, b))
    end = time.time()
    print(f"    [{end - start}s]")

    im.show()
