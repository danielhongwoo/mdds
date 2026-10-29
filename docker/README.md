# docker/ - Cyclone DDS / Fast DDS 상호운용 캡처 환경

[roadmap](../docs-for-agent/roadmap.md) | [네트워크 설계](../docs/environment/docker-network.md) | [빌드 설계](../docs/environment/vendor-build.md) | [chapter 1-4 노트](../docs-for-agent/notes/01-04-environment-and-captures.md)

OMG 상호운용 테스트 프로그램 shape_main을 Cyclone DDS와 Fast DDS로 각각 소스에서 빌드하고, 컨테이너 두 개에서 서로 통신시켜 그 패킷을 캡처한다. 캡처는 이후 chapter에서 정답 바이트 (테스트 데이터)로 쓴다. chapter 1에서 만들었고, 이후 mdds의 상호운용 시험에도 같은 환경을 쓴다.

## 구성
```
docker/
  compose.yaml          네트워크 dds-net 과 서비스 두 개
  cyclone/Dockerfile    -> 이미지 mdds/cyclone-shape:11.0.1-f2aaccb4   (/usr/local/bin/cyclone_shape)
  fastdds/Dockerfile    -> 이미지 mdds/fastdds-shape:3.6.1             (/usr/local/bin/fastdds_shape)
  tools/Dockerfile      -> 이미지 mdds/tools   (/usr/local/bin/mcast-check, 저장소 루트를 컨텍스트로 빌드)
  mcast-check.sh        벤더 컨테이너의 네트워크 네임스페이스에서 mcast-check 실행
  capture-shape-scenario.ts  shape_main 시나리오 하나 실행: 캡처, 구독, 발행, 정리 (Deno 2)
  deno.json             Deno fmt / lint 설정 (규약: docs-for-agent/conventions/typescript.md)
```
- 벤더마다 이미지와 컨테이너가 따로 있다. `cyclone` 컨테이너에는 `cyclone_shape`만, `fastdds` 컨테이너에는 `fastdds_shape`만 있다.
- 두 이미지 모두 같은 베이스 이미지와 같은 shape_main 소스 커밋을 쓴다. 이 두 값은 두 Dockerfile에 각각 적혀 있으므로 **바꿀 때는 함께 고친다.**
- 두 이미지에 캡처와 진단 도구 (`tcpdump`, `socat`, `ping`, `pgrep` / `pkill`, `ip`)를 똑같이 넣었다. 어느 쪽에서든 캡처할 수 있다.
- 컨테이너는 `sleep infinity`로 대기만 한다. DDS와 tcpdump는 `docker compose exec`로 실행한다. 그래야 캡처를 먼저 켜고 DDS를 시작할 수 있다.

## 네트워크
```
실행 호스트 (기본: Mac Docker Desktop의 Linux VM)
  +-- bridge 네트워크 "dds-net" 172.28.0.0/24  (gateway 172.28.0.1)
        +-- Docker container "cyclone"   172.28.0.10   cyclone_shape   (image mdds/cyclone-shape)
        +-- Docker container "fastdds"   172.28.0.11   fastdds_shape   (image mdds/fastdds-shape)
```
- SPDP 멀티캐스트 239.255.0.1:7400 (도메인 0)과 유니캐스트 모두 이 bridge 안에서만 돈다. 포트를 publish하지 않으며 호스트는 DDS에 참여하지 않는다.
- IP와 호스트 이름을 고정해서 캡처에서 .10이 Cyclone, .11이 Fast DDS임을 바로 알 수 있다.
- 왜 이렇게 구성했는지, 주소 배정 규칙, 멀티캐스트가 넘어가는 원리, 문제 대응은 [docker-network.md](../docs/environment/docker-network.md)에 있다.

