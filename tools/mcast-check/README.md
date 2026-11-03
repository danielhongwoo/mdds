# mcast-check

컨테이너 사이에 UDP 멀티캐스트가 전달되는지 확인하는 작은 진단 도구다.

## 왜 있는가
DDS 참가자는 멀티캐스트 (239.255.0.1:7400, 도메인 0)로 서로를 찾는다. 네트워크가 멀티캐스트를 넘기지 못하면 DDS는 오류 없이 상대를 못 찾으므로, DDS를 실행하기 전에 네트워크만 따로 시험한다. 쓰는 법과 원리는 [chapter2](../../book/02-multicast/README.md)에 있다. socat으로도 같은 시험을 할 수 있지만, 소켓 동작 (bind -> 그룹 가입 -> 수신)을 코드로 드러내려고 직접 만들었다. 같은 호출을 chapter8 이후 mdds의 UDP transport에서 다시 쓴다.

## 의도와 범위
- 재사용 빈도가 높은 도구가 아니다. 쓰는 때는 실행 환경이 바뀌었을 때 (Docker Desktop 업데이트, 호스트 변경, 컨테이너 추가)와 DDS 발견이 안 될 때 네트워크를 먼저 의심할 때 정도다.
- 기능은 `recv`와 `send` 두 가지로 묶어 두고 늘리지 않는다. 외부 의존성 없이 표준 라이브러리만 쓴다.
- chapter8에서 mdds의 UDP transport가 생기면 이 도구를 그 transport를 쓰는 예제로 옮길지 다시 검토한다.

## 하는 일
- `recv`: 0.0.0.0:7400에 bind하고 239.255.0.1 그룹에 가입한 뒤 (`IP_ADD_MEMBERSHIP`) 받은 패킷의 출발 주소와 앞부분을 찍는다. DDS 참가자가 SPDP를 받을 때와 같은 일이다.
- `send`: 그룹 주소로 `hello i/N from <자기 주소>`를 N번 보낸다. 보내는 쪽은 그룹에 가입할 필요가 없다.
- 기본값과 규격 근거는 `src/main.rs` 주석에 있다.

## 실행
벤더 컨테이너 안에서 돌린다. 벤더 이미지에는 넣지 않고, 별도 이미지 `mdds/tools`를 벤더 컨테이너의 네트워크 네임스페이스에 붙여 실행한다 (같은 `eth0`, 같은 IP).
```
cd docker
docker compose build tools
./mcast-check.sh fastdds recv      # 터미널 1
./mcast-check.sh cyclone send      # 터미널 2
```
자세한 절차와 원리: [docs/environment/docker-network.md "검증 명령"](../../docs/environment/docker-network.md#검증-명령)

## 한계
- `SO_REUSEADDR`를 걸지 않는다 (표준 라이브러리로는 bind 전에 설정할 수 없다). 같은 컨테이너에서 DDS가 이미 7400을 쓰고 있으면 `recv`가 실패한다. DDS를 띄우기 전에 쓰거나 `--port`로 다른 포트를 쓴다.
- IPv4만 다룬다.
