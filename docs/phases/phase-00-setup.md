# Phase 00 - 준비 (구현 없음, 관찰 단계)

상태: 대기

[roadmap](../roadmap.md) | 다음: [Phase 01](phase-01-dcps-concepts.md)

## 목표
규격을 읽기 전에 실제 DDS 트래픽을 눈으로 본다. Cyclone DDS와 Fast DDS가 Docker 컨테이너 두 개에서 서로 통신하게 만들고, 그 패킷을 캡처해 이후 단계의 테스트 데이터(정답 바이트)로 쓴다. 프로토콜 코드는 쓰지 않는다.

## 범위
하는 것
- Docker 기반 실행 환경 구성 (`docker/`)
- 두 구현 사이의 통신 확인과 시나리오별 패킷 캡처 (`tests/fixtures/`)
- Cargo 워크스페이스 뼈대와 프로젝트 규약 확정

하지 않는 것
- RTPS / CDR 코드 작성, 캡처 파일을 읽는 테스트 코드 (Phase 02, 03에서)
- 규격 정독 (Phase 01부터)
- ROS 2 환경 (Phase 08에서)
- `ddsperf` 등 벤더 고유 도구. 모든 시나리오는 OMG `shape_main` 하나로 만든다 (큰 샘플도 `--additional-payload-size`로 가능).

## 산출물
| 산출물 | 위치 |
|---|---|
| Docker 이미지 정의, compose 파일, 시나리오 실행 스크립트, 사용법 | `docker/` |
| 시나리오별 캡처 파일, 양쪽 stdout 로그, 설명 | `tests/fixtures/phase00/<이름>.pcap`, `<이름>.pub.log`, `<이름>.sub.log`, `README.md` |
| 빈 crate 세 개가 있는 Cargo 워크스페이스 | `Cargo.toml`, `crates/` |
| 확정된 프로젝트 규약 | 이 문서의 "프로젝트 규약" 절, `CLAUDE.md` |
| 캡처 분석 메모 (어느 프레임에 무엇이 보이는지) | `tests/fixtures/phase00/README.md`, 이 문서의 "진행 기록" |

## 작업 순서

### Step 1. 실행 호스트 준비
**Mac (Apple Silicon, arm64)의 Docker Desktop에서 빌드, 실행, 캡처를 모두 하는 것을 기본**으로 한다 (2026-10-28 결정). shape_main과 두 벤더의 DDS 라이브러리를 이미지 안에서 소스로 빌드하므로 아키텍처 제약이 없고, arm64로 네이티브 빌드된다 (에뮬레이션 없음). 상세는 아래 "실행 호스트: Mac (Docker Desktop)" 절.

- [ ] Docker Desktop 설치, `docker compose version`이 동작한다.
- [ ] Docker Desktop 자원: 메모리 8 GB 이상 (Fast DDS C++ 빌드용, 16 GB 권장), 디스크 여유 10 GB 이상 (빌드 중간 산출물 포함).
- [ ] Wireshark (`brew install --cask wireshark`). 캡처 파일을 여는 용도로만 쓴다.
- [ ] 확인: `docker run --rm ubuntu:24.04 uname -m` 이 `aarch64`를 출력한다.

### Step 2. 저장소 뼈대
- [ ] `rust-toolchain.toml` (stable 버전 고정), 루트 `Cargo.toml` (workspace), `crates/mdds-cdr`, `crates/mdds-rtps`, `crates/mdds` (각각 `lib.rs`만 있는 빈 crate)
- [ ] `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt --check` 통과
- [ ] `docker/`, `tests/fixtures/phase00/` 디렉터리

### Step 3. Docker 이미지와 compose
- [ ] `docker/cyclone/Dockerfile`, `docker/fastdds/Dockerfile`: 벤더마다 shape_main을 **Docker 안에서 소스로 빌드**한다. 설계는 [environment/vendor-build.md](../environment/vendor-build.md).
  - [x] 첫 빌드 (`docker compose build`)가 끝나고, 빌드 시간과 이미지 크기를 진행 기록에 적는다.
  - [x] 각 컨테이너에서 `ldd /usr/local/bin/<벤더>_shape` 로 `not found`가 없는지 확인한다.
  - [x] `cyclone_shape -h`, `fastdds_shape -h` 로 옵션이 README와 같은지 확인한다.
  - [x] 버전 확인: Fast DDS 쪽 원래 실행 파일 이름에 `3.6.1`이 들어 있다 (실제 이름은 `eprosima_fastdds-3.6.1.0_shape_main_linux`) (CMake가 라이브러리 버전으로 이름을 짓는다). Cyclone 쪽은 직접 컴파일하므로 `docker/README.md`의 버전 기록으로 확인한다.
- [ ] `docker/compose.yaml`: 아래 "Docker 네트워크 구성" 절의 설계대로.
- [ ] `docker/run-scenario.sh <시나리오>`: 아래 "시나리오 실행 흐름"대로. 스크립트는 자기 위치(`docker/`)로 `cd`한 뒤 시작한다.
- [ ] `docker compose up -d --build` 로 두 컨테이너가 뜬다.
- [ ] `docker/README.md`: 사용법 (Mac의 Docker Desktop에서 빌드와 실행), 네트워크 설명 (이 문서의 네트워크 절을 요약하고 링크), 빌드에 쓴 소스 태그와 커밋 해시.

### Step 4. 네트워크 검증 (DDS를 띄우기 전에)
- [ ] 컨테이너 두 개가 서로 ping 된다 (`docker compose exec cyclone ping -c1 fastdds`). 이후 명령은 모두 `docker/`에서 친다.
- [ ] 멀티캐스트가 컨테이너 사이를 넘어간다. `socat`으로 239.255.0.1:7400에 보내고 받는다 (명령은 네트워크 절 참고). 받는 쪽에서 `ip maddr show eth0`에 239.255.0.1이 보이는 것도 확인한다.
- [ ] 캡처가 된다. `tcpdump`가 위 socat 패킷과 IGMP 가입 패킷을 잡고, pcap 파일이 Mac에서 열린다.
- 여기서 실패하면 "예상 문제와 대응" 절을 본다. DDS를 띄우기 전에 해결한다.

