#!/usr/bin/env python3

from PIL import Image
import argparse

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

    print(f"Receiving image [0/{client.x_size * client.y_size}]", end="")
    for ix in range(0, client.x_size):
        print(f"\rReceiving image [{ix * client.y_size}/{client.x_size * client.y_size}]", end="")
        for iy in range(0, client.y_size):
            hex_color = client.get_pixel(ix, iy)
            r = int(f"0x{hex_color[0:2]}", 0)
            g = int(f"0x{hex_color[2:4]}", 0)
            b = int(f"0x{hex_color[4:6]}", 0)

            im.putpixel((ix, iy), (r, g, b))

    print(f"\rReceiving image [{client.x_size * client.y_size}/{client.x_size * client.y_size}]", end="")
    im.show()
