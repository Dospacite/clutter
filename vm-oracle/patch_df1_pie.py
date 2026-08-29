#!/usr/bin/env python3
"""Clear DF_1_PIE from an ELF's DT_FLAGS_1 so old bionic linkers accept it.

NDK r28's lld sets DT_FLAGS_1 = DF_1_NOW|DF_1_PIE (0x08000001). Bionic
linkers older than Android 8 reject any bit they do not know, which blocks
running a modern arm32 analyzer under an API-24 sysroot. DF_1_PIE is purely
informational for the loader here -- the ELF is already ET_DYN and is loaded
position-independently either way -- so clearing the bit changes how the file
is described, not how it executes.

Usage: patch_df1_pie.py <elf-in> <elf-out>
"""

import struct
import sys

DT_NULL = 0
DT_FLAGS_1 = 0x6FFFFFFB
DF_1_PIE = 0x08000000


def patch(source: str, destination: str) -> int:
    with open(source, "rb") as handle:
        data = bytearray(handle.read())

    if data[:4] != b"\x7fELF":
        raise SystemExit(f"{source}: not an ELF file")
    if data[4] != 1:
        raise SystemExit(f"{source}: expected a 32-bit ELF")
    if data[5] != 1:
        raise SystemExit(f"{source}: expected little-endian")

    # ELF32 header: e_phoff at 0x1c, e_phentsize 0x2a, e_phnum 0x2c.
    e_phoff = struct.unpack_from("<I", data, 0x1C)[0]
    e_phentsize = struct.unpack_from("<H", data, 0x2A)[0]
    e_phnum = struct.unpack_from("<H", data, 0x2C)[0]

    PT_DYNAMIC = 2
    dynamic_offset = None
    dynamic_size = None
    for index in range(e_phnum):
        base = e_phoff + index * e_phentsize
        p_type, p_offset = struct.unpack_from("<II", data, base)
        if p_type == PT_DYNAMIC:
            p_filesz = struct.unpack_from("<I", data, base + 0x10)[0]
            dynamic_offset, dynamic_size = p_offset, p_filesz
            break

    if dynamic_offset is None:
        raise SystemExit(f"{source}: no PT_DYNAMIC segment")

    patched = 0
    cursor = dynamic_offset
    end = dynamic_offset + dynamic_size
    while cursor + 8 <= end:
        d_tag, d_val = struct.unpack_from("<Ii", data, cursor)
        if d_tag == DT_NULL:
            break
        if d_tag == DT_FLAGS_1:
            value = d_val & 0xFFFFFFFF
            if value & DF_1_PIE:
                struct.pack_into("<I", data, cursor + 4, value & ~DF_1_PIE)
                patched += 1
                print(
                    f"DT_FLAGS_1 0x{value:x} -> 0x{value & ~DF_1_PIE:x}",
                    file=sys.stderr,
                )
        cursor += 8

    if patched == 0:
        print("DF_1_PIE not set; copying unchanged", file=sys.stderr)

    with open(destination, "wb") as handle:
        handle.write(data)
    return patched


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    patch(sys.argv[1], sys.argv[2])