### Step 5. 첫 통신
- [ ] `run-scenario.sh rel-cyclone-pub`: Cyclone 발행 -> Fast DDS 구독. `rel-cyclone-pub.sub.log`에 샘플이 찍힌다.
- [ ] `run-scenario.sh rel-fastdds-pub`: 반대 방향
- [ ] 캡처에서 SPDP (7400 멀티캐스트), SEDP, DATA, HEARTBEAT, ACKNACK를 Wireshark `rtps` 필터로 식별한다.

### Step 6. 시나리오 캡처
- 아래 시나리오 표의 각 항목을 `run-scenario.sh`로 실행해 `tests/fixtures/phase00/`에 저장한다.
- [ ] be-cyclone-pub, be-fastdds-pub
- [ ] rel-cyclone-pub, rel-fastdds-pub
- [ ] dispose-on-exit
- [ ] lease-expiry
- [ ] large-sample
- [ ] xcdr2

### Step 7. 캡처 분석 메모
- [ ] `tests/fixtures/phase00/README.md`에 시나리오마다: 실행 명령, 벤더와 버전, 캡처한 컨테이너, 주목할 프레임 번호 (첫 SPDP, 첫 SEDP, 첫 DATA, HEARTBEAT / ACKNACK 왕복, dispose, 첫 DATA_FRAG)를 적는다.
- [ ] 두 벤더의 차이를 메모한다: 벤더 ID, 프로토콜 버전, 벤더 전용 PID, 데이터 표현 ID, SPDP의 lease 기간 값, 유니캐스트 포트 (Fast DDS 7410 / 7411, Cyclone은 임의 포트), Fast DDS SPDP에 실리는 공유 메모리 로케이터 (벤더 전용 kind).

### Step 8. 규약 확정
- [ ] 아래 "프로젝트 규약" 절의 "제안"을 "확정"으로 바꾸고 `CLAUDE.md`와 맞춘다.

## shape_main 이미지 빌드
Cyclone DDS, Fast DDS, shape_main을 Docker 안에서 소스로 빌드한다. 소스 버전, Dockerfile 구성, 빌드 문제 대응은 별도 문서 [environment/vendor-build.md](../environment/vendor-build.md)에 있다.

## 실행 호스트: Mac (Docker Desktop)

### 왜 Mac인가
- shape_main과 두 벤더 라이브러리를 소스에서 빌드하므로 아키텍처 제약이 없다. 같은 Dockerfile이 Apple Silicon에서 arm64로 네이티브 빌드된다.
- 처음에는 OMG가 배포한 x86-64 실행 파일을 쓰려고 해서 x86_64 머신이 필요했고, 소스 빌드로 바꾼 뒤에도 빌드 부하 때문에 별도 머신을 기본으로 두었다. 2026-10-28 Mac에서 빌드하기로 정하면서 Mac을 기본으로 바꿨다. 머신 없이 편집, 빌드, 캡처, 분석이 한 곳에서 끝나고 ssh / rsync 단계가 사라진다.
- 대가: Docker Desktop은 Linux VM 안에서 Docker Engine을 돌리므로 bridge도 VM 안에 있다. Mac에서는 컨테이너 사이 트래픽이 보이지 않고, VM 안 bridge가 멀티캐스트를 전달하는지는 직접 시험해야 한다 (Step 4) [추정].
- 아래 Docker 네트워크 설계는 어느 호스트에서나 같다. 바뀌는 것은 "누가 Docker 데몬을 돌리느냐"뿐이다.

### 구성도
```
+-- Mac (Apple Silicon, macOS) --------------------------------------------------------------------------------+
|                                                                                                              |
|  ~/work/mdds/tests/fixtures/phase00/                                                                         |
|        :                                                                                                     |
|        :  bind mount (Docker Desktop file sharing)                                                           |
|        :                                                                                                     |
|   +----:-- Docker Desktop Linux VM (arm64) --------------------------------------------------------------+   |
|   |    :                                                                                                 |   |
|   |    :...................................................                                              |   |
|   |    :                                                  :                                              |   |
|   |  +-:-- Docker container "cyclone" -------------+    +-:-- Docker container "fastdds" -------------+  |   |
|   |  | :                                           |    | :                                           |  |   |
|   |  | /captures  (mount point)                    |    | /captures  (mount point)                    |  |   |
|   |  |                                             |    |                                             |  |   |
|   |  | image    mdds/cyclone-shape:11.0.1-f2aaccb4 |    | image    mdds/fastdds-shape:3.6.1           |  |   |
|   |  | process  cyclone_shape                      |    | process  fastdds_shape                      |  |   |
|   |  |          tcpdump (if subscriber)            |    |          tcpdump (if subscriber)            |  |   |
|   |  |                                             |    |                                             |  |   |
|   |  | eth0     172.28.0.10                        |    | eth0     172.28.0.11                        |  |   |
|   |  +---#-----------------------------------------+    +---#-----------------------------------------+  |   |
|   |      #                                                  #                                            |   |
|   |      #  veth                                            #  veth                                      |   |
|   |      #                                                  #                                            |   |
|   |  ====#==================================================#==========================================  |   |
|   |                                                                                                      |   |
|   |      bridge "dds-net"   172.28.0.0/24   gateway 172.28.0.1                                           |   |
|   |      multicast  239.255.0.1:7400  (SPDP)                                                             |   |
|   |      unicast    172.28.0.10 <-> 172.28.0.11  (SEDP, DATA, HEARTBEAT, ACKNACK)                        |   |
|   |                                                                                                      |   |
|   +------------------------------------------------------------------------------------------------------+   |
|                                                                                                              |
+--------------------------------------------------------------------------------------------------------------+

Legend
  +---+  |    box outline (Mac, VM, Docker container)
  .....  :    bind mount (file sharing between Mac and the containers)
  =====  #    network link (bridge "dds-net" and each container's veth)
```
- **Docker container**: 이미지로 띄운 실행 인스턴스다. 벤더마다 하나씩, 이미지도 따로다 (`cyclone`은 `mdds/cyclone-shape`, `fastdds`는 `mdds/fastdds-shape`). 각 컨테이너는 자기 네트워크 네임스페이스 (`eth0` 하나)를 가지므로 DDS 입장에서는 서로 다른 호스트 두 대다.
- **tcpdump**: 두 컨테이너에 모두 있고, 시나리오마다 구독자 쪽 컨테이너에서 실행한다 (실행 흐름 참고).
- **bridge 구간 (VM 안)**: 두 컨테이너의 `eth0`은 veth 쌍으로 bridge에 꽂혀 있다. SPDP 멀티캐스트와 유니캐스트 (SEDP, DATA, HEARTBEAT, ACKNACK)가 모두 이 안에서만 돈다. Mac의 네트워크 인터페이스에는 DDS 패킷이 나오지 않는다. 바깥으로 나가는 것은 이미지 빌드 때 GitHub clone, apt, Gradle 의존성 다운로드뿐이다.
- **캡처 경로**: 구독자 컨테이너 안의 tcpdump -> `/captures` (바인드 마운트) -> Mac의 `~/work/mdds/tests/fixtures/phase00/` -> Wireshark. rsync가 필요 없다.
- **주소 요약**

