set -e
set -x

docker run --rm -it -v "$(pwd)":/workshop -p 8080:8080 wasm-workshop:latest bash
