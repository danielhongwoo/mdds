# Cyclone DDS / Fast DDS / shape_main 빌드

[roadmap](../../docs-for-agent/roadmap.md) | 사용처: [chapter 1-4 노트](../../docs-for-agent/notes/01-04-environment-and-captures.md) (이후 캡처 추가와 상호운용 시험에도 같은 이미지를 쓴다)

OMG 상호운용 테스트 프로그램 shape_main과, 그것이 링크하는 두 벤더의 DDS 라이브러리를 Docker 이미지 안에서 소스로 빌드하는 방법을 정리한다. 실행 호스트와 네트워크는 [docker-network.md](docker-network.md), 시나리오 실행은 [chapter 1-4 노트](../../docs-for-agent/notes/01-04-environment-and-captures.md)에 있다.

## 왜 소스에서 빌드하는가
- OMG 릴리스의 실행 파일은 x86-64 Linux용뿐이고, 빌드 환경과 정확한 소스 커밋이 드러나지 않는다 (zip 안 파일 날짜가 릴리스일보다 늦어 자산이 교체된 흔적도 있다).
- 소스에서 빌드하면 아키텍처에 묶이지 않고, 어떤 커밋으로 만든 실행 파일인지 Dockerfile에 그대로 남는다. 나중에 shape_main이나 벤더 라이브러리에 로그를 넣어 동작을 들여다볼 수도 있다 (학습 목적에 맞다).
- 대가는 첫 빌드 시간이다. Docker 레이어 캐시 덕분에 두 번째부터는 바뀐 단계만 다시 빌드한다.

## 소스와 버전
| 구성 요소 | 저장소 | 고정 버전 | 비고 |
|---|---|---|---|
| shape_main 소스 | omg-dds/dds-rtps | 커밋 `526dc56eed8b1965a81ba5b7a644c4489337fe0a` (2026-06-09, "Cyclone dds fixes (#87)") | 아래 "릴리스 태그를 쓰지 않는 이유" 참고. 두 벤더 모두 이 커밋을 쓴다 |
| Cyclone DDS | eclipse-cyclonedds/cyclonedds | master 커밋 `f2aaccb4873c4bc33d59ee45c8568a4653900f78` (2026-06-02, 버전 번호 11.0.1) | C 라이브러리 `ddsc`와 IDL 컴파일러 `idlc`. 태그 11.0.1이 아닌 이유는 아래 "Cyclone을 master 커밋으로 고정하는 이유" |
| Fast DDS | eProsima/Fast-DDS | 태그 v3.6.1 | 아래 세 개는 Fast DDS v3.6.1의 `fastdds.repos`에 고정된 버전 |
| Fast-CDR | eProsima/Fast-CDR | 태그 v2.3.5 | |
| foonathan_memory_vendor | eProsima/foonathan_memory_vendor | 태그 v1.4.1 | |
| Fast-DDS-Gen | eProsima/Fast-DDS-Gen | 태그 v4.3.0 | IDL -> C++ 코드 생성기. Java (Gradle), git 서브모듈 필요 |

- Fast DDS 3.6.1은 OMG 릴리스 v2.1.2026에 올라간 실행 파일과 같은 버전이다. Cyclone은 버전 번호만 같고 (11.0.1) 실제로는 그 뒤의 master다.
- 태그는 옮겨질 수 있으므로 각 저장소의 커밋 해시를 Dockerfile의 `ARG`로 고정하고, clone 후 해시가 다르면 빌드를 멈춘다. 해시 목록은 `docker/README.md`.