| 위치 | 주소 | 비고 |
|---|---|---|
| bridge gateway | 172.28.0.1 | VM 안. DDS 참가자 아님 |
| cyclone eth0 | 172.28.0.10 | SPDP에 이 주소를 광고. 유니캐스트 포트는 임의 |
| fastdds eth0 | 172.28.0.11 | SPDP에 이 주소를 광고. 유니캐스트 7410 / 7411 |
| 멀티캐스트 | 239.255.0.1:7400 | SPDP (도메인 0). bridge 안에서만 |

### 주의
- `~/work/mdds`가 Docker Desktop의 파일 공유 목록 (기본: `/Users`) 안에 있어야 바인드 마운트가 된다.
- 첫 빌드는 Fast DDS C++ 빌드 때문에 메모리를 많이 쓴다. 컴파일러가 죽으면 Docker Desktop 메모리를 늘리거나 `--build-arg JOBS=2`로 병렬 수를 줄인다.
- 컨테이너 안 tcpdump는 root로 파일을 쓰지만, Docker Desktop의 파일 공유를 거치면 Mac에서는 Mac 사용자 소유로 보인다 [추정, 첫 캡처 때 확인].
- 호스트에서 bridge를 직접 캡처할 수 없다 (VM 안에 있다). 캡처는 항상 컨테이너 안 tcpdump로 한다.

### 선택: 별도 x86_64 Linux 머신
Mac 대신 Linux 머신의 Docker Engine을 써도 Dockerfile과 compose는 그대로다. 다른 점만 적는다.
- Docker Engine과 compose 플러그인을 설치하고 (`https://docs.docker.com/engine/install/ubuntu/`), 사용자를 `docker` 그룹에 넣는다. 저장소는 원격 git에서 clone하거나 Mac에서 `rsync -a`로 보낸다.
- ssh로 들어가 `~/work/mdds/docker`에서 같은 명령을 친다. 캡처는 머신에서 커밋하거나 `rsync -av <머신>:~/work/mdds/tests/fixtures/phase00/ tests/fixtures/phase00/`로 Mac에 가져온다. 한 시나리오는 한 곳에서만 커밋한다.
- pcap 파일은 머신에서 root 소유가 된다 (지울 때 `sudo`).
- Linux에서는 bridge가 호스트 커널에 바로 있어서, 디버깅 보조로 호스트에서 `br-<id>` 인터페이스를 캡처할 수 있다 (`docker network inspect dds-net`의 Id 앞 12자). 공식 절차는 어느 호스트에서나 같은 "컨테이너 안 tcpdump"로 통일한다.
- `docker context`로 Mac에서 원격 데몬을 조작하는 방식은 쓰지 않는다. compose가 바인드 마운트 경로를 Mac 쪽 절대 경로로 풀어 보내기 때문이다.

## Docker 네트워크 구성

### 전체 그림
```
실행 호스트 (기본: Mac의 Docker Desktop Linux VM / 선택: x86_64 Linux 머신)
  |
  +-- Docker 데몬
        |
        +-- 사용자 정의 bridge 네트워크 "dds-net" (172.28.0.0/24)
              |
              +-- cyclone   172.28.0.10 (eth0)   cyclone_shape
              +-- fastdds   172.28.0.11 (eth0)   fastdds_shape

Mac: Wireshark로 pcap 파일을 연다 (바인드 마운트로 바로 보인다)
```

### 왜 이렇게 구성하는가
1. **사용자 정의 bridge 네트워크를 쓴다.** 기본 `bridge`(docker0)와 달리 컨테이너 이름으로 서로를 찾을 수 있고(`ping fastdds`), 다른 프로젝트의 컨테이너와 격리된다. 멀티캐스트는 같은 bridge 안에서만 돈다.
2. **IP를 고정한다 (ipam).** 캡처를 읽을 때 172.28.0.10이 Cyclone, .11이 Fast DDS라고 바로 알 수 있다. 컨테이너를 다시 만들어도 IP가 같아서 캡처끼리 비교하기 쉽다.
3. **포트를 publish하지 않는다 (`ports:` 없음).** 통신은 컨테이너끼리만 한다. 호스트는 이 네트워크에 참여하지 않으며, `-p 7400:7400` 같은 설정은 쓸모가 없고 오해만 만든다.
4. **`network_mode: host`를 쓰지 않는다.** 두 컨테이너가 호스트 네임스페이스를 공유하면 "한 호스트 안의 두 프로세스"가 되어 participantId가 0, 1로 갈리고 포트도 달라진다. 두 호스트 사이 통신이라는 관찰 목적과 멀어진다. 또 Linux 머신에서는 실제 NIC로 멀티캐스트가 새어 나가고, Mac의 Docker Desktop에서는 host 모드가 4.34부터 켜는 선택 기능이며 Mac이 아니라 VM 네트워크에 붙는다 [추정].
5. **캡처는 컨테이너 안에서 한다.** 호스트가 Linux든 Mac이든 같은 절차로 재현되게 하기 위해서다. Mac의 경우 VM 안 bridge 트래픽은 Mac에서 아예 보이지 않는다. 컨테이너에 `tcpdump`를 넣고 `docker compose exec`로 실행한 뒤 pcap 파일을 볼륨으로 꺼낸다.

