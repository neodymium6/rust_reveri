# generate
# const REVERSED: [[(u8, u8, u8, u8, u8); 8]; 64] = [
#     [
#         // (0)000000(0)
#         // (need_l, need_r, reverse_l, reverse_r, reverse_lr)
#         (   0,    0,    0,    0,    0), // put 10000000
#         (   0,    0,    0,    0,    0), // put 01000000
#         (   0,    0,    0,    0,    0), // put 00100000
#         (   0,    0,    0,    0,    0), // put 00010000
#         (   0,    0,    0,    0,    0), // put 00001000
#         (   0,    0,    0,    0,    0), // put 00000100
#         (   0,    0,    0,    0,    0), // put 00000010
#         (   0,    0,    0,    0,    0), // put 00000001
#     ],
#     [
#         // (0)000001(0)
#         (   0,    0,    0,    0,    0), // put 10000000
#         (   0,    0,    0,    0,    0), // put 01000000
#         (   0,    0,    0,    0,    0), // put 00100000
#         (   0,    0,    0,    0,    0), // put 00010000
#         (   0,    0,    0,    0,    0), // put 00001000
#         (   0, 0x01,    0, 0x02, 0x02), // put 00000100
#         (   0,    0,    0,    0,    0), // put 00000010
#         (0x04,    0, 0x02,    0, 0x02), // put 00000001
#     ],
#     [
#         // (0)000011(0)
#         (   0,    0,    0,    0,    0), // put 10000000
#         (   0,    0,    0,    0,    0), // put 01000000
#         (   0,    0,    0,    0,    0), // put 00100000
#         (   0,    0,    0,    0,    0), // put 00010000
#         (   0, 0x01,    0, 0x06, 0x06), // put 00001000
#         (   0,    0,    0,    0,    0), // put 00000100
#         (   0,    0,    0,    0,    0), // put 00000010
#         (0x08,    0, 0x06,    0, 0x06), // put 00000001
#     ],
#     ...
# ];


def to_bit_string(n):
    return format(n, "08b")


def to_bit_string_hex(n):
    return format(n, "02x")


def get_need_l(i, j):
    put = 1 << (7 - j)
    state8 = i << 1
    if state8 & put:
        # puting already put position
        # not happen in game
        return 0
    if state8 & (put << 1) == 0:
        # no opposite stone in left
        return 0
    put <<= 1
    while state8 & put:
        put <<= 1
    return put


def get_need_r(i, j):
    put = 1 << (7 - j)
    state8 = i << 1
    if state8 & put:
        # puting already put position
        # not happen in game
        return 0
    if state8 & (put >> 1) == 0:
        # no opposite stone in right
        return 0
    put >>= 1
    while state8 & put:
        put >>= 1
    return put


def get_reverse_l(i, j):
    put = 1 << (7 - j)
    state8 = i << 1
    if get_need_l(i, j) == 0:
        return 0
    other = get_need_l(i, j)
    res = 0
    other = other >> 1
    while other != put:
        res |= other
        other = other >> 1
    return res


def get_reverse_r(i, j):
    put = 1 << (7 - j)
    state8 = i << 1
    if get_need_r(i, j) == 0:
        return 0
    other = get_need_r(i, j)
    res = 0
    other = other << 1
    while other != put:
        res |= other
        other = other << 1
    return res


def get_reverse_lr(i, j):
    return get_reverse_l(i, j) | get_reverse_r(i, j)


def get_02x(n):
    if n == 0:
        return "   0"
    return f"0x{to_bit_string_hex(n)}"


def main():
    with open("tmp.txt", "w") as f:
        f.write("const REVERSED: [[(usize, usize, u64, u64, u64); 8]; 64] = [\n")
        for i in range(64):
            f.write("    [\n")
            f.write(f"        // (0){to_bit_string(i)[2:8]}(0)\n")
            for j in range(8):
                f.write("        (")
                need_l = get_need_l(i, j)
                need_r = get_need_r(i, j)
                reverse_l = get_reverse_l(i, j)
                reverse_r = get_reverse_r(i, j)
                reverse_lr = get_reverse_lr(i, j)
                f.write(f"{get_02x(need_l)}, ")
                f.write(f"{get_02x(need_r)}, ")
                f.write(f"{get_02x(reverse_l)}, ")
                f.write(f"{get_02x(reverse_r)}, ")
                f.write(f"{get_02x(reverse_lr)}")
                f.write(f"),  // put {to_bit_string(1 << (7 - j))}\n")
            f.write("    ],\n")
        f.write("];\n")


if __name__ == "__main__":
    main()
