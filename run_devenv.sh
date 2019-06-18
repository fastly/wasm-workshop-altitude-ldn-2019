set -e
set -x

docker run --rm -it -v "$(pwd)":/workshop -p 8080:8080 acfoltzer/wasm-workshop-altitude-ldn-2019:latest bash
