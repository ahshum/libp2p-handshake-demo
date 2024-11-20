#!/usr/bin/env sh

COMPOSE="docker compose"

init() {
  mkdir -p .data/ipfs
  mkdir -p .data/rust
  echo "*" > .data/.gitignore

  echo "PUID=$(id -u)" > .env
  echo "PGID=$(id -g)" >> .env
  echo "RUST_LOG=info" >> .env

  $COMPOSE run --rm \
    -v "$PWD:/app" \
    -w "/app" \
    --entrypoint="" \
    ipfs ./cmd.sh init-keys
}

init_keys() {
  ipfs init --algorithm=ed25519
  ipfs key gen --type=ed25519 local
  ipfs key export --format=pem-pkcs8-cleartext --output=ed25519.pem local
}

cmd="$1"
[ "$#" -ge 1 ] && shift

case "$cmd" in
  init)
    init
    ;;

  init-keys)
    init_keys
    ;;

  dev)
    $COMPOSE run --rm -it \
      dev bash
    ;;

  test)
    $COMPOSE run --rm \
      local cargo t
      ;;

  handshake)
    mode=${1:-local}
    $COMPOSE run --rm \
      -w "/app/handshake" \
      $mode \
      sh -c "cargo r"
    ;;

  target-node-up)
    $COMPOSE up -d ipfs
    ;;

  target-node-log)
    peerid=${1:-$(cat peerid)}
    docker logs -f --since=30s handshake-ipfs 2>&1 | grep "$peerid"
    ;;

  target-node-peer)
    peerid=${1:-$(cat peerid)}
    docker exec handshake-ipfs \
      watch -n 0.5 "ipfs swarm peers | grep $peerid"
    ;;

  clean)
    $COMPOSE down
    rm -rf .data/
    rm *.pem peerid .env
    ;;
esac
