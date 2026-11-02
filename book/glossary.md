# 용어 정리

[목차](README.md)

## 기본 개념
- **data-centric / message-centric**: message-centric 미들웨어는 바이트 메시지를 나르고 뜻은 응용이 안다. data-centric 미들웨어는 데이터의 타입과 "어떤 대상의 값인가"를 알고 상태를 관리한다. DDS는 data-centric이다 (DDS 1.4 Sec. 1.2).
- **global data space**: DCPS가 데이터를 보는 모델. 응용은 이 공간에 값을 쓰고 (publisher) 관심 있는 부분을 읽으며 (subscriber), 미들웨어가 값을 관심 있는 쪽으로 전한다 (DDS 1.4 Sec. 1.2).
- **토픽 (topic)**: global data space를 나누는 단위. 이름과 타입을 가진다.
- **key, 인스턴스 (instance)**: 타입 필드 가운데 대상을 가르는 필드가 key다. key가 같은 값은 같은 인스턴스의 연속된 값이고, key가 다르면 다른 인스턴스다 (DDS 1.4 Sec. 2.2.1.2.2).
- **도메인 (domain)**: 서로 통신하는 응용들의 묶음. 같은 도메인 번호 (domainId)를 쓰는 참가자끼리만 서로를 찾고 통신한다. 그래서 같은 네트워크에서 여러 시스템이 서로 간섭하지 않고 함께 동작한다 (DDS 1.4 Sec. 2.2.2.2.1).
- **참가자 (DomainParticipant)**: 한 DDS 도메인에 참여하는 단위. 보통 프로세스 하나에 하나다. 디스커버리는 참가자 단위로 시작한다.
- **QoS 정책**: 데이터마다 다른 요구 (신뢰성, 보관 개수, 늦게 온 reader에게 줄지, 마감 시간 등)를 정하는 설정. writer와 reader가 각자 밝히고 맞아야 매칭한다.
- **브로커 (broker)**: 모든 메시지가 거쳐 가는 중앙 서버. MQTT가 이 방식이다. DDS의 와이어 프로토콜은 브로커에 기대지 않는다 (RTPS 2.5 Sec. 7.2).
- **단일 장애점 (single point of failure)**: 그 하나가 멈추면 전체가 멈추는 구성 요소. 이중화와 failover로 없앤다.
- **lease**: 참가자가 SPDP 공지에 실어 보내는 유효 기간 (leaseDuration). 상대는 그 참가자의 공지가 이 기간 넘게 갱신되지 않으면 사라진 것으로 보고 정리한다 (RTPS 2.5 Sec. 8.5.3.3, 8.5.5.2). 기간은 구현의 설정값이고 (예: Cyclone DDS 10초, Fast DDS 20초), 공지에 값이 없으면 100초로 본다 (RTPS 2.5 표 9.19).
- **DDS-XRCE** (DDS for eXtremely Resource Constrained Environments): 자원이 극히 제한된 기기를 위한 OMG 규격. 작은 기기 (Client)는 Agent를 거쳐 DDS에 붙고, Agent가 대신 DDS 참가자가 된다. micro-ROS가 이 방식을 쓴다.

## 모델
- **PIM**(Platform Independent Model): 특정 언어/전송 수단과 무관하게 개념/동작만 정의한 추상 모델. 예: "DataWriter는 write()로 샘플을 발행한다"는 수준.
- **PSM**(Platform Specific Model): PIM을 특정 플랫폼에 맞춰 구체화한 것. 예: DDS의 C++ API(DDS-PSM-Cxx), Java API(DDS-JAVA), RTPS의 UDP/IP 매핑(RTPS 2.5 Sec. 9).
- **DCPS**(Data-Centric Publish-Subscribe): DDS 1.4 규격의 본체. 토픽/DataWriter/DataReader/QoS로 구성되는 발행/구독 API의 PIM이다. DDS 1.4 2.2절이 PIM이고, 2.3절 IDL PSM은 그 PIM을 IDL 인터페이스로 옮긴 것이다.
- **IDL4 -> Rust 언어 매핑**: IDL로 정의한 타입/인터페이스를 Rust 코드로 어떻게 옮길지 정하는 규격. 2026년 RFP(mars/2026-3-6) 단계로 아직 표준이 아니다. DDS API(DCPS PSM)를 정하는 규격도 아니다.

