# 용어 정리

[roadmap](../docs-for-agent/roadmap.md)

## 기본 개념
- **data-centric / message-centric**: message-centric 미들웨어는 바이트 메시지를 나르고 뜻은 응용이 안다. data-centric 미들웨어는 데이터의 타입과 "어떤 대상의 값인가"를 알고 상태를 관리한다. DDS는 data-centric이다 (DDS 1.4 Sec. 1.2).
- **global data space**: DCPS가 데이터를 보는 모델. 응용은 이 공간에 값을 쓰고 (publisher) 관심 있는 부분을 읽으며 (subscriber), 미들웨어가 값을 관심 있는 쪽으로 전한다 (DDS 1.4 Sec. 1.2).
- **토픽 (topic)**: global data space를 나누는 단위. 이름과 타입을 가진다.
- **키 (key), 인스턴스 (instance)**: 타입 필드 가운데 대상을 가르는 필드가 키다. 키가 같은 값은 같은 인스턴스의 연속된 값이고, 키가 다르면 다른 인스턴스다 (DDS 1.4 Sec. 2.2.1.2.2).
- **참가자 (DomainParticipant)**: 한 DDS 도메인에 참여하는 단위. 보통 프로세스 하나에 하나다. 디스커버리는 참가자 단위로 시작한다.
- **QoS 정책**: 데이터마다 다른 요구 (신뢰성, 보관 개수, 늦게 온 reader에게 줄지, 마감 시간 등)를 정하는 설정. writer와 reader가 각자 밝히고 맞아야 매칭한다.
- **브로커 (broker)**: 모든 메시지가 거쳐 가는 중앙 서버. MQTT가 이 방식이다. DDS의 와이어 프로토콜은 브로커에 기대지 않는다 (RTPS 2.5 Sec. 7.2).
- **단일 장애점 (single point of failure)**: 그 하나가 멈추면 전체가 멈추는 구성 요소. 이중화와 failover로 없앤다.
- **lease**: 참가자가 SPDP 공지에 실어 보내는 유효 기간. 그 안에 소식이 없으면 상대는 그 참가자가 사라진 것으로 보고 정리한다.
- **DDS-XRCE** (DDS for eXtremely Resource Constrained Environments): 자원이 극히 제한된 기기를 위한 OMG 규격. 작은 기기 (Client)는 Agent를 거쳐 DDS에 붙고, Agent가 대신 DDS 참가자가 된다. micro-ROS가 이 방식을 쓴다.

## 모델
- **PIM**(Platform Independent Model): 특정 언어/전송 수단과 무관하게 개념/동작만 정의한 추상 모델. 예: "DataWriter는 write()로 샘플을 발행한다"는 수준.
- **PSM**(Platform Specific Model): PIM을 특정 플랫폼에 맞춰 구체화한 것. 예: DDS의 C++ API(DDS-PSM-Cxx), Java API(DDS-JAVA), RTPS의 UDP/IP 매핑(RTPS 9장).
- **DCPS**(Data-Centric Publish-Subscribe): DDS 1.4 규격의 본체. 토픽/DataWriter/DataReader/QoS로 구성되는 발행/구독 API의 PIM이다. DDS 1.4 2.2절이 PIM이고, 2.3절 IDL PSM은 그 PIM을 IDL 인터페이스로 옮긴 것이다.
- **IDL4 -> Rust 언어 매핑**: IDL로 정의한 타입/인터페이스를 Rust 코드로 어떻게 옮길지 정하는 규격. 2026년 RFP(mars/2026-3-6) 단계로 아직 표준이 아니다. DDS API(DCPS PSM)를 정하는 규격도 아니다.

## 프로토콜
- **RTPS**(Real-Time Publish-Subscribe): 서로 다른 벤더의 DDS 구현이 통신할 수 있게 정한 와이어 프로토콜. 메시지 형식, 신뢰성(HEARTBEAT/ACKNACK), 디스커버리(SPDP/SEDP)를 정의한다. 규격 이름 **DDSI-RTPS**의 DDSI는 "DDS Interoperability"다. DCPS가 응용에 보이는 API라면, RTPS는 그 아래에서 실제로 네트워크에 흐르는 바이트를 정한다.
- **SPDP**(Simple Participant Discovery Protocol): 같은 도메인의 참가자(DomainParticipant)끼리 서로를 찾는 프로토콜. 각 참가자가 멀티캐스트로 "나는 누구이고 어느 주소로 연락하면 된다"를 주기적으로 알린다(best-effort).
- **SEDP**(Simple Endpoint Discovery Protocol): SPDP로 찾은 참가자끼리 각자의 DataWriter/DataReader(엔드포인트)의 토픽/타입/QoS 정보를 교환하고, 서로 맞는 쌍을 매칭하는 프로토콜(reliable). 이름의 "Simple"은 RTPS가 기본 제공하는 방식이라는 뜻으로, 벤더가 다른 디스커버리 방식을 쓸 수도 있다.
- **매칭 (matching)**: 상대의 writer 또는 reader를 SEDP로 알게 된 뒤, 자기 쪽 엔드포인트와 짝이 맞는지 판정해 통신 상대로 등록하는 일. 조건 (토픽, 타입, partition, RxO QoS)과 동작은 [chapter 11 노트](../docs-for-agent/notes/11-sedp-matching.md). shape_main은 매칭되면 `on_publication_matched` / `on_subscription_matched`를 찍는다.
- **벤더 전용 서브메시지 / PID (vendor-specific submessage / ParameterId)**: RTPS가 벤더 확장용으로 비워 둔 번호 구간. 서브메시지 ID 0x80-0xff, ParameterId 0x8000-0xffff (RTPS 2.5 Sec. 9.4.5.1.1, Sec. 9.6.2.2.1). 해석 규칙은 [chapter 7 노트](../docs-for-agent/notes/07-rtps-messages.md). 예: Fast DDS는 모든 메시지 끝에 통계 서브메시지 0x80을 붙인다.
- **pre-emptive ACKNACK**: reader가 HEARTBEAT를 받기 전에 보내는 ACKNACK (RTPS 2.5 Sec. 8.4.2.3). 동작은 [chapter 10 노트](../docs-for-agent/notes/10-reliable.md).
- **HEARTBEAT piggyback**: writer가 DATA 서브메시지 바로 뒤에 HEARTBEAT를 같은 메시지에 넣어 보내는 것 (RTPS 2.5 Sec. 8.4.15.4).
- **ContentFilteredTopic**: 기존 토픽에 SQL 비슷한 조건식 (예: `color = %0`)을 붙여 조건에 맞는 샘플만 받는 DCPS 토픽. 구독자가 SEDP에 `PID_CONTENT_FILTER_PROPERTY`로 조건을 알리므로 발행자 쪽에서 미리 거를 수도 있다. shape_main은 구독자에게 `-c`를 주면 이것을 만든다.

