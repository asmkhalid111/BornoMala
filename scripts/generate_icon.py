import zlib
import struct
import math

def make_png(width, height, pixels):
    def chunk(tag, data):
        return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)

    raw_data = bytearray()
    for y in range(height):
        raw_data.append(0) # filter byte 0 (None)
        for x in range(width):
            r, g, b, a = pixels[y * width + x]
            raw_data.extend([r, g, b, a])

    compressed = zlib.compress(bytes(raw_data), 9)
    ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr) + chunk(b'IDAT', compressed) + chunk(b'IEND', b'')
    return png

W, H = 256, 256
pixels = []

for y in range(H):
    for x in range(W):
        # Center coordinates normalized to [-1, 1]
        nx = (x - W / 2) / (W / 2)
        ny = (y - H / 2) / (H / 2)
        dist = math.sqrt(nx * nx + ny * ny)

        # Rounded badge (squircle)
        sq = (nx**4 + ny**4)**0.25
        if sq <= 0.88:
            # Gradient from deep indigo/blue (#1e1b4b) to vibrant emerald/cyan (#0ea5e9)
            t = (nx + ny + 2) / 4.0
            r = int(14 + t * (14 - 14))
            g = int(116 + t * (165 - 116))
            b = int(144 + t * (233 - 144))
            
            # Subtle keyboard / key cap shape in the middle
            kx = abs(nx)
            ky = abs(ny)
            if kx < 0.55 and ky < 0.45:
                # Key surface
                r = min(255, int(r * 1.35))
                g = min(255, int(g * 1.35))
                b = min(255, int(b * 1.35))
                
                # Letter stroke in the middle (stylized 'B' / Bengali letter shape)
                if (-0.25 <= nx <= -0.15 and -0.3 <= ny <= 0.3) or \
                   (-0.25 <= nx <= 0.2 and (-0.3 <= ny <= -0.22 or 0.22 <= ny <= 0.3 or -0.04 <= ny <= 0.04)) or \
                   (0.12 <= nx <= 0.22 and (-0.3 <= ny <= 0.3)):
                    r, g, b = 255, 255, 255

            a = 255
        else:
            r, g, b, a = 0, 0, 0, 0
            
        pixels.append((r, g, b, a))

png_bytes = make_png(W, H, pixels)
with open('app-icon.png', 'wb') as f:
    f.write(png_bytes)

print('Generated app-icon.png successfully')