## 프로토콜
- **RTPS**(Real-Time Publish-Subscribe): 서로 다른 벤더의 DDS 구현이 통신할 수 있게 정한 와이어 프로토콜. 메시지 형식, 신뢰성(HEARTBEAT/ACKNACK), 디스커버리(SPDP/SEDP)를 정의한다. 규격 이름 **DDSI-RTPS**의 DDSI는 "DDS Interoperability"다. DCPS가 응용에 보이는 API라면, RTPS는 그 아래에서 실제로 네트워크에 흐르는 바이트를 정한다.
- **RTPS 메시지와 서브메시지 (submessage)**: 네트워크에 흐르는 RTPS 단위. 메시지 하나는 헤더 (프로토콜 버전, vendorId, 보낸 참가자의 guidPrefix) 뒤에 서브메시지 여러 개가 이어진 것이고, UDP에서는 데이터그램 하나가 메시지 하나다 (RTPS 2.5 Sec. 8.3.3, 9.5). 서브메시지에는 엔드포인트를 상대로 하는 것 (DATA, HEARTBEAT, ACKNACK, GAP 등)과 뒤따르는 서브메시지를 해석할 정보를 주는 것 (INFO_TS 시각, INFO_DST 목적지 등)이 있다 (Sec. 8.3.8). Wireshark의 Info 열은 이 서브메시지 이름을 차례로 나열한 것이다.
- **내장 엔드포인트 (built-in endpoint)**: 디스커버리와 liveliness를 위해 규격이 미리 정해 둔 writer와 reader. 참가자는 필요한 것을 갖추고 어떤 것이 있는지 SPDP로 알린다. 참가자 정보 (SPDP), writer 정보와 reader 정보 (SEDP), 참가자 메시지 (liveliness)를 사용자 데이터와 같은 DATA 서브메시지로 주고받는다 (RTPS 2.5 Sec. 8.5.2, 8.5.4.3, 8.4.13). 사용자 엔드포인트와는 entityId로 구분한다.
- **entityId, GUID**: entityId는 참가자 안에서 엔드포인트 하나를 가리키는 4바이트 번호다. 마지막 바이트가 종류 (key가 있는 사용자 writer 0x02, key가 있는 사용자 reader 0x07, key가 있는 내장 writer 0xc2, key가 있는 내장 reader 0xc7 등. key가 없는 종류는 번호가 다르다. Sec. 9.3.1.2 표 9.1)이고, 내장 엔드포인트의 값은 규격이 정해 모든 참가자에서 같다 (예: SPDP writer `0x000100c2`, Sec. 9.3.1.3). 사용자 엔드포인트의 값은 구현이 고른다. 도메인 안에서 엔드포인트를 하나로 가리키는 것은 GUID = 참가자의 guidPrefix (12바이트) + entityId (4바이트)다 (Sec. 8.2.4.1).
- **로케이터 (locator)**: 패킷을 보낼 곳을 나타내는 주소 종류, 주소, 포트의 묶음 (예: UDPv4 172.28.0.10:44686). 참가자는 자기가 받는 로케이터를 SPDP와 SEDP의 정보에 실어 알리고, 상대는 그것을 읽어 보낸다.
- **메타트래픽 (metatraffic)**: 디스커버리와 liveliness 같은 내장 엔드포인트의 트래픽. 사용자 데이터 트래픽과 구분해 부르는 이름이고, 기본 포트도 따로 있다 (도메인 0의 참가자 0이면 메타트래픽 유니캐스트 7410, 사용자 데이터 유니캐스트 7411. RTPS 2.5 Sec. 9.6.2).
- **SPDP**(Simple Participant Discovery Protocol): 같은 도메인의 참가자(DomainParticipant)끼리 서로를 찾는 프로토콜. 각 참가자가 미리 정한 주소 (기본은 멀티캐스트 239.255.0.1. 설정한 유니캐스트 주소도 가능)로 "나는 누구이고 어느 주소로 연락하면 된다"를 주기적으로 알린다(best-effort). 규격의 기본 주기는 30초이고 구현마다 다르게 설정한다 (RTPS 2.5 Sec. 9.6.2.4).
- **SEDP**(Simple Endpoint Discovery Protocol): SPDP로 찾은 참가자끼리 각자의 DataWriter/DataReader(엔드포인트)의 토픽/타입/QoS 정보를 교환하고, 서로 맞는 쌍을 매칭하는 프로토콜(reliable). 이름의 "Simple"은 RTPS가 기본 제공하는 방식이라는 뜻으로, 벤더가 다른 디스커버리 방식을 쓸 수도 있다.
- **매칭 (matching)**: 상대의 writer 또는 reader를 SEDP로 알게 된 뒤, 자기 쪽 엔드포인트와 짝이 맞는지 판정해 통신 상대로 등록하는 일. 토픽 이름과 타입이 같고, partition이 겹치고, 양쪽 QoS가 호환될 때 (reader가 요구한 수준을 writer가 제공할 때) 매칭된다. shape_main은 매칭되면 `on_publication_matched` / `on_subscription_matched`를 찍는다.
- **벤더 전용 서브메시지 / PID (vendor-specific submessage / ParameterId)**: RTPS가 벤더 확장용으로 비워 둔 번호 구간. 서브메시지 ID 0x80-0xff, ParameterId 0x8000-0xffff (RTPS 2.5 Sec. 9.4.5.1.1, Sec. 9.4.2.11 표 9.6). 받는 쪽은 모르는 벤더의 서브메시지를 건너뛰고 나머지를 계속 해석한다 (RTPS 2.5 Sec. 8.3.4.1). 모르는 ParameterId도 건너뛰되, must-understand 비트 (0x4000)가 켜진 것을 모르면 그 서브메시지를 버린다 (표 9.6). 예: 이 책이 쓰는 Fast DDS 3.6.1은 통계 모듈이 모든 메시지 끝에 서브메시지 0x80 (60바이트)을 붙인다. Wireshark는 이것을 `Unknown[80]`으로 표시한다.
- **reliable, best-effort (RELIABILITY QoS)**: writer와 reader가 샘플 유실을 어떻게 다룰지 정하는 QoS. reliable이면 writer가 HEARTBEAT로 가진 샘플의 범위를 알리고 reader가 ACKNACK로 받은 것과 빠진 것을 알려 다시 받는다. best-effort이면 reader는 받기만 하고 아무것도 보내지 않으며, 빠진 샘플은 그대로 지나간다 (RTPS 2.5 Sec. 8.4.2.2, 8.4.2.3). 사용자 QoS와 무관하게 SEDP 엔드포인트와 참가자 메시지 writer는 reliable이고, SPDP는 best-effort다.
- **vendorId**: RTPS 헤더와 SPDP 참가자 정보에 들어가는 2바이트 구현 식별자. 값은 OMG가 배정하고 목록은 OMG 웹에 있다 (RTPS 2.5 Sec. 8.3.3.1). 벤더 전용 서브메시지와 ParameterId는 이 값에 따라 해석한다. 예: Eclipse Cyclone DDS 0x0110 (Wireshark 표시 01.16), eProsima Fast DDS 0x010f (01.15).
- **dispose, unregister, 인스턴스 상태**: writer가 인스턴스 (key 값 하나)에 대해 하는 두 가지 선언. dispose는 그 인스턴스가 더는 유효하지 않다는 선언이고, unregister는 이 writer가 그 인스턴스를 더는 갱신하지 않는다는 선언이다. 와이어에서는 DATA의 inline QoS에 PID_STATUS_INFO의 D, U 비트로 실린다 (RTPS 2.5 Sec. 9.6.4.9). reader가 보는 인스턴스 상태는 ALIVE, NOT_ALIVE_DISPOSED (writer가 dispose함), NOT_ALIVE_NO_WRITERS (살아 있는 writer가 없음을 reader가 감지함)다 (DDS 1.4 Sec. 2.2.2.5.1.3).
- **inline QoS**: DATA 서브메시지 안에 함께 실리는 ParameterId 목록. 샘플에 붙는 정보 (PID_STATUS_INFO의 dispose / unregister 비트, key hash 등)를 나른다. DATA 헤더의 InlineQosFlag가 있고 없음을 나타낸다 (RTPS 2.5 Sec. 8.3.8.2, 9.6.4).
- **DATA_FRAG, HEARTBEAT_FRAG, NACK_FRAG**: 전송 한 번에 담을 수 없는 큰 샘플을 writer가 조각 (fragment)으로 나누어 보내는 서브메시지와, 그 조각의 범위를 알리고 빠진 조각을 요청하는 서브메시지 (RTPS 2.5 Sec. 8.3.8.3, 8.3.8.7, 8.3.8.12). RTPS의 조각은 IP 층의 단편화 (IP fragmentation)와 다른 층이다. 조각 크기는 writer가 정한다 (Sec. 8.4.14.1).
- **RepresentationIdentifier (encapsulation kind)**: 직렬화된 페이로드 맨 앞 2바이트로, 데이터가 어떤 표현으로 직렬화되었는지 알린다 (RTPS 2.5 Sec. 10.2). 예: CDR_LE 0x0001 (XCDR1, 리틀 엔디언), D_CDR2_LE 0x0009 (XCDR2의 appendable 타입, 리틀 엔디언. DDS-XTypes 1.3 표 60). 그 뒤 2바이트는 옵션이고, XTypes는 그 하위 2비트에 끝의 패딩 바이트 수를 싣는다 (DDS-XTypes 1.3 Sec. 7.6.3.1.2).
- **데이터 표현 (PID_DATA_REPRESENTATION)**: writer와 reader가 SEDP의 정보에 실어 알리는, 자기가 쓰거나 받을 수 있는 표현 (XCDR, XCDR2)의 목록. 비어 있으면 XCDR로 본다 (DDS-XTypes 1.3 Sec. 7.6.3.1.1).
- **pre-emptive ACKNACK**: reader가 HEARTBEAT를 받기 전에 보내는 ACKNACK. 평소 reader는 HEARTBEAT에 답할 때만 ACKNACK를 보내지만, writer를 처음 찾았을 때는 최적화로 먼저 보낼 수 있다. writer가 여기에 답할 의무는 없다 (RTPS 2.5 Sec. 8.4.2.3).
- **HEARTBEAT piggyback**: writer가 DATA 서브메시지 바로 뒤에 HEARTBEAT를 같은 메시지에 넣어 보내는 것 (RTPS 2.5 Sec. 8.4.15.4).
- **ContentFilteredTopic**: 기존 토픽에 SQL 비슷한 조건식 (예: `color = %0`)을 붙여 조건에 맞는 샘플만 받는 DCPS 토픽. 구독자가 SEDP에 `PID_CONTENT_FILTER_PROPERTY`로 조건을 알리므로 발행자 쪽에서 미리 거를 수도 있다. shape_main은 구독자에게 `-c`를 주면 이것을 만든다.