### 멀티캐스트가 컨테이너 사이를 넘어가는 원리
- 사용자 정의 bridge 네트워크는 호스트 커널 (Mac이면 VM 안)의 Linux bridge (가상 L2 스위치)다. 각 컨테이너의 `eth0`은 veth 쌍으로 이 스위치의 포트에 꽂혀 있다.
- DDS 구현이 239.255.0.1 그룹에 가입하면 커널이 IGMP Membership Report를 `eth0`으로 내보낸다. Linux bridge는 멀티캐스트 스누핑(multicast snooping)이 켜져 있으면 이 보고를 보고 "이 포트는 이 그룹을 원한다"고 기억해 그 포트로 전달하고 (학습이 만료되면 다시 모든 포트로 흘려보낸다), 꺼져 있으면 처음부터 모든 포트로 흘려보낸다. 어느 쪽이든 두 컨테이너 사이에서는 전달된다.
- 멀티캐스트 TTL 기본값 1이어도 같은 L2 세그먼트라 라우터를 지나지 않으므로 문제없다.
- 반대로 다른 bridge 네트워크, 호스트의 물리 NIC 바깥, (Mac의 경우) VM 바깥으로는 넘어가지 않는다. 그래서 캡처는 반드시 같은 네트워크 안에서 한다.
- 디버깅 순서: (1) `ip maddr show eth0`로 가입 여부, (2) tcpdump에서 `igmp` 패킷, (3) socat으로 전달 여부. 이 셋이 되면 네트워크는 정상이고 문제는 DDS 쪽이다.

### RTPS 트래픽이 이 네트워크에서 흐르는 방식
- **참가자 발견 (SPDP)**: 각 컨테이너가 239.255.0.1:7400 (도메인 0)으로 주기적으로 멀티캐스트한다. 주기는 Cyclone 약 8초 (lease 10초의 80%), Fast DDS 3초 (lease 20초).
- **인터페이스 선택 문제가 없다.** 컨테이너에는 `lo`와 `eth0`뿐이라 DDS 구현이 어느 인터페이스로 멀티캐스트할지 고민할 일이 없다. (Mac에서 직접 띄우면 en0, bridge100, utun 등 여러 인터페이스 때문에 자주 꼬인다. Docker를 기본으로 정한 이유 중 하나다.)
- **유니캐스트 (SEDP, 사용자 데이터, HEARTBEAT / ACKNACK)**: SPDP 안에 각 컨테이너가 자기 eth0 주소(172.28.0.x)와 유니캐스트 포트를 로케이터로 광고한다. 같은 bridge라 NAT 없이 직접 도달한다.
  - Fast DDS는 규격 기본 공식대로 7410 (메타트래픽), 7411 (사용자 데이터)을 쓴다.
  - Cyclone은 멀티캐스트 발견을 쓸 때 `Discovery/ParticipantIndex`가 `none`과 같아져 **유니캐스트 포트를 임의로** 고른다. 7410이 아니라고 이상한 것이 아니다. SPDP의 PID_METATRAFFIC_UNICAST_LOCATOR / PID_DEFAULT_UNICAST_LOCATOR 값으로 확인한다.
- **공유 메모리 전송**: Fast DDS 실행 파일에는 공유 메모리 전송이 들어 있고 SPDP에 그 로케이터(벤더 전용 kind)를 실을 수 있다. 하지만 Cyclone과 Fast DDS 사이에서는 애초에 공유 메모리로 통신할 수 없으므로 이 계획에서는 UDP만 쓰인다. 캡처에서 모르는 kind의 로케이터가 보이면 이것이다.
- **어느 쪽에서 캡처해도 RTPS는 전부 보인다.** 참가자가 둘뿐이라 한 컨테이너의 `eth0`에서 캡처하면 양방향 유니캐스트와 양쪽 멀티캐스트가 모두 잡힌다. 기본은 **구독자 쪽**에서 캡처한다 (lease-expiry 시나리오에서 발행자 프로세스를 죽여도 캡처가 살아 있어야 하므로). 예외: 상대 컨테이너의 IGMP 가입 보고는 bridge 스누핑이 켜져 있으면 전달되지 않아 안 보일 수 있다. IGMP는 자기 쪽 것만 본다.

### compose 설계
파일은 [docker/compose.yaml](../../docker/compose.yaml). 요점만 적는다.
- 프로젝트 이름 `mdds`, 네트워크 이름 `dds-net`을 고정한다 (네트워크 `name:`이 없으면 compose가 프로젝트 이름을 붙여 `mdds_dds-net`이 된다).
- 서비스 `cyclone` (이미지 `mdds/cyclone-shape:11.0.1`, 172.28.0.10)과 `fastdds` (이미지 `mdds/fastdds-shape:3.6.1`, 172.28.0.11). `hostname`도 서비스 이름과 같게 고정한다.
- 두 서비스 모두 `init: true` (PID 1이 고아 프로세스를 거둬 좀비가 pgrep에 잡히지 않게), `cap_add: [NET_RAW]` (tcpdump용. 기본 권한에 포함되지만 의도를 남긴다), `../tests/fixtures/phase00:/captures` 바인드 마운트.
- 컨테이너는 이미지의 `CMD ["sleep", "infinity"]`로 떠 있고, 실제 명령은 `docker compose exec`로 실행한다. 이렇게 하면 **캡처를 먼저 켜고 나서 DDS를 시작**할 수 있어 첫 IGMP 가입과 첫 SPDP 패킷부터 잡힌다. 또한 발행자 프로세스만 죽이고 컨테이너는 살려 둘 수 있다.
- **이미지를 벤더마다 나눈다** (2026-10-28 결정). 한 이미지에 실행 파일 둘을 넣으면 컨테이너마다 쓰지 않는 실행 파일이 들어 있어 헷갈린다. 나누면 "컨테이너 = 벤더"가 되고 한쪽만 다시 빌드할 수도 있다. 대가로 베이스 이미지와 shape_main 소스 커밋을 두 Dockerfile에 같은 값으로 유지해야 한다.