### Cyclone을 master 커밋으로 고정하는 이유
- 태그 11.0.1로 빌드하면 (2026-10-28 확인) `shape_main.c`가 링크되지 않는다: `undefined reference to dds_take_next_instance`, `dds_read_next_instance`.
- 이 두 API는 Cyclone master에 2026-03-31 (#2376, `fa4c9b98`)에 추가되었고 2026-05-25 (`a47f209f`)에 버그가 고쳐졌다. 2026-10-28 현재 이것을 포함한 릴리스 태그는 없다 (최신 태그 11.0.1).
- dds-rtps의 `srcC/shape_main.c`는 처음 들어온 #81 (2026-04-20)부터 이 API를 쓴다. 그래서 C판 shape_main은 처음부터 master Cyclone을 전제로 한다.
- 고른 커밋 `f2aaccb4`는 dds-rtps #87 (2026-06-09) 직전의 master (2026-06-02)다. 위 버그 수정을 포함하고, 태그 11.0.1보다 121커밋 앞이다. CMake 버전 번호는 여전히 11.0.1이라 `CycloneDDS_VERSION`도 11.0.1로 나온다.
- 따라서 OMG 릴리스의 `eclipse_cyclone-11.0.1` 실행 파일 (2026-02-27)은 이 `srcC/` 소스로 만든 것일 수 없다. 무엇으로 만들었는지는 확인하지 못했다 [추정 불가].
- 이미지 태그는 `mdds/cyclone-shape:11.0.1-f2aaccb4`로 커밋을 드러낸다.

### 릴리스 태그를 쓰지 않는 이유
- dds-rtps 태그 v2.1.2026 (2026-02-27)에는 Cyclone용 C판 소스 `srcC/`가 **없다**. 태그에는 `srcCxx/`의 Fast DDS, InterCom, OpenDDS용 CMake만 있다.
- `srcC/` (C판 shape_main과 `cyclone-dds-cmake`)는 2026-04-20 (#81)에 추가되고 2026-06-09 (#87)에 수정되었다. 그래서 Cyclone은 #87 커밋 이후의 소스로만 빌드할 수 있다.
- 두 벤더가 같은 shape_main 소스 커밋을 쓰도록 Fast DDS도 같은 커밋으로 빌드한다. 결과적으로 shape_main 소스는 릴리스 실행 파일과 다를 수 있다 (릴리스 이후 수정분 포함).

## Dockerfile 구성 (벤더별 이미지, 멀티스테이지)
벤더마다 Dockerfile과 이미지를 따로 둔다: `docker/cyclone/Dockerfile` -> `mdds/cyclone-shape:11.0.1-f2aaccb4`, `docker/fastdds/Dockerfile` -> `mdds/fastdds-shape:3.6.1`. 두 파일 모두 `src` -> `build` -> `runtime` 세 단계이며, 아래 개요의 `cyclone-build` / `fastdds-build`가 각 파일의 `build` 단계다. 실제 파일이 기준이고 아래는 구조 요약이다.

공통 (두 Dockerfile에 같은 값으로 둔다)
- 베이스 이미지: `ubuntu:24.04` 대신 날짜 태그와 다이제스트로 고정 (`ubuntu:noble-20261024.1@sha256:90b3a804...`). 멀티 아키텍처 인덱스라 amd64와 arm64가 같은 값을 쓴다.
- 각 저장소를 태그로 clone한 뒤 `ARG`의 커밋 해시와 다르면 빌드를 멈춘다. 해시 목록은 `docker/README.md`.
- runtime 단계에는 두 이미지 모두 캡처/진단 도구 (`tcpdump socat iputils-ping procps iproute2`)를 넣는다.

```
ARG DDS_RTPS_COMMIT=526dc56eed8b1965a81ba5b7a644c4489337fe0a

stage src             (BASE_IMAGE, 두 Dockerfile에 같은 내용)
  apt: git ca-certificates
  git clone dds-rtps && git checkout $DDS_RTPS_COMMIT

stage cyclone-build   (BASE_IMAGE)
  apt: build-essential cmake git ca-certificates libssl-dev
  git fetch --depth 1 cyclonedds $CYCLONE_COMMIT   (태그가 없어 커밋 하나만 받는다)
    cmake -DCMAKE_INSTALL_PREFIX=/opt/cyclone -DBUILD_IDLC=ON
          -DBUILD_EXAMPLES=OFF -DBUILD_TESTING=OFF
    cmake --build . --target install
  COPY --from=src dds-rtps/srcC -> /work/srcC
  cd /work/srcC
    /opt/cyclone/bin/idlc -Wno-implicit-extensibility shape.idl        -> shape.c, shape.h
    gcc -O2 -DCYCLONE_DDS -I. -I/opt/cyclone/include shape_main.c shape.c
        -L/opt/cyclone/lib -lddsc -lm -o cyclone_shape

stage fastdds-build   (BASE_IMAGE)
  apt: build-essential cmake git ca-certificates libasio-dev libtinyxml2-dev libssl-dev openjdk-17-jdk-headless
  foonathan_memory_vendor v1.4.1 -> Fast-CDR v2.3.5 -> Fast-DDS v3.6.1 순서로 /opt/fastdds 에 설치
     공통: -DCMAKE_INSTALL_PREFIX=/opt/fastdds -DCMAKE_PREFIX_PATH=/opt/fastdds -DBUILD_SHARED_LIBS=OFF
     Fast-DDS만: -DCOMPILE_TOOLS=OFF -DCOMPILE_EXAMPLES=OFF   (빌드 시간 단축)
  git clone --depth 1 --recurse-submodules --shallow-submodules -b v4.3.0 Fast-DDS-Gen
     ./gradlew assemble          (PATH에 Fast-DDS-Gen/scripts 추가 -> fastddsgen)
  COPY --from=src dds-rtps/srcCxx -> /work/srcCxx
  cmake -S /work/srcCxx/fast-dds-cmake -DCMAKE_PREFIX_PATH=/opt/fastdds && cmake --build .
  결과: eprosima_fastdds-3.6.1.0_shape_main_linux

stage runtime (cyclone 이미지)
  apt: 도구 + libssl3t64
  COPY --from=cyclone-build /opt/cyclone/lib        -> /opt/cyclone/lib   (/etc/ld.so.conf.d 에 등록 후 ldconfig)
  COPY --from=cyclone-build /work/srcC/cyclone_shape -> /usr/local/bin/cyclone_shape

stage runtime (fastdds 이미지)
  apt: 도구 + libtinyxml2-10 libssl3t64
  COPY --from=fastdds-build .../eprosima_fastdds-*_shape_main_linux -> /usr/local/bin/fastdds_shape
       (원래 이름은 /usr/local/share/fastdds_shape.origname 에 남긴다)
```

### 설계 메모
- **Cyclone은 OMG의 `cyclone-dds-cmake`를 쓰지 않고 직접 컴파일한다.** 그 CMakeLists는 `idlc_generate(TARGET shape_lib ...)` 뒤에 `add_library(shape_lib)`를 다시 부른다. Cyclone 11.0.1의 `idlc_generate`가 같은 이름의 타깃을 이미 만들기 때문에 CMake가 "target already exists"로 실패할 가능성이 크다 (검토자가 같은 구조를 재현함, 11.0.1과의 실제 조합은 미확인). 성공하더라도 `libshape_lib.so`가 빌드 디렉터리에만 생겨 runtime 이미지에서 빠진다. `idlc` + `gcc` 두 줄이 더 단순하고, 무엇이 링크되는지도 그대로 보인다.
  - `-DCYCLONE_DDS`를 반드시 정의한다. `shape_main.c`가 이 매크로로 Cyclone용 코드를 고른다.
  - `idlc` 경고 옵션 `-Wno-implicit-extensibility`는 OMG CMakeLists의 `WARNINGS no-implicit-extensibility`와 같은 뜻이다. Cyclone `Generate.cmake`가 `WARNINGS` 값을 `-W<값>`으로 넘기는 것을 확인했다.
  - 직접 컴파일하므로 실행 파일 이름에 버전이 들어가지 않는다. 버전은 `docker/README.md`와 이미지 라벨로 남긴다.
- **Cyclone의 보안 기능은 기본값(켜짐)을 그대로 둔다.** `libssl-dev`를 설치해 `ENABLE_SSL`(기본 AUTO)이 켜지게 한다. OMG 릴리스와 기능을 맞추기 위해서다. 상호운용 시나리오에서는 보안을 쓰지 않는다.
- **Fast DDS는 정적 라이브러리만 설치해서 정적 링크되게 한다.** OMG CMakeLists의 `set(FASTDDS_STATIC ON)`은 Fast DDS 3.6.1의 config 파일이 읽지 않아 효과가 없다. 실제 선택은 "설치된 타깃 파일"로 결정되므로, `-DBUILD_SHARED_LIBS=OFF`로 정적 라이브러리만 설치하면 정적이 선택된다. 결과 실행 파일에 Fast DDS가 들어가고, runtime에는 tinyxml2와 OpenSSL 공유 라이브러리만 있으면 된다.
  - 위험 (첫 빌드에서는 생기지 않음): Fast DDS 정적 config는 `find_dependency(TinyXml2 REQUIRED)`를 호출하는데 대소문자 차이로 찾지 못할 수 있다. 그러면 `-DTinyXML2_DIR`을 주거나, 대안으로 세 저장소를 공유 라이브러리로 빌드하고 `/opt/fastdds/lib`을 runtime에 복사한다.
- **Fast-DDS-Gen은 git 서브모듈 (`thirdparty/idl-parser`)과 Gradle을 쓴다.** `--recurse-submodules`가 없으면 Gradle 빌드 중에 서브모듈을 다시 받으려다 실패한다. Gradle wrapper (9.x)는 JDK 17 이상이 필요하고, 빌드 중 Gradle 배포본과 Maven 의존성을 내려받으므로 네트워크가 필요하다. JDK는 빌드 단계에만 있고 runtime 이미지에는 들어가지 않는다.
- **Fast DDS의 공유 메모리 전송은 기본으로 켜져 있지만** 컨테이너마다 IPC 네임스페이스가 달라 컨테이너 사이에서는 UDP만 쓰인다. 끄지 않아도 된다.
- **짧은 이름으로 복사한다.** OMG 방식의 이름(`eprosima_fastdds-3.6.1.0_shape_main_linux`)은 15자를 넘어 `pkill -x`, `pgrep -x`에 걸리지 않는다 (프로세스 이름 비교는 앞 15자만 쓴다).
- runtime 단계의 패키지 이름 (`libtinyxml2-10`, `libssl3t64`)은 Ubuntu 24.04 기준이다.

## 빌드 결과
2026-10-28, Mac (Apple Silicon 10코어), Docker Desktop 메모리 16 GB, `--build-arg JOBS=6`.
- 빌드 시간: 약 3분 (일부 apt 단계는 캐시). Cyclone 라이브러리 12초, Fast DDS 라이브러리 91초, Fast-DDS-Gen (Gradle) 27초, 나머지는 apt 설치. 느린 머신에서는 Fast DDS 때문에 수십 분 걸릴 수 있다.
- 메모리 8 GB에서는 빌드 중 Docker 데몬 연결이 끊긴 적이 있다 (`rpc error ... EOF`). 8 GB면 `JOBS=4`, 그래도 죽으면 2로 줄인다.
- 이미지 크기: `mdds/cyclone-shape:11.0.1-f2aaccb4` 113 MB, `mdds/fastdds-shape:3.6.1` 124 MB.
- `ldd`: 둘 다 `not found` 없음.
  - cyclone_shape: `/opt/cyclone/lib/libddsc.so.11`, libssl3, libcrypto3
  - fastdds_shape: Fast DDS는 정적으로 들어가고 libtinyxml2.so.10, libssl3, libstdc++만 공유. TinyXml2 대소문자 문제는 생기지 않았다.
- `-h`: 두 실행 파일의 옵션 목록이 같고 dds-rtps README의 옵션을 모두 포함한다. README에 없는 옵션도 있다: `-R`, `--read-period`, `--time-filter`, `--lifespan`, `--num-instances`, `--num-topics`, `--access-scope`, `--coherent`, `--ordered`, `--coherent-sample-count`, `--take-read` (take_next_instance 대신 take/read 사용).
- Fast DDS 원래 실행 파일 이름은 `eprosima_fastdds-3.6.1.0_shape_main_linux`다 (`fastdds_VERSION`이 4자리). OMG 릴리스 자산 이름 `eprosima_fastdds-3.6.1`과 끝자리가 다르다.

## 빌드 후 확인
Dockerfile이나 버전을 바꿔 다시 빌드했을 때 한다. 이미지 빌드가 성공해도 실행할 때 문제가 드러날 수 있어서다. 위 "빌드 결과"는 이 확인을 마지막으로 한 결과다. 명령은 `docker/`에서 `docker compose up -d` 뒤에 친다.
```
docker compose exec cyclone ldd /usr/local/bin/cyclone_shape    # not found 가 없고 libddsc.so 가 /opt/cyclone/lib 에서 온다
docker compose exec fastdds ldd /usr/local/bin/fastdds_shape    # not found 가 없다
docker compose exec cyclone cyclone_shape -h
docker compose exec fastdds fastdds_shape -h
docker compose exec fastdds cat /usr/local/share/fastdds_shape.origname
docker image inspect mdds/cyclone-shape:11.0.1-f2aaccb4 --format '{{json .Config.Labels}}'
docker image inspect mdds/fastdds-shape:3.6.1  --format '{{json .Config.Labels}}'
docker image ls 'mdds/*'
```
| 확인 | 잡으려는 문제 |
|---|---|
| `ldd` | runtime 단계는 빌드 결과의 일부만 복사하고 apt 패키지 이름도 배포판마다 다르다. 빠진 공유 라이브러리가 있으면 이미지 빌드는 성공해도 실행할 때 죽는다 |
| `-h` | 실행되는지, 옵션이 dds-rtps README와 같은지. 01-04-environment-and-captures 노트의 시나리오 표가 그 옵션을 전제로 한다 |
| `fastdds_shape.origname` | 실제로 링크된 Fast DDS 버전. 실행 파일 이름을 짧게 바꿔서 원래 이름에 든 버전을 여기서만 볼 수 있다 |
| 이미지 라벨 | 소스 커밋. Cyclone은 직접 컴파일해서 실행 파일 이름에 버전이 없다 |
| `image ls` | 이미지 크기. "빌드 결과"와 크게 다르면 원인을 본다 |

## 빌드 문제와 대응
| 증상 | 원인 후보 | 대응 |
|---|---|---|
| shape_main 실행 시 라이브러리 없음 | runtime 단계에 공유 라이브러리 누락 | `ldd`로 빠진 라이브러리 확인 후 runtime 단계 apt에 추가하거나 빌드 단계에서 COPY |
| Fast DDS shape_main configure / 링크 실패 (TinyXml2 관련) | 정적 config의 `find_dependency(TinyXml2)` 대소문자 문제 | `-DTinyXML2_DIR` 지정, 안 되면 Fast DDS를 공유 라이브러리로 빌드하고 `/opt/fastdds/lib` 복사 |
| `fastddsgen`이 없음 / Gradle 실패 | JDK 누락, 서브모듈 누락, 네트워크 | JDK 17 설치 확인, `--recurse-submodules`로 clone했는지 확인, 빌드 머신이 인터넷에 나갈 수 있는지 확인 |
| Cyclone `idlc` 옵션 오류 | 경고 옵션 철자 | `idlc -h`로 확인 후 수정. 옵션 없이 생성해도 동작에는 영향이 없다 |
| 빌드 중 메모리 부족으로 컴파일러나 Docker 데몬이 죽음 | 병렬 빌드, Docker Desktop 메모리 부족 | `--build-arg JOBS=4` (또는 2)로 병렬 수를 줄이거나 Docker Desktop 메모리를 늘린다 |
| 이미지 빌드 중 git clone 실패 (인증서 오류) | `ca-certificates` 누락 | 빌드 단계 apt에 추가 |
