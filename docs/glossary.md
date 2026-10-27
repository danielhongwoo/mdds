# 용어 정리

[roadmap](roadmap.md)

## 모델
- **PIM**(Platform Independent Model): 특정 언어/전송 수단과 무관하게 개념/동작만 정의한 추상 모델. 예: "DataWriter는 write()로 샘플을 발행한다"는 수준.
- **PSM**(Platform Specific Model): PIM을 특정 플랫폼에 맞춰 구체화한 것. 예: DDS의 C++ API(DDS-PSM-Cxx), Java API(DDS-JAVA), RTPS의 UDP/IP 매핑(RTPS 9장).
- **DCPS**(Data-Centric Publish-Subscribe): DDS 1.4 규격의 본체. 토픽/DataWriter/DataReader/QoS로 구성되는 발행/구독 API의 PIM이다. DDS 1.4 2.2절이 PIM이고, 2.3절 IDL PSM은 그 PIM을 IDL 인터페이스로 옮긴 것이다.
- **IDL4 -> Rust 언어 매핑**: IDL로 정의한 타입/인터페이스를 Rust 코드로 어떻게 옮길지 정하는 규격. 2026년 RFP(mars/2026-3-6) 단계로 아직 표준이 아니다. DDS API(DCPS PSM)를 정하는 규격도 아니다.

## 프로토콜
- **RTPS**(Real-Time Publish-Subscribe): 서로 다른 벤더의 DDS 구현이 통신할 수 있게 정한 와이어 프로토콜. 메시지 형식, 신뢰성(HEARTBEAT/ACKNACK), 디스커버리(SPDP/SEDP)를 정의한다. 규격 이름 **DDSI-RTPS**의 DDSI는 "DDS Interoperability"다. DCPS가 응용에 보이는 API라면, RTPS는 그 아래에서 실제로 네트워크에 흐르는 바이트를 정한다.
- **SPDP**(Simple Participant Discovery Protocol): 같은 도메인의 참가자(DomainParticipant)끼리 서로를 찾는 프로토콜. 각 참가자가 멀티캐스트로 "나는 누구이고 어느 주소로 연락하면 된다"를 주기적으로 알린다(best-effort).
- **SEDP**(Simple Endpoint Discovery Protocol): SPDP로 찾은 참가자끼리 각자의 DataWriter/DataReader(엔드포인트)의 토픽/타입/QoS 정보를 교환하고, 서로 맞는 쌍을 매칭하는 프로토콜(reliable). 이름의 "Simple"은 RTPS가 기본 제공하는 방식이라는 뜻으로, 벤더가 다른 디스커버리 방식을 쓸 수도 있다.

## 직렬화
- **CDR**(Common Data Representation): CORBA에서 온 바이너리 직렬화 규칙. 각 값을 자기 크기만큼 정렬(4바이트 int는 4의 배수 위치)하고, 길이 태그 없이 선언 순서대로 이어 붙인다.
- **XCDR1**(XTypes의 Extended CDR 버전 1): 고전 CDR 그대로(최대 정렬 8)에, 필드 추가/순서 변경이 가능한 mutable 타입용 "파라미터 리스트(PL_CDR, 멤버마다 ID+길이)" 형식을 더한 것. ROS 2가 쓰는 방식이다.
- **XCDR2**(Extended CDR 버전 2): 최대 정렬을 4로 줄이고, 타입 진화를 위해 본문 길이 헤더(DHEADER)와 멤버별 헤더(EMHEADER)를 도입한 개선판. 구조가 바뀐 타입끼리도 안전하게 통신하기 위한 형식이다.

## 설계
- **I/O-free core**: 소켓, 시계, 스레드를 직접 쓰지 않고 입력을 받아 출력(보낼 패킷, 타이머, 이벤트)만 돌려주는 프로토콜 코어. 커뮤니티에서는 "sans-IO"라고도 부른다. 자세한 내용은 [roadmap 설계 원칙](roadmap.md#4-설계-원칙).