### 시나리오 실행 흐름 (`run-scenario.sh`)
시나리오마다 아래 변수를 정한다. 값은 시나리오 표에 있다.
```
NAME              시나리오 이름 (파일 이름에 쓴다)
SUB, PUB          구독 / 발행 컨테이너 이름 (cyclone 또는 fastdds)
SUB_BIN, PUB_BIN  cyclone_shape 또는 fastdds_shape
COMMON            양쪽 공통 옵션 (-d, -t, -c, -x, -r / -b)
PUB_ONLY          발행 전용 옵션 (--write-period, --num-iterations, --final-instance-state, --additional-payload-size)
SUB_ONLY          구독 전용 옵션 (보통 없음)
END               발행자 종료 방식: wait (스스로 끝남) 또는 kill9 (10초 뒤 SIGKILL)
AFTER             발행자 종료 후 관찰 대기 (초)
```
```
cd "$(dirname "$0")"            # compose.yaml이 있는 docker/ 에서 실행
X="docker compose exec -T"      # 스크립트에서는 TTY가 없으므로 -T 필수

 1. $X -d $SUB tcpdump -i eth0 -U -Z root -w /captures/$NAME.pcap "udp or igmp"
 2. sleep 1
 3. $X -d $SUB sh -c "stdbuf -oL $SUB_BIN -S $COMMON $SUB_ONLY > /captures/$NAME.sub.log 2>&1"
 4. sleep 3                      # 구독자가 먼저 SPDP를 내보내게 한다
 5. $X -d $PUB sh -c "stdbuf -oL $PUB_BIN -P $COMMON $PUB_ONLY > /captures/$NAME.pub.log 2>&1"
 6. END=wait : while $X $PUB pgrep -x $PUB_BIN >/dev/null; do sleep 1; done
    END=kill9: sleep 10; $X $PUB pkill -9 -x $PUB_BIN
 7. sleep $AFTER                 # dispose 전파나 lease 만료를 관찰할 시간
 8. $X $SUB pkill -INT -x $SUB_BIN || true ; sleep 2    # 구독자 정상 종료
 9. $X $SUB pkill -TERM -x tcpdump || true ; sleep 1    # 파일이 닫히도록 SIGTERM
10. $X cyclone pgrep -x cyclone_shape; $X fastdds pgrep -x fastdds_shape   # 컨테이너마다 자기 벤더 실행 파일만 있다
    # 아무것도 안 나와야 정상. pgrep은 못 찾으면 종료 코드 1이므로 set -e 환경에서는 결과를 따로 판정한다
```
- **종료 신호는 SIGINT다.** shape_main은 SIGINT 핸들러만 설치하므로 SIGTERM을 보내면 정리 없이 죽어 dispose / unregister / SEDP 제거가 나가지 않는다. 정상 종료는 `--num-iterations N`으로 스스로 끝내거나 `pkill -INT`로 한다.
- `pkill -x` / `pgrep -x`는 프로세스 이름 전체 일치다. `-f`를 쓰면 `sh -c "... cyclone_shape ..."` 래퍼 셸까지 잡힌다. 실행 파일 이름이 15자 이하(`cyclone_shape` 13자, `fastdds_shape` 13자)라 `-x`가 된다.
- `exec -d`로 띄운 프로세스는 핸들이 없으므로 6단계처럼 `pgrep` 폴링으로 종료를 기다린다.
- **stdout은 파일로 받는다.** `exec -d`로 띄운 프로세스의 출력은 어디에도 남지 않는다. `stdbuf -oL`이 없으면 파일 리다이렉트 때 블록 버퍼링되어 SIGKILL 시 로그가 비어 있다.
- `tcpdump -U`는 패킷마다 바로 쓰기 (버퍼 손실 방지), `-Z root`는 tcpdump가 권한을 낮추면서 바인드 마운트에 쓰지 못하는 문제 방지.
- `tcpdump`는 pcap 형식으로 쓴다. Wireshark에서 바로 열린다. 저장소에는 `.pcap` 그대로 둔다.
- 캡처는 구독자 컨테이너에서 한다. 발행자를 바꾸는 시나리오에서는 캡처 컨테이너도 바뀐다.

### 멀티캐스트 검증 명령 (Step 4)
```
# 받는 쪽 (fastdds 컨테이너, 터미널 1). 가입 인터페이스는 이름보다 IP로 적는 쪽이 socat 버전을 타지 않는다
docker compose exec fastdds socat -u UDP4-RECV:7400,ip-add-membership=239.255.0.1:172.28.0.11,reuseaddr STDOUT

# 가입 확인 (fastdds 컨테이너, 터미널 2)
docker compose exec fastdds ip maddr show eth0        # 239.255.0.1 이 보여야 한다

# 보내는 쪽 (cyclone 컨테이너, 터미널 2)
docker compose exec cyclone sh -c 'echo hello | socat -u STDIN UDP4-DATAGRAM:239.255.0.1:7400'
```
터미널 1에 `hello`가 찍히면 bridge가 멀티캐스트를 전달하는 것이다. 이 검증은 DDS와 무관하게 네트워크만 확인하므로, 실패하면 원인이 네트워크에 있다고 확정할 수 있다.

