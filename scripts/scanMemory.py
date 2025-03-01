import os
import re
import sys
import platform
import ctypes
from ctypes import wintypes

# Constants for Windows memory scanning
PROCESS_QUERY_INFORMATION = 0x0400
PROCESS_VM_READ = 0x0010
PROCESS_VM_WRITE = 0x0020
PROCESS_VM_OPERATION = 0x0008

class MEMORY_BASIC_INFORMATION(ctypes.Structure):
    _fields_ = [
        ("BaseAddress", ctypes.c_void_p),
        ("AllocationBase", ctypes.c_void_p),
        ("AllocationProtect", wintypes.DWORD),
        ("RegionSize", ctypes.c_size_t),
        ("State", wintypes.DWORD),
        ("Protect", wintypes.DWORD),
        ("Type", wintypes.DWORD),
    ]

def scan_memory_windows(pid, search_pattern, replace_pattern=None):
    """
    Scan the memory of a process on Windows for a specific byte pattern.

    Args:
        pid (int): Process ID to scan.
        search_pattern (bytes): Byte pattern to search for.
        replace_pattern (bytes, optional): Byte pattern to replace the found pattern.

    Returns:
        list of tuples: Each tuple contains the address and size of the found pattern.
    """
    if replace_pattern is not None and len(search_pattern) != len(replace_pattern):
        print("Error: Search and replace patterns must have the same length.")
        sys.exit(1)

    process = ctypes.windll.kernel32.OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
        False,
        pid
    )
    if not process:
        print(f"Error: Unable to open process {pid}. Check permissions.")
        sys.exit(1)

    results = []
    address = 0
    mbi = MEMORY_BASIC_INFORMATION()
    while ctypes.windll.kernel32.VirtualQueryEx(process, address, ctypes.byref(mbi), ctypes.sizeof(mbi)):
        if mbi.State == 0x1000 and (mbi.Protect & 0x01):  # MEM_COMMIT and readable memory
            buffer = ctypes.create_string_buffer(mbi.RegionSize)
            bytes_read = ctypes.c_size_t()

            if ctypes.windll.kernel32.ReadProcessMemory(process, mbi.BaseAddress, buffer, mbi.RegionSize, ctypes.byref(bytes_read)):
                chunk = buffer.raw[:bytes_read.value]
                offset = chunk.find(search_pattern)

                while offset != -1:
                    start_address = mbi.BaseAddress + offset
                    results.append((start_address, start_address + len(search_pattern)))

                    if replace_pattern is not None:
                        written = ctypes.c_size_t()
                        ctypes.windll.kernel32.WriteProcessMemory(
                            process,
                            start_address,
                            replace_pattern,
                            len(replace_pattern),
                            ctypes.byref(written)
                        )
                        print(f"Replaced pattern at address: {hex(start_address)}")

                    offset = chunk.find(search_pattern, offset + len(search_pattern))

        address += mbi.RegionSize

    ctypes.windll.kernel32.CloseHandle(process)
    return results

def scan_memory_linux(pid, search_pattern, replace_pattern=None):
    """
    Scan the memory of a process on Linux for a specific byte pattern.

    Args:
        pid (int): Process ID to scan.
        search_pattern (bytes): Byte pattern to search for.
        replace_pattern (bytes, optional): Byte pattern to replace the found pattern.

    Returns:
        list of tuples: Each tuple contains the start address and memory segment where the pattern was found.
    """
    if replace_pattern is not None:
        if len(search_pattern) != len(replace_pattern):
            print("Error: Search and replace patterns must have the same length.")
            sys.exit(1)

    maps_path = f"/proc/{pid}/maps"
    mem_path = f"/proc/{pid}/mem"
    results = []

    try:
        with open(maps_path, 'r') as maps_file:
            maps = maps_file.readlines()

        with open(mem_path, 'r+b', 0) as mem_file:
            for line in maps:
                match = re.match(r"([0-9a-f]+)-([0-9a-f]+)", line)
                if not match:
                    continue

                start, end = [int(addr, 16) for addr in match.groups()]
                mem_file.seek(start)

                try:
                    mem_file.seek(start)
                    chunk = mem_file.read(end - start)

                    offset = chunk.find(search_pattern)
                    while offset != -1:
                        results.append((start + offset, start + offset + len(search_pattern)))

                        if replace_pattern is not None:
                            mem_file.seek(start + offset)
                            mem_file.write(replace_pattern)
                            print(f"Replaced pattern at address: {hex(start + offset)}")

                        offset = chunk.find(search_pattern, offset + len(search_pattern))
                except (OSError, ValueError):
                    continue
    except FileNotFoundError:
        print(f"Error: Process {pid} does not exist or access is denied.")
        return results
    except PermissionError:
        print(f"Error: Permission denied. Try running as root.")
        return results

    return results

def scan_memory(pid, search_pattern, replace_pattern=None):
    """
    Detect the platform and call the appropriate memory scanner.
    """
    if platform.system() == "Windows":
        return scan_memory_windows(pid, search_pattern, replace_pattern)
    elif platform.system() == "Linux":
        return scan_memory_linux(pid, search_pattern, replace_pattern)
    else:
        print(f"Unsupported platform: {platform.system()}")
        sys.exit(1)

def main():
    if len(sys.argv) != 3 and len(sys.argv) != 4:
        print(f"Usage: {sys.argv[0]} <PID> <STRING> [REPLACEMENT]")
        sys.exit(1)

    try:
        pid = int(sys.argv[1])
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
        print("Error: Invalid PID or byte pattern.")
        sys.exit(1)

    matches = scan_memory(pid, search_pattern, replace_pattern)
    if matches:
        print(f"Found pattern '{search_pattern.hex()}' in memory segments:")
        for start, end in matches:
            print(f"  - Start: {hex(start)}, End: {hex(end)}")
    else:
        print(f"No matches found for pattern '{search_pattern.hex()}' in process {pid}.")

if __name__ == "__main__":
    main()

'''
# Patch CQ:

# > Determine the PID of Conquest.exe first
PID=$(ps -eopid,cmd | grep 'Conquest.exe$' | grep -v 'C:\\windows\\system32\\start.exe' | awk '{print $1}')
# > Skip the SSL check
sudo python3 scanMemory.py $PID 81e1ee0f000083c1158bc1 81e1ee0f0000b815000000
# > Patch the URL endpoints
sudo python3 scanMemory.py $PID '"fesl.ea.com\x00".encode()' '"mordorwi.de\x00".encode()'
sudo python3 scanMemory.py $PID '".ea.com\x00".encode()' '"orwi.de\x00".encode()'
sudo python3 scanMemory.py $PID '".fesl\x00".encode()' '".mord\x00".encode()'
'''
