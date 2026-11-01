"""Generate the catalog's sample pictures (avatar.png, landscape.png).

Both pictures are drawn from scratch by this script, so they carry no third-party
license. Run `python generate.py` in this directory to rewrite them.
"""

import math
import struct
import zlib


def write_png(path, width, height, pixel):
    """Write an RGB PNG whose color at (x, y) is `pixel(x, y)`."""
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        for x in range(width):
            rows.extend(pixel(x, y))
    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    with open(path, "wb") as out:
        out.write(b"\x89PNG\r\n\x1a\n")
        out.write(chunk(b"IHDR", header))
        out.write(chunk(b"IDAT", zlib.compress(bytes(rows), 9)))
        out.write(chunk(b"IEND", b""))


def mix(a, b, t):
    t = max(0.0, min(1.0, t))
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def avatar(x, y, size=128):
    """A neutral portrait: a gradient backdrop, a head and shoulders."""
    u, v = x / (size - 1), y / (size - 1)
    color = mix((226, 226, 230), (170, 174, 186), v * 0.8 + u * 0.2)
    head = math.hypot(x - size * 0.5, y - size * 0.40) < size * 0.18
    shoulders = ((x - size * 0.5) / (size * 0.40)) ** 2 + ((y - size * 1.02) / (size * 0.42)) ** 2 < 1
    if head or shoulders:
        color = mix((92, 98, 112), (60, 64, 76), v)
    return color


def landscape(x, y, width=640, height=360):
    """A flat landscape: a sky, a sun and three layered hills."""
    u, v = x / (width - 1), y / (height - 1)
    color = mix((250, 226, 196), (150, 184, 214), v * 1.4)
    if math.hypot(x - width * 0.72, y - height * 0.30) < height * 0.11:
        color = (255, 244, 214)
    layers = [
        (0.62, 0.10, 2.0, 0.0, (120, 150, 132)),
        (0.72, 0.09, 3.0, 1.3, (84, 118, 100)),
        (0.84, 0.07, 4.0, 2.1, (52, 84, 72)),
    ]
    for base, amp, freq, phase, tint in layers:
        ridge = height * (base + amp * math.sin(u * math.pi * freq + phase))
        if y >= ridge:
            color = tint
    return color


write_png("avatar.png", 128, 128, avatar)
write_png("landscape.png", 640, 360, landscape)