## 시나리오 표
옵션은 OMG dds-rtps README (2026-10-28 확인)의 shape_main 옵션이다. Step 3에서 `-h`로 실행 파일과 같은지 재확인한다.

- COMMON 기본값: `-d 0 -t Square -c BLUE -x 1`에 표의 신뢰성 옵션을 더한다. xcdr2 시나리오만 `-x 2`. 데이터 표현을 명시하는 이유는 실행 파일의 기본값을 모르기 때문이다.
- PUB_ONLY 기본값: `--write-period 100 --num-iterations 100` (10 Hz로 약 10초 보내고 스스로 끝난다). 표의 값은 여기에 더하거나 대체한다.
- SUB_ONLY: 모든 시나리오에서 없음.

| 이름 | 발행 (PUB) | 구독 (SUB) | COMMON 추가 | PUB_ONLY 변경 | END | AFTER | 관찰 목표 |
|---|---|---|---|---|---|---|---|
| be-cyclone-pub | cyclone | fastdds | `-b` | 기본값 | wait | 5 | best-effort DATA, HEARTBEAT 없음 |
| be-fastdds-pub | fastdds | cyclone | `-b` | 기본값 | wait | 5 | 벤더별 차이 비교 |
| rel-cyclone-pub | cyclone | fastdds | `-r` | 기본값 | wait | 5 | HEARTBEAT / ACKNACK 왕복, GAP |
| rel-fastdds-pub | fastdds | cyclone | `-r` | 기본값 | wait | 5 | 위와 같음 |
| dispose-on-exit | cyclone | fastdds | `-r` | 기본값 + `--final-instance-state d` | wait | 10 | DATA에 STATUS_INFO dispose 비트, 이어서 unregister, SEDP의 writer 제거. 구독 로그의 instance state 변화 |
| lease-expiry | cyclone | fastdds | `-r` | `--write-period 100` 만 (`--num-iterations` 없음) | kill9 | 25 (Cyclone lease 10초 + 여유) | dispose 없이 사라진 참가자를 구독자가 lease 만료로 정리. 증거는 캡처의 마지막 SPDP 시각과 그 뒤 구독자의 동작 (ACKNACK 중단 등). 구독 로그에 liveliness 변화가 찍히는지는 미확인이므로 Step 3에서 `-h`와 소스로 확인한다 |
| large-sample | cyclone | fastdds | `-r` | 기본값 + `--additional-payload-size 100000`, `--num-iterations 30` | wait | 5 | DATA_FRAG, HEARTBEAT_FRAG, NACK_FRAG (두 벤더 사이). IP 단편화도 섞이므로 Wireshark에서 IP 재조립 설정을 켜고 본다 |
| xcdr2 | cyclone | fastdds | `-r -x 2` (`-x 1` 대체) | 기본값 | wait | 5 | 캡슐화 식별자 0x0007 (CDR2_LE) 등 XCDR2 페이로드 |

- lease 기본값: Cyclone `Discovery/LeaseDuration` 10초, Fast DDS 20초 (공식 문서로 확인). 구독자는 상대가 SPDP로 광고한 lease 값을 쓰므로, Cyclone 발행자를 죽이면 Fast DDS 구독자는 약 10초 뒤에 정리한다. 캡처의 PID_PARTICIPANT_LEASE_DURATION 값으로 재확인한다.
- 각 시나리오는 한 번에 참가자 둘만 띄운다. 실행 흐름 10번의 `pgrep`이 비어 있는지 확인하고 시작한다.
- 선택: dispose-on-exit의 `d`를 `u`로 바꾼 unregister-only 캡처, 발행 `-r` + 구독 `-b` 같은 QoS 불일치 캡처 (REQUESTED_INCOMPATIBLE_QOS 관찰)는 시간이 남으면 추가한다.

## 예상 문제와 대응
| 증상 | 원인 후보 | 대응 |
|---|---|---|
| 이미지 빌드 실패, 실행 파일 라이브러리 오류 | 빌드 쪽 문제 | [environment/vendor-build.md](../environment/vendor-build.md)의 "빌드 문제와 대응" |
| socat 멀티캐스트가 안 넘어감 | bridge가 멀티캐스트를 전달하지 않음 (Linux 머신에서는 드물다. Mac의 Docker Desktop VM에서 더 흔하다) | (1) `ip maddr`와 tcpdump의 igmp로 가입은 됐는지 확인. (2) Linux 머신: `cat /sys/class/net/br-<id>/bridge/multicast_snooping` 확인, 1이면 `echo 0 | sudo tee` 로 0으로 바꿔 시험. (3) Mac: Docker Desktop 버전과 네트워킹 설정 확인. (4) 대안 A: 컨테이너 하나에서 두 프로세스를 띄운다 (loopback 멀티캐스트는 항상 된다. 단 "한 호스트"가 되어 participantId가 갈린다). (5) 대안 B (Mac): OrbStack이나 Lima 같은 다른 Linux VM 환경. |
| 통신은 되는데 캡처가 비어 있음 | 캡처가 늦게 시작됨, 인터페이스 이름이 eth0가 아님, tcpdump가 파일에 못 씀 | `ip addr`로 인터페이스 확인. 실행 흐름의 순서를 지킨다. `-Z root`를 확인한다 |
| 로그 파일이 비어 있음 | 블록 버퍼링 | `stdbuf -oL`을 확인한다 |
| `pkill`이 프로세스를 못 찾음 | 15자 넘는 실행 파일 이름 | 짧은 이름으로 복사했는지 확인한다. `-x`는 이름 전체 일치다 |
| 스크립트에서 "input device is not a TTY" | `docker compose exec`에 `-T` 누락 | `-T`를 붙인다 |
| SPDP는 보이는데 매칭이 안 됨 | QoS 불일치, 데이터 표현 불일치 (`-x`) | 양쪽 옵션을 같게 맞춘다. 이것 자체도 좋은 캡처다 |
| 유니캐스트 목적지가 엉뚱한 IP | 컨테이너에 인터페이스가 여러 개 | `docker network inspect`로 확인. compose에서 네트워크를 하나만 붙인다 |
| Cyclone 유니캐스트 포트가 7410이 아님 | 정상. Cyclone은 멀티캐스트 발견 시 임의 포트 | SPDP 로케이터 값으로 확인 |

