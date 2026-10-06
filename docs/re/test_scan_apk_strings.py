import struct
import unittest

from scan_apk_strings import dex_strings


class DexStringTests(unittest.TestCase):
    def test_string_table_ignores_binary_length_prefix(self):
        data = bytearray(116)
        data[:8] = b"dex\n035\0"
        struct.pack_into("<II", data, 56, 1, 112)
        struct.pack_into("<I", data, 112, 116)
        data.extend(b"\x06BA7500\0")
        self.assertEqual(list(dex_strings(bytes(data))), ["BA7500"])

    def test_rejects_bad_table_and_truncated_string(self):
        with self.assertRaises(ValueError):
            list(dex_strings(b"not a dex"))
        data = bytearray(112)
        data[:8] = b"dex\n035\0"
        struct.pack_into("<II", data, 56, 1, 200)
        with self.assertRaises(ValueError):
            list(dex_strings(bytes(data)))
        data.extend(struct.pack("<I", 116))
        struct.pack_into("<II", data, 56, 1, 112)
        with self.assertRaises(ValueError):
            list(dex_strings(bytes(data)))


if __name__ == "__main__":
    unittest.main()
