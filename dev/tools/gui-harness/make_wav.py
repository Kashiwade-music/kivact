# Write a stereo PCM16 WAV: 440 Hz left, 880 Hz right, 44.1 kHz, argv[2] seconds.
import math, struct, sys
path = sys.argv[1]
seconds = float(sys.argv[2]) if len(sys.argv) > 2 else 3.0
rate = 44100
frames = int(rate * seconds)
data = bytearray()
for i in range(frames):
    t = i / rate
    l = int(0.4 * 32767 * math.sin(2 * math.pi * 440 * t))
    r = int(0.4 * 32767 * math.sin(2 * math.pi * 880 * t))
    data += struct.pack("<hh", l, r)
with open(path, "wb") as f:
    f.write(b"RIFF" + struct.pack("<I", 36 + len(data)) + b"WAVEfmt ")
    f.write(struct.pack("<IHHIIHH", 16, 1, 2, rate, rate * 4, 4, 16))
    f.write(b"data" + struct.pack("<I", len(data)) + bytes(data))
print(path, frames)