## 프로젝트 규약 (제안, Step 8에서 확정)
- 라이선스: `MIT OR Apache-2.0` 이중 라이선스 (2026-10-28 결정). 파일: `LICENSE-MIT`, `LICENSE-APACHE`
  - Rust 생태계 표준 관례 (Rust 자체, tokio, serde 등). 사용자가 둘 중 하나를 고른다.
  - Apache-2.0은 특허 허락 조항이 있고, MIT는 GPLv2 프로젝트와도 섞어 쓸 수 있다.
  - 참고 구현 중 Cyclone DDS는 EPL-2.0이다. 읽고 배우는 것은 괜찮지만 코드를 그대로 옮겨 오지 않는다.
- Rust: edition 2024. 시작 시점의 stable 버전을 `rust-toolchain.toml`에 고정하고 그 버전을 MSRV로 삼는다.
- 형식과 정적 검사: `rustfmt` 기본 설정, `cargo clippy -- -D warnings` 통과
- 테스트 배치: 단위 테스트는 모듈 안 `#[cfg(test)]`, pcap 기반 골든 테스트와 상호운용 테스트는 `tests/`
- 캡처 파일: `tests/fixtures/phase00/<시나리오>.pcap`과 `.pub.log`, `.sub.log`. README에 실행 명령, 벤더 버전, 빌드 소스 커밋 해시를 적는다. 파일 하나가 수 MB를 넘으면 (large-sample) 필요한 프레임만 `editcap`으로 잘라 저장한다.
- 코드 주석에 규격 근거를 적는다. 형식: `// RTPS 2.5 Sec. 9.4.5.3`
- 문서 / 커밋 / 주석은 ASCII와 한글만 쓴다 (전역 지침).
- CI: 원격 저장소를 연결하면 GitHub Actions로 fmt, clippy, test를 돌린다.
- 프로젝트 `CLAUDE.md`: 위 규약과 문서 갱신 규칙을 적는다.

## 참고 코드
- Cyclone DDS: RTPS 상태 머신, 디스커버리 로직 읽기 좋음
- RustDDS: Rust식 구조
- dust_dds: XTypes, 워크스페이스 구성

## 완료 기준
- Mac의 Docker Desktop (또는 어떤 Docker 호스트)에서 `docker compose up -d` 와 `run-scenario.sh <이름>` 만으로 누구나 같은 캡처를 재현할 수 있다.
- 시나리오 표의 캡처 8개와 로그가 저장소에 있고, README에 주목할 프레임이 적혀 있다.
- 캡처에서 SPDP / SEDP / DATA / HEARTBEAT / ACKNACK / STATUS_INFO(dispose) / DATA_FRAG를 Wireshark로 식별했고, 두 벤더의 차이를 메모했다.
- Cargo 워크스페이스가 빌드되고 clippy, fmt를 통과한다.
- 프로젝트 규약이 확정되어 이 문서와 `CLAUDE.md`에 반영되어 있다.

## 검토 기록
- 2026-10-28 사용자 결정: 실행 호스트를 Mac (arm64) Docker Desktop으로 바꿈. x86_64 머신 절은 "선택"으로 줄임. 같은 날 첫 빌드에서 Cyclone 11.0.1로는 C판 shape_main이 링크되지 않아 Cyclone을 master 커밋 f2aaccb4로 고정 (진행 기록 참고).
- 2026-10-28 사용자 결정: OMG 배포 실행 파일 대신 Docker 안에서 소스로 빌드 (빌드 설계는 environment/vendor-build.md로 분리). zip 다운로드 / 해시 고정 / amd64 플랫폼 지정 / Rosetta 관련 내용 제거. x86_64 머신은 빌드 부하 때문에 계속 기본 호스트로 둔다.
- 2026-10-28 3회차 (Sonnet 검토자, x86_64 머신 절과 2회차 반영 후 일관성). 지적 9건, 전부 반영.
  - 높음: 원격 git 저장소가 없어 머신에서 clone 불가 -> Step 1에 원격 생성과 push 추가, rsync 대안 명시
  - 중간: compose 네트워크 이름이 `docker_dds-net`이 됨 -> `name: dds-net`. docker context의 바인드 마운트 경로 설명 정정 (Mac 절대 경로 기준). docker-env.md의 Mac 전제 정리
  - 낮음: `$NAME` 변수 누락, Mac 중심 표현 정리, 스누핑 설명 완화, 머신 준비 패키지 (openssh-server, docker 패키지 이름, newgrp), root 소유 pcap 안내, 검토 기록의 `-f` 표기
- 2026-10-28 2회차 (Sonnet 검토자, 1회차 반영 확인 + 새 지적). 1회차 12건 모두 반영 확인. 새 지적 9건, 전부 반영.
  - 높음: Dockerfile에 `ca-certificates curl unzip` 누락 -> 추가, 해시 검증과 설치 단계 명시
  - 높음: `$OPTS` / `$PUB_OPTS` 구분 불명, `-t` 중복 -> COMMON / PUB_ONLY / SUB_ONLY로 분리하고 표를 다시 씀
  - 중간: 발행자 종료 대기 방법 없음 -> `pgrep -x` 폴링. `init: true` 추가 (좀비 방지). `exec -T`, `|| true`, 스크립트 `cd`
  - 낮음: `image:` 태그 고정, lease 로그 가정 완화, IGMP 가시성 주석, IP 재조립 메모, `up -d --build` 지시, socat 가입을 IP로
  - 같은 날 사용자 요청으로 실행 호스트를 별도 x86_64 Linux 머신으로 바꿈 (Mac 에뮬레이션은 대안으로 강등).