## 실행 호스트
- 기본은 **Mac (Apple Silicon)의 Docker Desktop**이다. 이미지는 arm64로 네이티브 빌드된다 (에뮬레이션 없음). 빌드, 실행, 캡처, Wireshark 분석이 모두 Mac 한 곳에서 끝난다.
- Docker Desktop 메모리는 8 GB 이상 (16 GB 권장). Fast DDS C++ 빌드가 메모리를 많이 쓴다.
- 저장소가 Docker Desktop의 파일 공유 목록 (기본 `/Users`) 안에 있어야 캡처 디렉터리 바인드 마운트가 된다.
- 별도 x86_64 Linux 머신에서도 같은 파일로 돈다. 차이점 (ssh, rsync, root 소유 pcap)은 [docker-network.md](../docs/environment/docker-network.md#선택-별도-x86_64-linux-머신)를 본다.

## 사용법
명령은 모두 `~/work/mdds/docker`에서 실행한다.

### 1. 빌드와 실행
```
mkdir -p ../captures/cyclone-fastdds        # 없으면 Docker가 root 소유로 만든다. 컨테이너에는 captures 전체가 /captures 로 붙는다
docker compose build --progress plain     # 첫 빌드는 Apple Silicon에서 약 3분. 두 번째부터는 바뀐 단계만
docker compose up -d
docker compose ps                         # cyclone, fastdds 두 개가 running
```
- 병렬 수 기본값은 CPU 수다. 메모리 부족으로 컴파일러가 죽거나 Docker Desktop이 재시작되면: `docker compose build --build-arg JOBS=4` (8 GB면 4, 그래도 죽으면 2)
- 한쪽만 다시 빌드: `docker compose build cyclone` 또는 `docker compose build fastdds`
- Dockerfile이나 버전을 바꿨으면 [vendor-build.md "빌드 후 확인"](../docs/environment/vendor-build.md#빌드-후-확인)을 따른다. 그대로 쓰는 경우에는 필요 없다.

### 2. 네트워크 확인 (DDS를 띄우기 전에)
```
docker compose build tools                # 처음 한 번. tools 는 profile 이 있어 up 으로는 뜨지 않는다
docker compose exec cyclone ping -c1 fastdds

# 터미널 1: fastdds 컨테이너의 eth0 에서 239.255.0.1:7400 수신 (Ctrl+C로 끝낸다)
./mcast-check.sh fastdds recv
# 터미널 2: 가입 확인 후 cyclone 컨테이너의 eth0 에서 3번 송신
docker compose exec fastdds ip maddr show eth0          # 239.255.0.1 이 보여야 한다
./mcast-check.sh cyclone send
```
터미널 1에 `hello 1/3 from 172.28.0.10` 등이 찍히면 bridge가 멀티캐스트를 전달하는 것이다. 옵션은 `./mcast-check.sh fastdds --help`. 동작 원리와 socat 대안은 [docker-network.md "검증 명령"](../docs/environment/docker-network.md#검증-명령).

### 3. 시나리오 캡처
```
brew install deno                         # 처음 한 번. 외부 모듈은 쓰지 않는다
./capture-shape-scenario.ts --list                  # 시나리오 이름
./capture-shape-scenario.ts rel-cyclone-pub         # 약 25초. 결과가 이미 있으면 멈춘다 (덮어쓰려면 --force)
```
실행 흐름과 시나리오 표는 [chapter 1-4 노트](../docs-for-agent/notes/01-04-environment-and-captures.md#시나리오-실행-흐름-capture-shape-scenariots)에, 캡처별 관찰은 [captures/cyclone-fastdds/README.md](../captures/cyclone-fastdds/README.md)에 있다. 결과는 `../captures/cyclone-fastdds/<이름>.pcap`, `.pub.log`, `.sub.log`로 생긴다 (컨테이너 안에서는 `/captures/cyclone-fastdds/`).
- 바인드 마운트라 파일이 Mac의 작업 트리에 바로 생긴다. Wireshark로 열고 그대로 커밋한다.

### 정리
```
docker compose down                       # 컨테이너와 dds-net 제거. 이미지와 캡처는 남는다
```

## 빌드 소스와 버전
태그로 clone한 뒤 커밋 해시를 검사하므로, 태그가 옮겨지면 빌드가 멈춘다. 해시는 2026-10-28 `git ls-remote`로 확인했다 (모두 lightweight 태그라 태그 해시가 곧 커밋 해시다).

Cyclone만 릴리스 태그가 아니라 master 커밋이다. dds-rtps의 C판 shape_main이 11.0.1 이후에 추가된 API (`dds_read_next_instance`, `dds_take_next_instance`)를 쓰기 때문이다. 자세한 근거는 [vendor-build.md](../docs/environment/vendor-build.md#cyclone을-master-커밋으로-고정하는-이유).

| 구성 요소 | 이미지 | 태그 / 커밋 |
|---|---|---|
| 베이스 이미지 | 둘 다 | `ubuntu:noble-20261024.1@sha256:90b3a804b4ba0a340cfa4ba8950a543464e15d97d4fc33a74c9613d239661f10` |
| omg-dds/dds-rtps (shape_main 소스) | 둘 다 | 커밋 `526dc56eed8b1965a81ba5b7a644c4489337fe0a` (태그 v2.1.2026에는 `srcC/`가 없음) |
| eclipse-cyclonedds/cyclonedds | cyclone | master (버전 번호 11.0.1) / `f2aaccb4873c4bc33d59ee45c8568a4653900f78` (2026-06-02) |
| eProsima/Fast-DDS | fastdds | `v3.6.1` / `4e81e8b71bcd6e7c5213c000503cba8e49d6022a` |
| eProsima/Fast-CDR | fastdds | `v2.3.5` / `7d33a3b51a1585f5631b0a8d905bcc4f249d0f34` |
| eProsima/foonathan_memory_vendor | fastdds | `v1.4.1` / `347cb67581e51273a612780eb256a3c134c10bae` |
| eProsima/Fast-DDS-Gen | fastdds | `v4.3.0` / `cc0072b8849b35c67bd7e187990efad58e2871ae` |

- 빌드 중 네트워크가 필요한 곳: 각 저장소 clone, foonathan_memory_vendor가 받는 foonathan/memory 소스, Fast-DDS-Gen의 Gradle 배포본과 Maven 의존성.
- 버전을 올릴 때: Dockerfile의 `ARG` (태그와 해시를 함께), compose의 `image:` 태그, 이 표를 같이 고친다. 다시 빌드한 뒤 [vendor-build.md "빌드 후 확인"](../docs/environment/vendor-build.md#빌드-후-확인)을 하고, 결과가 달라졌으면 "빌드 결과"를 고친다. Cyclone이 next_instance API를 포함한 릴리스 태그를 내면 그 태그로 옮기는 것을 검토한다.
- 빌드 문제 대응은 [vendor-build.md "빌드 문제와 대응"](../docs/environment/vendor-build.md#빌드-문제와-대응)을 본다.