## 직렬화
- **CDR**(Common Data Representation): CORBA에서 온 바이너리 직렬화 규칙. 각 값을 자기 크기만큼 정렬(4바이트 int는 4의 배수 위치)하고, 길이 태그 없이 선언 순서대로 이어 붙인다.
- **XCDR1**(XTypes의 Extended CDR 버전 1): 고전 CDR 그대로(최대 정렬 8)에, 필드 추가/순서 변경이 가능한 mutable 타입용 "파라미터 리스트(PL_CDR, 멤버마다 ID+길이)" 형식을 더한 것. ROS 2가 쓰는 방식이다.
- **XCDR2**(Extended CDR 버전 2): 최대 정렬을 4로 줄이고, 타입 진화를 위해 본문 길이 헤더(DHEADER)와 멤버별 헤더(EMHEADER)를 도입한 개선판. 구조가 바뀐 타입끼리도 안전하게 통신하기 위한 형식이다.

## 도구
- **shape_main**: OMG DDS 상호운용 작업반이 만든 작은 테스트 프로그램. 토픽 하나(ShapeType: `@appendable struct { @key string<128> color; int32 x; int32 y; int32 shapesize; sequence<uint8> additional_payload_size; }`)를 QoS 옵션에 따라 발행하거나 구독하고, 받은 샘플을 정해진 형식으로 stdout에 찍는다. 구현은 `omg-dds/dds-rtps` 저장소에 언어별로 있다: Cyclone DDS용 C 언어 구현 (`srcC/shape_main.c`), Connext, Fast DDS, OpenDDS 등이 함께 쓰는 C++ 언어 구현 (`srcCxx/shape_main.cxx`, 벤더별 분기는 `#if defined(...)`), Rust로 만든 DDS 구현 (Dust DDS, HDDS, RustDDS)용 Rust 구현 (`srcRs/`, `RustDDS/`). 코드는 다르지만 명령행 옵션, 동작, 출력 형식이 같도록 맞춰져 있다. 각 벤더가 자기 DDS 라이브러리로 빌드한 실행 파일을 릴리스에 올린다. OMG 상호운용 테스트(`interoperability_report.py`)는 이 실행 파일들을 짝지어 돌리고 stdout을 비교해 합격 여부를 판정한다. 벤더가 올린 실행 파일은 x86-64 Linux용뿐이지만 소스 자체는 아키텍처 제한이 없다.
- **ShapesDemo**: 같은 ShapeType 토픽을 GUI로 보여 주는 각 벤더의 데모 프로그램 (RTI, eProsima 등). shape_main은 이것의 CLI판이라고 보면 된다.
- **bridge 네트워크 (Docker)**: Docker가 호스트 안에 만드는 가상 L2 스위치. 컨테이너의 `eth0`가 여기에 꽂힌다. 같은 bridge 안에서는 유니캐스트와 멀티캐스트가 모두 통하고, 바깥으로는 NAT를 거친다. 사용자 정의 bridge는 기본 bridge와 달리 컨테이너 이름으로 서로를 찾을 수 있다.
- **네트워크 네임스페이스 (network namespace)**: Linux 커널이 네트워크 스택 (인터페이스, IP 주소, 라우팅 표, 소켓)을 프로세스 묶음마다 따로 두는 기능. Docker 컨테이너가 각자 `eth0`과 IP를 갖는 것이 이것 덕분이다. 다른 컨테이너의 네임스페이스를 빌려 쓰면 (`--network container:<id>`) 같은 `eth0`과 IP를 공유한다.

