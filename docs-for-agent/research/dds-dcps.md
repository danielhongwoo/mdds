# 조사 노트: DDS 1.4 (DCPS)와 규격 목록

[roadmap](../roadmap.md)

- 조사일: 2026-10-27
- 출처: omg.org/spec 페이지, DDS 1.4 PDF 원문 (텍스트 추출본: `omg-specs/dds-1.4.txt`)
- 표시: [확인] 원문 확인, [추정] 조사자 판단 또는 미확인

## 1. OMG DDS 규격 목록

| 규격 | 최신 정식판 | URL | 범위 |
|---|---|---|---|
| DDS (DCPS) | 1.4, 2015-03 (formal/15-04-10) | https://www.omg.org/spec/DDS/1.4/PDF | API 모델 (PIM), QoS, 내장 토픽, IDL PSM |
| DDSI-RTPS | 2.5, 2022-04 (formal/22-04-01) | https://www.omg.org/spec/DDSI-RTPS/2.5/PDF | 와이어 프로토콜, 메시지 형식, Writer/Reader 상태 머신, SPDP/SEDP, UDP PSM |
| DDS-XTypes | 1.3, 2020-02. 1.4 beta 진행 중 (2026-08) | https://www.omg.org/spec/DDS-XTypes/1.3 | 타입 시스템, 할당 가능성, XCDR1/XCDR2, TypeObject, DynamicType |
| IDL | 4.2, 2018-03 (formal/18-01-05). ISO/IEC 19516 판: formal/22-06-02 | https://www.omg.org/spec/IDL/4.2/PDF | 타입 정의 언어, 어노테이션 |
| DDS-Security | 1.2 (formal/25-03-06), 1.1 (formal/2018-04-01) [확인] | https://www.omg.org/spec/DDS-SECURITY/ | 인증, 접근제어, 암호화, 로깅 플러그인, 보안 디스커버리 |
| DDS-RPC | 1.0, 2017-04 (formal/17-04-01) | https://www.omg.org/spec/DDS-RPC/ | DDS 토픽 위의 요청/응답, RPC |
| DDS-XML | 1.0, 2018-12. 1.1 beta 진행 중 (2026-08) | https://www.omg.org/spec/DDS-XML/ | QoS 프로파일, 엔티티 구성의 XML 문법 |
| DDS-TSN | 1.0 (2026-09 formal로 표시, [추정]) | https://www.omg.org/spec/DDS-TSN/ | DDS를 Time-Sensitive Networking에 매핑 |
| DDS-WEB | 1.0, 2016-02 | https://www.omg.org/spec/DDS-WEB/ | REST/HTTP로 DDS 접근 |
| DDS-PSM-Cxx | 1.0, 2013-11 (formal/13-11-01) | https://www.omg.org/spec/DDS-PSM-Cxx/ | C++11 API |
| DDS-JAVA | 1.0, 2013-11 (formal/13-11-02) | https://www.omg.org/spec/DDS-JAVA/ | Java API |

- DLRL: DDS 1.4 PDF에는 DLRL 장이 없다. 사실상 사용되지 않는다.
- Rust PSM: 없다. "IDL4 -> Rust 언어 매핑" RFP (mars/2026-3-6)만 있다 ([roadmap](../roadmap.md) 참고).

## 2. DCPS 핵심 (DDS 1.4 2장)

### 엔티티 (Sec. 2.2.2)
- DomainParticipantFactory (싱글턴) -> DomainParticipant
- DomainParticipant -> Publisher, Subscriber, Topic, ContentFilteredTopic, MultiTopic
- Publisher -> DataWriter, Subscriber -> DataReader

### 알림
- Listener: 콜백
- Condition: StatusCondition, GuardCondition, ReadCondition, QueryCondition. WaitSet으로 기다린다.
- 통신 상태: Sec. 2.2.4.1

### 인스턴스와 샘플 상태 (Sec. 2.2.2.5.1)
- sample_state: READ / NOT_READ
- view_state: NEW / NOT_NEW
- instance_state: ALIVE / NOT_ALIVE_DISPOSED / NOT_ALIVE_NO_WRITERS
- SampleInfo에는 generation count와 rank도 들어 있다.

### QoS 정책 (Sec. 2.2.3, PDF 92 ~ 101쪽)

RxO: 요청(reader) / 제공(writer) 호환성 검사 대상인지. 와이어 열은 [추정]이며 RTPS 9.6.3 PID 표와 대조해야 한다.

