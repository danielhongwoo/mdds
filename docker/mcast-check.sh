#!/bin/sh
# 벤더 컨테이너의 네트워크 네임스페이스 안에서 mcast-check 를 실행한다.
#
#   ./mcast-check.sh fastdds recv          # fastdds 컨테이너의 eth0 에서 239.255.0.1:7400 수신
#   ./mcast-check.sh cyclone send          # cyclone 컨테이너의 eth0 에서 3번 송신
#
# 도구 컨테이너는 따로 뜨지만 --network container:<id> 로 벤더 컨테이너와 같은 eth0 (같은 IP) 를 쓴다.
# 벤더 이미지를 건드리지 않고 벤더 컨테이너와 똑같은 네트워크 조건에서 시험하기 위해서다.
set -eu
cd "$(dirname "$0")"

if [ $# -lt 2 ]; then
    echo "usage: $0 <cyclone|fastdds> recv|send [options]" >&2
    exit 2
fi
service=$1
shift

id=$(docker compose ps -q "$service")
if [ -z "$id" ]; then
    echo "error: '$service' 컨테이너가 떠 있지 않다. 먼저 docker compose up -d" >&2
    exit 1
fi

# 터미널에서 실행할 때만 -t 를 붙인다 (Ctrl+C 전달).
# --init: mcast-check 가 PID 1 이 되면 SIGINT 기본 동작이 무시되어 Ctrl+C 로 끝나지 않는다.
tty=""
if [ -t 0 ]; then tty="-t"; fi
exec docker run --rm -i $tty --init --network "container:$id" mdds/tools mcast-check "$@"