## 직렬화
- **CDR**(Common Data Representation): CORBA에서 온 바이너리 직렬화 규칙. 각 값을 자기 크기만큼 정렬(4바이트 int는 4의 배수 위치)하고, 길이 태그 없이 선언 순서대로 이어 붙인다.
- **XCDR1**(XTypes의 Extended CDR 버전 1): 고전 CDR 그대로(최대 정렬 8)에, 필드 추가/순서 변경이 가능한 mutable 타입용 "파라미터 리스트(PL_CDR, 멤버마다 ID+길이)" 형식을 더한 것. ROS 2가 쓰는 방식이다.
- **XCDR2**(Extended CDR 버전 2): 최대 정렬을 4로 줄이고, 타입 진화를 위해 본문 길이 헤더(DHEADER)와 멤버별 헤더(EMHEADER)를 도입한 개선판. 구조가 바뀐 타입끼리도 안전하게 통신하기 위한 형식이다.

## 도구
- **shape_main**: OMG DDS 상호운용 작업반이 만든 작은 테스트 프로그램. 토픽 하나(ShapeType: `@key string color; long x; long y; long shapesize;`)를 QoS 옵션에 따라 발행하거나 구독하고, 받은 샘플을 정해진 형식으로 stdout에 찍는다. 소스는 하나(`omg-dds/dds-rtps`의 `srcCxx/shape_main.cxx`)이고, 각 벤더가 자기 DDS 라이브러리로 빌드한 실행 파일을 릴리스에 올린다. OMG 상호운용 테스트(`interoperability_report.py`)는 이 실행 파일들을 짝지어 돌리고 stdout을 비교해 합격 여부를 판정한다. 벤더가 올린 실행 파일은 x86-64 Linux용뿐이지만 소스 자체는 아키텍처 제한이 없다. 이 프로젝트는 Docker 안에서 소스로 빌드해 쓴다 (chapter 1). Cyclone용은 C판(`srcC/shape_main.c`), 나머지 벤더는 C++판이다.
- **ShapesDemo**: 같은 ShapeType 토픽을 GUI로 보여 주는 각 벤더의 데모 프로그램 (RTI, eProsima 등). shape_main은 이것의 CLI판이라고 보면 된다.
- **bridge 네트워크 (Docker)**: Docker가 호스트 안에 만드는 가상 L2 스위치. 컨테이너의 `eth0`가 여기에 꽂힌다. 같은 bridge 안에서는 유니캐스트와 멀티캐스트가 모두 통하고, 바깥으로는 NAT를 거친다. 사용자 정의 bridge는 기본 bridge와 달리 컨테이너 이름으로 서로를 찾을 수 있다.
- **네트워크 네임스페이스 (network namespace)**: Linux 커널이 네트워크 스택 (인터페이스, IP 주소, 라우팅 표, 소켓)을 프로세스 묶음마다 따로 두는 기능. Docker 컨테이너가 각자 `eth0`과 IP를 갖는 것이 이것 덕분이다. 다른 컨테이너의 네임스페이스를 빌려 쓰면 (`--network container:<id>`) 같은 `eth0`과 IP를 공유한다. 이 프로젝트의 `mcast-check.sh`가 이 방식으로 벤더 컨테이너 안에서 도구를 돌린다.

## 설계
- **I/O-free core**: 소켓, 시계, 스레드를 직접 쓰지 않고 입력을 받아 출력(보낼 패킷, 타이머, 이벤트)만 돌려주는 프로토콜 코어. 커뮤니티에서는 "sans-IO"라고도 부른다. 자세한 내용은 [roadmap 설계 원칙](../docs-for-agent/roadmap.md#4-설계-원칙).
