# MordorWide EA Nation Game Server

This work implements the EA Nation backend server for public matchmaking for the game The Lord of the Rings: Conquest. The game server is hosted on [MordorWi.de](https://mordorwi.de/).

In order to work, it is assumed that the game binary is patched to point to the IP of this game server, and that the SSL verification is disabled. You find more information for the patching on [MordorWi.de](https://mordorwi.de/) or directly at the corresponding repositories [ConquestPatch](https://github.com/MordorWide/ConquestPatch) and [ConquestServerPatch](https://github.com/MordorWide/ConquestServerPatch).

While the certificates in the repository are there for demonstration purposes, you may create a new pair. However, due to the disabled SSL verification and an outdated cipher suite, no truely secure connection can be established.

## How to Run
### Dev Mode
If the game server should be launched in development mode without containerization, do the following steps.
1. Generate the certificate using: `cd data && ../scripts/generateCertificate.sh && cd ..`
2. Ensure that Rust and Cargo are installed and up to date.
3. Build the legacy OpenSSL library with support for unsecure cipher suites by running `./deps/setup_deps.sh`. Make sure that all relevant dependencies are installed.
4. Update the environmental variables in `env.standalone` and load it via `ENV_ARGS="$(grep -v '^#' env.standalone  | grep -v '^$' | tr '\n' ' '  )"`.
5. Run `eval "$ENV_ARGS cargo run"` to build and run the game server with the environmental variables set.
6. Make sure that the domains in use are pointing towards your dev machine, e.g. add to the host file `/etc/hosts` the lines:
```
# Unpatched domain (still requires the SSL patch)
127.0.0.1 lotr-pandemic-pc.fesl.ea.com

# Or for the patched files
127.0.0.1 lotr-pandemic-pc.mordorwi.de
127.0.0.1 mordorwi.de
127.0.0.1 theater.mordorwi.de
```

### Container Mode
1. Make sure that Docker is installed.
2. Generate the certificate using: `cd data && ../scripts/generateCertificate.sh && cd ..`
3. Build the image using `docker compose -f docker-compose.standalone.yml build`.
4. Update the environmental variables in `env.standalone`.
5. Run the standalone container with `docker compose --env-file env.standalone -f docker-compose.standalone.yml up -d`
6. Check the logs via `docker logs -f mordorwide-eanation`
7. Stop the standalone server again with `docker compose --env-file env.standalone -f docker-compose.standalone.yml down -v`

## Acknowledgements
I developed this game server mostly to learn Rust, but also to revive the old EA Nation functionality from the game.
While at the beginning I did a lot of effortful reverse engineering, I later found several resources on GitHub that already implemented similar projects for other games and in other languages.

The following list is the set of projects I studied a lot during my Rust implementation, and you should really have a look at it as well.
- [Arcadia](https://github.com/valters-tomsons/arcadia)
- [BFBC2 MasterServer](https://github.com/GrzybDev/BFBC2_MasterServer)
- [OpenSpy](https://github.com/openspy/openspy-core)

The work from the authors of these projects deserves your attention as well!
