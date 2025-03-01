import sys

def scan_binary(file, search_pattern, replace_pattern=None):
    """
    Scan a binary file for a specific pattern.
    """
    matches = []

    if replace_pattern is not None:
        if len(search_pattern) != len(replace_pattern):
            raise Exception("Error: Search and replace patterns must have the same length.")

    with open(file, 'rb') as f:
        data = f.read()

    data = bytearray(data)

    offset = 0
    while True:
        offset = data.find(search_pattern, offset)
        if offset == -1:
            break

        matches.append((offset, offset + len(search_pattern)))

        if replace_pattern is not None:
            print(f"Replaced pattern at offset: {hex(offset)}")
            data[offset:offset+len(replace_pattern)] = replace_pattern

        offset += 1

    if replace_pattern is not None:
        with open(file, 'wb') as f:
            f.write(data)

    return matches


def main():
    if len(sys.argv) != 3 and len(sys.argv) != 4:
        print(f"Usage: {sys.argv[0]} <FILE> <STRING> [REPLACEMENT]")
        sys.exit(1)

    try:
        file = sys.argv[1]
        try:
            search_pattern = bytes.fromhex(sys.argv[2])
        except ValueError:
            search_pattern = eval(sys.argv[2])

        if len(sys.argv) == 4:
            try:
                replace_pattern = bytes.fromhex(sys.argv[3])
            except ValueError:
                replace_pattern = eval(sys.argv[3])
        else:
            replace_pattern = None
    except ValueError:
        print("Error: Invalid file or byte pattern.")
        sys.exit(1)

    matches = scan_binary(file, search_pattern, replace_pattern)
    if matches:
        print(f"Found pattern '{search_pattern.hex()}' in bytestream:")
        for start, end in matches:
            print(f"  - Start: {hex(start)}, End: {hex(end)}")
    else:
        print(f"No matches found for pattern '{search_pattern.hex()}' in file {file}.")

if __name__ == "__main__":
    main()

'''
# Patch CQ (ONLY WORKS ON non-SecurROM binaries, e.g. cracked executables):

# > Backup the original binary first
cp Conquest.exe Conquest.exe.bak
# > Skip the SSL check
python3 scanBinary.py Conquest.exe 81e1ee0f000083c1158bc1 81e1ee0f0000b815000000
# > Patch the URL endpoints
python3 scanBinary.py Conquest.exe '"fesl.ea.com\x00".encode()' '"mordorwi.de\x00".encode()'
python3 scanBinary.py Conquest.exe '".ea.com\x00".encode()' '"orwi.de\x00".encode()'
python3 scanBinary.py Conquest.exe '".fesl\x00".encode()' '".mord\x00".encode()'
# > (Optional) Skip the RAZOR1911 intro
python3 scanBinary.py Conquest.exe ffd0e995d31900 e995d319009090
'''
