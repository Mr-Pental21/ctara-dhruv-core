"""List export names of a PE (DLL), one per line, using only the stdlib.

Fallback for check_cdylib_artifact.sh on Windows runners where no
binutils-style tool reliably prints the COFF export table.
"""
import struct
import sys


def exports(path):
    with open(path, "rb") as f:
        data = f.read()
    if data[:2] != b"MZ":
        raise ValueError("not a PE file")
    pe_off = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe_off : pe_off + 4] != b"PE\0\0":
        raise ValueError("bad PE signature")
    coff = pe_off + 4
    num_sections = struct.unpack_from("<H", data, coff + 2)[0]
    opt_size = struct.unpack_from("<H", data, coff + 16)[0]
    opt = coff + 20
    magic = struct.unpack_from("<H", data, opt)[0]
    # Data directories start at 0x60 (PE32) / 0x70 (PE32+); entry 0 is exports.
    ddir_off = opt + (0x60 if magic == 0x10B else 0x70)
    exp_rva = struct.unpack_from("<I", data, ddir_off)[0]
    if exp_rva == 0:
        return []
    sections = []
    sec = opt + opt_size
    for i in range(num_sections):
        s = sec + i * 40
        vsize, vaddr, rsize, roff = struct.unpack_from("<IIII", data, s + 8)
        sections.append((vaddr, max(vsize, rsize), roff))

    def rva2off(rva):
        for vaddr, size, roff in sections:
            if vaddr <= rva < vaddr + size:
                return roff + (rva - vaddr)
        raise ValueError(f"rva {rva:#x} not mapped")

    e = rva2off(exp_rva)
    num_names = struct.unpack_from("<I", data, e + 24)[0]
    names_rva = struct.unpack_from("<I", data, e + 32)[0]
    names_off = rva2off(names_rva)
    out = []
    for i in range(num_names):
        name_rva = struct.unpack_from("<I", data, names_off + 4 * i)[0]
        off = rva2off(name_rva)
        end = data.index(b"\0", off)
        out.append(data[off:end].decode("ascii", "replace"))
    return out


if __name__ == "__main__":
    for name in exports(sys.argv[1]):
        print(name)