## 네트워크
- **IP 주소, MAC 주소**: 패킷에 적히는 두 종류의 주소. IP 주소는 프로그램이 상대를 가리킬 때 쓰고, MAC 주소는 스위치와 네트워크 인터페이스가 쓴다. 스위치는 MAC 주소만 보고 패킷을 전달하므로, 커널은 보낼 때 목적지 IP 주소에 맞는 MAC 주소를 함께 적는다.
- **ARP** (Address Resolution Protocol): 목적지 IP 주소를 가진 호스트에게 MAC 주소를 물어 알아내는 절차. 멀티캐스트 그룹 주소에는 쓰지 않는다. 그룹 주소의 MAC 주소는 `01:00:5e:00:00:00`의 아래 23비트에 그룹 주소의 아래 23비트를 넣어 계산한다 (RFC 1112 Sec. 6.4).
- **서브넷**: IP 주소의 앞부분이 같은 호스트의 묶음. 보통 같은 스위치에 연결되어 라우터 없이 바로 주고받는다. 라우터는 멀티캐스트 라우팅을 따로 설정하지 않으면 멀티캐스트를 다른 서브넷으로 넘기지 않는다.
- **멀티캐스트, 그룹 가입**: 그룹 주소 (IPv4 224.0.0.0/4)로 보내 그 그룹에 가입한 호스트만 받게 하는 전송 방식. 받는 쪽 프로그램이 소켓 옵션 `IP_ADD_MEMBERSHIP`으로 가입하면 커널이 그 그룹의 패킷을 소켓으로 올려 보내도록 등록하고, IGMP로 가입을 네트워크에 알린다. 보내는 쪽은 가입하지 않아도 된다. RTPS의 참가자 디스커버리 기본 주소는 239.255.0.1이다 (RTPS 2.5 Sec. 9.6.2.4).
- **IGMP** (Internet Group Management Protocol): 호스트가 가입한 멀티캐스트 그룹을 네트워크 장비에 알리는 프로토콜. 버전 3 (RFC 3376)에서는 가입과 탈퇴를 모두 Membership Report로 224.0.0.22에 보내고, 유실에 대비해 1초 안의 무작위 시점에 한 번 더 보낸다 (기본값).
- **IGMP snooping**: 스위치가 IGMP Report를 엿보아 어느 스위치 포트에 어느 그룹의 가입자가 있는지 기록하고, 멀티캐스트를 그 스위치 포트로만 전달하는 기능 (RFC 4541, 정보성 문서). 기록을 유지하려면 querier가 있어야 한다.
- **querier**: 주기적으로 IGMP Query를 보내 호스트들의 가입 상태를 다시 묻는 장치. 보통 라우터가 맡는다. Linux bridge는 snooping이 활성화되어 있어도 querier가 없으면 가입 기록을 전달에 쓰지 않고 모든 멀티캐스트를 flooding한다.
- **호스트, 네트워크 인터페이스**: 네트워크에 연결된 컴퓨터가 호스트다. Docker 컨테이너는 자기 네트워크 스택을 가지므로 네트워크에서 보면 호스트 하나다. 호스트가 네트워크에 연결되는 접점이 네트워크 인터페이스다 (컨테이너에서는 `eth0`).
- **스위치, 스위치 포트**: 같은 네트워크의 호스트들을 잇고 MAC 주소를 보고 패킷을 전달하는 장치가 스위치다. 호스트가 연결되는 자리가 스위치 포트다. UDP 포트 (한 호스트 안에서 소켓을 가르는 번호)와는 다른 것이다.
- **MTU** (Maximum Transmission Unit): 링크가 한 프레임에 실을 수 있는 최대 바이트 수. 이더넷은 보통 1,500바이트다. IP 패킷이 이보다 크면 IP 층이 조각 (IP fragmentation)으로 나누어 보내고 받는 쪽 커널이 다시 합친다. RTPS의 DATA_FRAG 조각과는 다른 층이다.
- **TTL** (Time to Live): IP 패킷이 지날 수 있는 라우터 수의 상한. 라우터를 지날 때마다 1씩 줄고 0이 되면 버려진다. 멀티캐스트 소켓의 기본값은 1이다.
- **flooding**: 스위치가 받은 패킷을 받은 스위치 포트를 뺀 모든 스위치 포트로 복사해 보내는 것. 받을 호스트가 어느 스위치 포트에 있는지 모를 때의 동작이다.
- **multicast loopback**: 호스트가 보낸 멀티캐스트를 자기 수신 경로로도 한 부 돌려주는 커널 동작 (Linux 기본값 켜짐). 그래서 보낸 호스트에서 캡처하면 같은 멀티캐스트 패킷이 나가는 것과 들어오는 것 두 번 보인다. `lo` 인터페이스와는 관계없다.

## 설계
- **I/O-free core**: 소켓, 시계, 스레드를 직접 쓰지 않고 입력을 받아 출력(보낼 패킷, 타이머, 이벤트)만 돌려주는 프로토콜 코어. 커뮤니티에서는 "sans-IO"라고도 부른다. 시간과 받은 바이트를 바깥에서 넣어 주므로 타이머 만료, 패킷 손실과 재정렬을 결정적으로 시험할 수 있다. 실제 송수신과 대기는 바깥의 얇은 드라이버가 맡는다.