- 2026-10-28 1회차 (Opus 검토자, dds-rtps README와 `srcCxx/shape_main.cxx`, Cyclone / Fast DDS 문서, 릴리스 zip을 직접 확인). 지적 12건, 전부 반영.
  - 높음: SIGTERM으로는 정상 종료가 안 됨 (SIGINT 핸들러만 있음) -> SIGINT / `--num-iterations`, dispose는 `--final-instance-state d`
  - 높음: `exec -d` 출력이 남지 않음 -> `stdbuf -oL`로 파일 저장, 구독자 종료 단계 추가
  - 높음: 실행 파일 이름이 15자를 넘어 `pkill shape_main` 실패 -> 짧은 이름으로 복사 (2회차에서 `-f` 대신 `-x`로 확정)
  - 높음: 이미지에 ping / pkill / ip 없음 -> 패키지 추가
  - 높음: `--additional-payload-size`가 있어 ddsperf와 arm64 이미지 불필요 -> 삭제, large-sample을 두 벤더 시나리오로
  - 중간: Cyclone은 유니캐스트 포트를 임의로 씀, lease 기본값 (Cyclone 10초, Fast DDS 20초) 확정, 공유 메모리 설명 수정, 멀티캐스트 원리와 디버깅 순서 보강
  - 낮음: tcpdump `-U -Z root`, cap_add 정리, zip 해시 고정 필수화, `-x` 명시와 `--write-period` / `--num-iterations`로 결정적 실행

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
- 2026-10-28: Dockerfile 초안 작성 (아직 빌드하지 않음, `docker build --check`와 `docker compose config`만 통과). 처음에는 `docker/shape-main/` 이미지 하나였으나 같은 날 벤더별 `docker/cyclone/`, `docker/fastdds/`로 나눴다 (compose 설계 절 참고). `docker/compose.yaml`, `docker/README.md`도 작성.
  - 태그별 커밋 해시를 `git ls-remote`로 확인해 `ARG`로 고정했다. 다섯 태그 모두 lightweight 태그라 태그 해시가 곧 커밋 해시다. clone 후 해시가 다르면 빌드가 멈춘다.
    - cyclonedds 11.0.1 `e54e991f`, Fast-DDS v3.6.1 `4e81e8b7`, Fast-CDR v2.3.5 `7d33a3b5`, foonathan_memory_vendor v1.4.1 `347cb675`, Fast-DDS-Gen v4.3.0 `cc0072b8`
  - 베이스 이미지는 `ubuntu:24.04` 대신 날짜 태그와 다이제스트로 고정했다: `ubuntu:noble-20261024.1@sha256:90b3a804...`. 멀티 아키텍처 인덱스라 amd64와 arm64가 같은 값을 쓴다.
  - `idlc` 경고 옵션 철자는 `-Wno-implicit-extensibility`로 확정했다. Cyclone 11.0.1 `Generate.cmake`가 `WARNINGS` 값을 `-W<값>`으로 넘긴다.
  - 커밋 526dc56의 `srcC/`에 `shape.idl`, `shape_main.c`, `shape_configurator_cyclone_dds.h`가 있다. 헤더가 `shape.h`와 `dds/dds.h`를 include하므로 `-I.`와 `-I/opt/cyclone/include`면 된다.
  - Fast DDS `fast-dds-cmake`는 생성 코드를 `srcCxx/GeneratedCode`에 쓴다 (`${CMAKE_SOURCE_DIR}/../GeneratedCode`).
- 2026-10-28: 실행 호스트를 Mac (arm64) Docker Desktop으로 바꾸고 첫 빌드. 두 이미지 모두 성공.
  - 첫 시도는 Cyclone 11.0.1에서 링크 실패: `undefined reference to dds_take_next_instance / dds_read_next_instance`. C판 shape_main은 11.0.1 이후 master에만 있는 API를 쓴다. Cyclone을 master `f2aaccb4` (2026-06-02, 버전 번호 11.0.1)로 고정했다. 근거는 environment/vendor-build.md.
  - Docker Desktop 메모리 8 GB에서 빌드 중 데몬 연결이 끊긴 적이 있다 (`rpc error ... EOF`, 메모리를 16 GB로 올린 재시작과 겹침). 16 GB, `--build-arg JOBS=6`으로 성공했다.
  - 빌드 시간: 약 3분 (189초, 일부 apt 단계는 캐시됨). 단계별로 Cyclone 라이브러리 12초, Fast DDS 라이브러리 91초, Fast-DDS-Gen (Gradle) 27초, apt 설치가 각 수십 초. 수십 분이라던 예상보다 훨씬 빠르다 (Apple Silicon 10코어).
  - 이미지 크기: `mdds/cyclone-shape:11.0.1-f2aaccb4` 113 MB, `mdds/fastdds-shape:3.6.1` 124 MB.
  - `ldd`: 둘 다 `not found` 없음. cyclone_shape는 `/opt/cyclone/lib/libddsc.so.11`, libssl3, libcrypto3. fastdds_shape는 Fast DDS가 정적으로 들어가고 libtinyxml2.so.10, libssl3, libstdc++만 공유. 걱정했던 TinyXml2 대소문자 문제는 생기지 않았다.
  - `-h`: 두 실행 파일의 옵션 목록이 같고 README의 옵션을 모두 포함한다. README에 없던 옵션도 보인다: `-R`, `--read-period`, `--time-filter`, `--lifespan`, `--num-instances`, `--num-topics`, `--access-scope`, `--coherent`, `--ordered`, `--coherent-sample-count`, `--take-read` (take_next_instance 대신 take/read 사용).
  - Fast DDS 원래 실행 파일 이름은 `eprosima_fastdds-3.6.1.0_shape_main_linux`다 (`fastdds_VERSION`이 4자리). OMG 릴리스 자산 이름 `eprosima_fastdds-3.6.1`과 끝자리가 다르다.
