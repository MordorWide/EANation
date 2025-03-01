# Load Dependencies

Run the following steps in order.

## OpenSSL
Install the perl dependencies for OpenSSL first
```bash
# Assuming Fedora
sudo dnf group install development-tools
sudo dnf install perl-FindBin perl-IPC-Cmd perl-File-Compare perl-File-Copy

# Assuming Ubuntu
sudo apt install build-essential zlib1g-dev perl
```

## Finally
Run the preparation script.
```bash
./setup_deps.sh
```