| QoS | 적용 엔티티 | RxO | 변경 가능 | 와이어 |
|---|---|---|---|---|
| USER_DATA | Participant, DataReader, DataWriter | No | Yes | 디스커버리 |
| TOPIC_DATA | Topic | No | Yes | 디스커버리 |
| GROUP_DATA | Publisher, Subscriber | No | Yes | 디스커버리 |
| DURABILITY | Topic, DataReader, DataWriter | Yes | No | 디스커버리 |
| DURABILITY_SERVICE | Topic, DataWriter | No | No | 디스커버리 |
| PRESENTATION | Publisher, Subscriber | Yes | No | 디스커버리 |
| DEADLINE | Topic, DataReader, DataWriter | Yes | Yes | 디스커버리 |
| LATENCY_BUDGET | Topic, DataReader, DataWriter | Yes | Yes | 디스커버리 |
| OWNERSHIP | Topic, DataReader, DataWriter | Yes | No | 디스커버리 |
| OWNERSHIP_STRENGTH | DataWriter | N/A | Yes | 디스커버리 (인라인 QoS 가능) |
| LIVELINESS | Topic, DataReader, DataWriter | Yes | No | 디스커버리 + liveliness 메시지 |
| TIME_BASED_FILTER | DataReader | N/A | Yes | 디스커버리, 필터는 reader 쪽에서 가능 |
| PARTITION | Publisher, Subscriber | No | Yes | 디스커버리 (이름 매칭) |
| RELIABILITY | Topic, DataReader, DataWriter | Yes | No | 디스커버리, reliable / best-effort 상태 머신 선택 |
| TRANSPORT_PRIORITY | Topic, DataWriter | N/A | Yes | 로컬 (전송 계층 힌트) |
| LIFESPAN | Topic, DataWriter | N/A | Yes | 로컬 (writer 쪽) |
| DESTINATION_ORDER | Topic, DataReader, DataWriter | Yes | No | 디스커버리 |
| HISTORY | Topic, DataReader, DataWriter | No | No | 로컬 (캐시) |
| RESOURCE_LIMITS | Topic, DataReader, DataWriter | No | No | 로컬 |
| ENTITY_FACTORY | Factory, Participant, Publisher, Subscriber | No | Yes | 로컬 |
| WRITER_DATA_LIFECYCLE | DataWriter | N/A | Yes | 로컬 (dispose / unregister 결과는 STATUS_INFO로 전송) |
| READER_DATA_LIFECYCLE | DataReader | N/A | Yes | 로컬 |

## 3. 적합성 프로파일 (Annex A, 163쪽) [확인]

| 프로파일 | 추가 내용 |
|---|---|
| Minimum | 필수 DCPS 기능만. 선택 기능 없음 |
| Content-subscription | ContentFilteredTopic, QueryCondition, MultiTopic. SQL 서브셋은 Annex B (165쪽) |
| Persistence | DURABILITY_SERVICE, DURABILITY의 TRANSIENT / PERSISTENT |
| Ownership | OWNERSHIP EXCLUSIVE, OWNERSHIP_STRENGTH, HISTORY depth > 1 |
| Group access | PRESENTATION access_scope GROUP |

Object model 프로파일은 DLRL과 함께 예전 판에만 있었다.

## 4. DDS 1.4 섹션 지도 (PDF 목차 기준) [확인]

| 내용 | 섹션 | PDF 쪽 |
|---|---|---|
| 전체 개념 모델 | Sec. 2.2.1.2.2 | 7 |
| Infrastructure: Entity, Listener, WaitSet, Condition | Sec. 2.2.2.1 | 11 |
| DomainParticipant, DomainParticipantFactory | Sec. 2.2.2.2.1 ~ 2 | 20, 31 |
| Topic, ContentFilteredTopic, MultiTopic | Sec. 2.2.2.3 | 35 ~ 41 |
| Publisher, DataWriter | Sec. 2.2.2.4 | 42 ~ 57 |
| SampleInfo 상태 | Sec. 2.2.2.5.1.1 ~ 8 | 59 ~ 63 |
| Subscriber, DataReader, ReadCondition, QueryCondition | Sec. 2.2.2.5 | 65 ~ 89 |
| QoS 정책 (각 정책 Sec. 2.2.3.1 ~ 22) | Sec. 2.2.3 | 90 |
| Listener, Condition, WaitSet | Sec. 2.2.4 | 113 |
| 내장 토픽 | Sec. 2.2.5 | 128 |
| 상호작용 모델 | Sec. 2.2.6 | 131 |
| IDL PSM | Sec. 2.3 | 137 |
| 적합성 | Annex A | 163 |
| query / filter 문법 | Annex B | 165 |
