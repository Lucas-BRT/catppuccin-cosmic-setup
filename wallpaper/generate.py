#!/usr/bin/env python3
"""Gera o papel de parede Catppuccin Mocha (PNG 2560x1440).

Só usa a biblioteca padrão (zlib/struct), então roda em qualquer Python 3
sem instalar nada. O resultado é determinístico: rodar de novo gera
exatamente o mesmo arquivo.

Uso:
    python3 wallpaper/generate.py saida.png [largura altura]

Desenho:
  * fundo em degradê diagonal entre Crust (#11111b) e Base (#1e1e2e);
  * três brilhos suaves: Mauve embaixo à direita, Blue em cima à esquerda
    e Pink bem fraco na borda de baixo;
  * um pouco de ruído fixo para evitar faixas visíveis no degradê.
"""
import math
import struct
import sys
import zlib

CRUST = (17, 17, 27)  # #11111b
BASE = (30, 30, 46)   # #1e1e2e

# (centro x, centro y) em fração da tela, raio, cor, intensidade
GLOWS = [
    ((0.82, 0.85), 0.55, (203, 166, 247), 0.35),  # Mauve #cba6f7
    ((0.12, 0.10), 0.50, (137, 180, 250), 0.22),  # Blue  #89b4fa
    ((0.50, 1.10), 0.45, (245, 194, 231), 0.12),  # Pink  #f5c2e7
]


def pixel(x, y, w, h):
    fx, fy = x / w, y / h
    # Degradê: mais escuro (Crust) nas pontas da diagonal, Base no meio.
    t = fx * 0.4 + fy * 0.6
    c = [CRUST[i] + (BASE[i] - CRUST[i]) * (1 - abs(t - 0.5) * 2) ** 0.5 for i in range(3)]
    for (gx, gy), r, col, a in GLOWS:
        # 1.78 ≈ 16/9 deixa os brilhos redondos numa tela widescreen.
        d = math.hypot((fx - gx) * 1.78, fy - gy) / r
        if d < 1:
            k = a * (1 - d) ** 2.2
            for i in range(3):
                c[i] += (col[i] - c[i]) * k
    # Ruído fixo e barato (hash das coordenadas), entre -2 e +2.
    n = ((x * 73856093 ^ y * 19349663) & 7) - 3.5
    return bytes(max(0, min(255, int(v + n * 0.6))) for v in c)


def png(w, h):
    raw = b"".join(
        b"\x00" + b"".join(pixel(x, y, w, h) for x in range(w)) for y in range(h)
    )

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 6))
        + chunk(b"IEND", b"")
    )


if __name__ == "__main__":
    out = sys.argv[1]
    w, h = (int(sys.argv[2]), int(sys.argv[3])) if len(sys.argv) == 4 else (2560, 1440)
    with open(out, "wb") as f:
        f.write(png(w, h))
