"""DAT/CCL -> JSON-Konverter fuer CIRCUIT RUNNER.

Liest das originale Chip's-Challenge-Bin-Levelformat (.dat/.ccl, wie es
Tile World / CCEdit nutzen, das auf Fansites verbreitet ist) und schreibt
ein selbstbeschreibendes JSON-Set, das die Engine direkt laedt.

    py circuitrunner/convert_dat.py  pfad/zu/CHIPS.dat  [ausgabe.json]

Format-Referenz: www.seasip.info/ccfile.html . Die Tile-Codes (0x00..0x6F)
sind in der Engine 1:1 die Zellen-Indizes im Tileset (siehe make_tiles.py).

JSON-Set-Schema:
    { "name": "...", "ruleset": "ms|lynx",
      "levels": [ {
          "title","number","time","chips","hint","password",
          "width"=32,"height"=32,
          "upper": "<2048 Hex-Zeichen>", "lower": "<2048 Hex-Zeichen>",
          "traps":   "bx,by,tx,ty;...",
          "cloners": "bx,by,mx,my;...",
          "monsters":"x,y;..." } ] }
"""
from __future__ import annotations

import json
import struct
import sys
from pathlib import Path

W = H = 32
N = W * H  # 1024


class Reader:
    def __init__(self, data: bytes):
        self.d = data
        self.i = 0

    def u8(self) -> int:
        v = self.d[self.i]
        self.i += 1
        return v

    def u16(self) -> int:
        v = struct.unpack_from("<H", self.d, self.i)[0]
        self.i += 2
        return v

    def u32(self) -> int:
        v = struct.unpack_from("<I", self.d, self.i)[0]
        self.i += 4
        return v

    def bytes(self, n: int) -> bytes:
        v = self.d[self.i:self.i + n]
        self.i += n
        return v


def rle_decode(raw: bytes) -> list[int]:
    """RLE -> 1024 Tile-Codes. 0xFF,count,tile = count Wiederholungen."""
    out: list[int] = []
    i = 0
    while i < len(raw) and len(out) < N:
        b = raw[i]
        if b == 0xFF:
            count = raw[i + 1]
            tile = raw[i + 2]
            out.extend([tile] * count)
            i += 3
        else:
            out.append(b)
            i += 1
    # auf 1024 auffuellen / kappen
    if len(out) < N:
        out.extend([0] * (N - len(out)))
    return out[:N]


def hexstr(tiles: list[int]) -> str:
    return "".join(f"{t & 0xFF:02X}" for t in tiles)


def cstr(b: bytes) -> str:
    z = b.find(0)
    if z >= 0:
        b = b[:z]
    return b.decode("latin-1", "replace")


def parse_optional(reader: Reader, total: int) -> dict:
    """Optionale Felder einlesen -> dict mit title/hint/password/traps/cloners/monsters."""
    end = reader.i + total
    res = {"title": "", "hint": "", "password": "",
           "traps": [], "cloners": [], "monsters": []}
    while reader.i < end:
        ftype = reader.u8()
        flen = reader.u8()
        payload = reader.bytes(flen)
        if ftype == 0x03:                       # Titel
            res["title"] = cstr(payload)
        elif ftype == 0x07:                     # Hinweis
            res["hint"] = cstr(payload)
        elif ftype == 0x06:                     # Passwort (XOR 0x99)
            dec = bytes(x ^ 0x99 for x in payload)
            # robust gegen (un)verschluesselten Terminator: nur druckbare ASCII
            res["password"] = "".join(c for c in cstr(dec) if 32 <= ord(c) < 127)
        elif ftype == 0x08:                     # Passwort unverschluesselt
            res["password"] = cstr(payload)
        elif ftype == 0x04:                     # Fallen (10-Byte-Records)
            for o in range(0, len(payload) - 9, 10):
                bx, by, tx, ty = struct.unpack_from("<HHHH", payload, o)
                res["traps"].append((bx, by, tx, ty))
        elif ftype == 0x05:                     # Klon-Maschinen (8-Byte-Records)
            for o in range(0, len(payload) - 7, 8):
                bx, by, mx, my = struct.unpack_from("<HHHH", payload, o)
                res["cloners"].append((bx, by, mx, my))
        elif ftype == 0x0A:                     # Monster-Bewegungsliste (x,y Bytes)
            for o in range(0, len(payload) - 1, 2):
                res["monsters"].append((payload[o], payload[o + 1]))
        # andere Felder ignorieren
    reader.i = end
    return res


def parse_level(reader: Reader) -> dict:
    size = reader.u16()                # Bytes nach diesem Wort
    end = reader.i + size
    number = reader.u16()
    time = reader.u16()
    chips = reader.u16()
    _detail = reader.u16()             # Map-Detail (0/1)
    up_len = reader.u16()
    upper = rle_decode(reader.bytes(up_len))
    lo_len = reader.u16()
    lower = rle_decode(reader.bytes(lo_len))
    opt_len = reader.u16()
    opt = parse_optional(reader, opt_len)
    reader.i = end                     # robust gegen Restbytes

    def packlist(seq):
        return ";".join(",".join(str(v) for v in rec) for rec in seq)

    return {
        "title": opt["title"] or f"Level {number}",
        "number": number,
        "time": time,
        "chips": chips,
        "hint": opt["hint"],
        "password": opt["password"],
        "width": W, "height": H,
        "upper": hexstr(upper),
        "lower": hexstr(lower),
        "traps": packlist(opt["traps"]),
        "cloners": packlist(opt["cloners"]),
        "monsters": packlist(opt["monsters"]),
    }


def convert(path: Path) -> dict:
    data = path.read_bytes()
    reader = Reader(data)
    magic = reader.u32()
    if magic == 0x0102AAAC:
        ruleset = "lynx"
    elif magic == 0x0002AAAC:
        ruleset = "ms"
    else:
        # Manche CCL haben fuehrende Metadaten -> nach Magic suchen
        idx = data.find(b"\xAC\xAA\x02\x00")
        if idx < 0:
            idx = data.find(b"\xAC\xAA\x02\x01")
        if idx < 0:
            raise ValueError(f"Keine gueltige CC-Datei (Magic 0x{magic:08X}).")
        reader.i = idx
        magic = reader.u32()
        ruleset = "lynx" if magic == 0x0102AAAC else "ms"
    count = reader.u16()
    levels = []
    for _ in range(count):
        if reader.i >= len(data):
            break
        levels.append(parse_level(reader))
    return {"name": path.stem, "ruleset": ruleset, "levels": levels}


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 1
    src = Path(argv[1])
    if not src.exists():
        print(f"Nicht gefunden: {src}")
        return 1
    out = Path(argv[2]) if len(argv) > 2 else (
        Path(__file__).resolve().parent / "levels" / f"{src.stem}.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    result = convert(src)
    out.write_text(json.dumps(result, ensure_ascii=False, separators=(",", ":")),
                   encoding="utf-8")
    print(f"{result['name']}: {len(result['levels'])} Level ({result['ruleset']}) "
          f"-> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
