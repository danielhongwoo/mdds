# 조사 노트: DDSI-RTPS 2.5 와이어 프로토콜

[roadmap](../roadmap.md)

- 조사일: 2026-10-27
- 출처: DDSI-RTPS 2.5 PDF 원문 (https://www.omg.org/spec/DDSI-RTPS/2.5/PDF, 텍스트 추출본: `omg-specs/ddsi-rtps-2.5.txt`)
- 표시: 따로 표시가 없으면 원문 확인. [추정] 조사자 판단 또는 미확인

## 0. 버전
- OMG 페이지에는 2.5 (2022-04, 최신), 2.3 (2019-05), 2.2 (2014-09), 2.1, 2.0이 있다. 2.4는 목록에 없지만 2.5 PDF 앞부분에 "Revision 2.4" 태스크포스가 언급된다.
- 와이어 헤더에 프로토콜 버전이 들어간다. 구현들은 2.1 ~ 2.5를 보낸다. 수신 측은 2.x를 모두 받아야 한다.

## 1. 규격 구조 (2.5 PDF 기준)
- **8.2 Structure Module**: Entity, GUID, Participant, Endpoint, Writer/Reader, HistoryCache (8.2.2), CacheChange (8.2.3)
- **8.3 Messages Module**: Header (8.3.3.1, 8.3.6), HeaderExtension (8.3.7), Receiver (8.3.4), SubmessageElements (8.3.5), Submessages (8.3.8)
- **8.4 Behavior Module**
  - 8.4.2 상호운용 요구사항 (먼저 읽는다)
  - 8.4.7, 8.4.9: Writer 참조 구현 (StatelessWriter, ReaderLocator, StatefulWriter, ReaderProxy)
  - 8.4.10 ~ 8.4.12: Reader 쪽 (WriterProxy, Reader 동작)
  - 8.4.13: Liveliness
  - 8.4.15: 구현 가이드
- **8.5 Discovery Module**: SPDP 8.5.3, SEDP 8.5.4, 새 참가자 발견 8.5.5.1
- **9 UDP PSM**: 9.3 타입 매핑, 9.4 와이어 레이아웃 (9.4.2 서브메시지 요소, 9.4.5 서브메시지), 9.6.2 포트와 기본 로케이터, 9.6.3 내장 엔드포인트 IDL과 PID 표, 9.6.4 인라인 QoS PID
- **10 Serialized payload**: 10.2 표현 식별자와 옵션, 10.3 디스커버리 페이로드

## 2. 기본 타입
- **GUID** (8.2.4.1, 9.3.1): 16바이트 = GuidPrefix (12) + EntityId (4)
  - EntityId = entityKey (3) + entityKind (1). kind 상위 2비트: 00 사용자 정의, 01 벤더 전용, 11 내장
  - kind 값 (Table 9.1): 사용자 writer 키 있음 0x02, writer 키 없음 0x03, reader 키 없음 0x04, reader 키 있음 0x07. 내장은 같은 하위 비트에 0xc0을 OR (0xc2, 0xc3, 0xc4, 0xc7). participant는 0xc1
  - GuidPrefix 구성은 구현 자유. 보통 vendorId + 호스트 + 프로세스 정보 [추정]
- **내장 EntityId** (Table 9.2):

| 엔티티 | EntityId |
|---|---|
| Participant | 00 00 01 c1 |
| SPDP writer / reader | 00 01 00 c2 / 00 01 00 c7 |
| SEDP topics writer / reader | 00 00 02 c2 / 00 00 02 c7 |
| SEDP publications writer / reader | 00 00 03 c2 / 00 00 03 c7 |
| SEDP subscriptions writer / reader | 00 00 04 c2 / 00 00 04 c7 |
| P2P participant message writer / reader | 00 02 00 c2 / 00 02 00 c7 |
| ENTITYID_UNKNOWN | 00 00 00 00 |

  00 03 xx c3/c4는 XTypes 예약, 00 02 01 c3/c4는 Security 예약 (Table 9.3)

- **SequenceNumber** (9.4.2.5): int64를 high (i32) + low (u32)로 보낸다. 1부터 시작. SEQUENCENUMBER_UNKNOWN = {-1, 0}
- **SequenceNumberSet** (9.4.2.6): bitmapBase (SN), numBits (u32, 최대 256), ceil(numBits/32)개의 32비트 워드. **bit 0은 첫 워드의 MSB** (버그 단골)
- **Locator_t** (9.3.2.4): kind (i32), port (u32), address (16바이트). UDPv4 = 1 (주소는 마지막 4바이트), UDPv6 = 2
- **Time_t** (9.4.2.9): seconds (i32) + fraction (u32, 1/2^32초 단위). Duration_t (9.3.2.3)도 같은 구조
- **ProtocolVersion** (9.4.2.4): major, minor 각 1바이트. **VendorId** (9.4.2.3): 2바이트. Cyclone 0x0110, Fast DDS 0x010F [추정, OMG 벤더 목록 확인 필요]
- **ChangeKind** (8.2.3): ALIVE, ALIVE_FILTERED, NOT_ALIVE_DISPOSED, NOT_ALIVE_UNREGISTERED
- **CacheChange**: kind, writerGuid, instanceHandle, sequenceNumber, data_value. **HistoryCache**: add_change, remove_change, get_seq_num_min / max (8.2.2)

## 3. 메시지 형식
- **Header** (9.4.4, 8.3.3.1), 20바이트: "RTPS" (4) + ProtocolVersion (2) + VendorId (2) + GuidPrefix (12). 2.5에서 HeaderExtension 서브메시지 (8.3.7)가 선택 사항으로 추가됨
- **서브메시지 헤더** (9.4.5.1), 4바이트: id (u8), flags (u8), octetsToNextHeader (u16)
  - flag 0x01 = E (엔디안). E=1이면 리틀엔디안. **서브메시지마다** 따로 정한다.
  - octetsToNextHeader가 0이고 PAD / INFO_TS가 아니면 메시지 끝까지가 이 서브메시지다.
  - 0x80 ~ 0xff는 벤더 전용 ID, 0x30 ~ 0x3f는 Security 예약. 모르는 ID는 길이로 건너뛴다.
- **서브메시지 ID** (9.4.5.1.1):

| ID | 이름 | 용도 |
|---|---|---|
| 0x00 | RTPS_HE | HeaderExtension |
| 0x01 | PAD | 패딩 |
| 0x06 | ACKNACK | reader의 확인 / 재전송 요청 |
| 0x07 | HEARTBEAT | writer가 보유 범위 알림 |
| 0x08 | GAP | 해당 없는 SN 알림 |
| 0x09 | INFO_TS | 원본 타임스탬프 |
| 0x0c | INFO_SRC | 출발지 prefix / 버전 변경 |
| 0x0d | INFO_REPLY_IP4 | 응답 로케이터 (UDPv4) |
| 0x0e | INFO_DST | 목적지 GuidPrefix |
| 0x0f | INFO_REPLY | 응답 로케이터 |
| 0x12 | NACK_FRAG | 단편 재전송 요청 |
| 0x13 | HEARTBEAT_FRAG | 단편 보유 알림 |
| 0x15 | DATA | 샘플 |
| 0x16 | DATA_FRAG | 단편화된 샘플 |

- **주요 플래그**
  - DATA (9.4.5.4): E 0x01, Q (인라인 QoS) 0x02, D (데이터) 0x04, K (키) 0x08, N (비표준 페이로드) 0x10. D와 K 동시 설정은 무효
    - 레이아웃: extraFlags (u16), octetsToInlineQos (u16), readerId, writerId, writerSN, [inlineQos], [payload]
    - octetsToInlineQos를 항상 따른다 (보통 16)
  - HEARTBEAT (9.4.5.7): F (final) 0x02, L (liveliness) 0x04. 필드: readerId, writerId, firstSN, lastSN, count
  - ACKNACK: F (final) 0x02. 필드: readerId, writerId, readerSNState (SequenceNumberSet), count
  - GAP (9.4.5.6): readerId, writerId, gapStart, gapList (SequenceNumberSet)
- **Receiver 상태** (8.3.4): sourceVersion, sourceVendorId, sourceGuidPrefix, destGuidPrefix, 유니캐스트 / 멀티캐스트 응답 로케이터, haveTimestamp, timestamp
  - 메시지 시작 시 헤더로 초기화하고, INFO_TS / INFO_SRC / INFO_DST / INFO_REPLY가 이후 서브메시지의 상태를 바꾼다.
  - destGuidPrefix 기본값은 수신 참가자. INFO_DST가 다른 prefix를 가리키면 이후 서브메시지를 버린다.
- **ParameterList** (9.4.2.11): pid (u16), length (u16, 4의 배수), 값 (length까지 패딩). PID_SENTINEL (0x0001, 길이 무시)로 끝난다. PID_PAD = 0x0000
  - 모르는 PID는 건너뛴다. 단 must-understand 비트 (0x4000)가 붙은 모르는 PID는 해당 서브메시지를 무시한다. HeaderExtension 안이면 메시지 전체를 무시한다.
  - 0x8000 ~ 0xffff는 벤더 전용
  - 인라인 QoS PID (9.6.4): PID_KEY_HASH 0x0070, PID_STATUS_INFO 0x0071 (dispose / unregister 비트), 그 밖에 CONTENT_FILTER_INFO, COHERENT_SET
- **SerializedPayload** (10장): 4바이트 헤더 = 표현 ID (2) + options (2). CDR_BE 0x0000, CDR_LE 0x0001, PL_CDR_BE 0x0002, PL_CDR_LE 0x0003. 디스커버리 데이터는 PL_CDR

## 4. 동작 (8.4)
- Stateless writer / reader (8.4.8, 8.4.11): 피어별 상태 없음. writer는 ReaderLocator 목록만 가진다. SPDP가 사용
- Stateful writer / reader (8.4.9, 8.4.12): 매칭마다 ReaderProxy / WriterProxy
- Best-effort (8.4.9.1, 8.4.12.1): writer는 DATA (해당 없는 변경은 GAP). reader는 마지막보다 큰 SN만 받고 확인하지 않는다.
- Reliable writer (8.4.9.2): ReaderProxy마다 changes_for_reader 상태 (UNSENT, UNACKNOWLEDGED, REQUESTED, ACKNOWLEDGED)와 highestSentSeqNum. 이것으로 HEARTBEAT, ACKNACK에 따른 재전송, GAP을 결정한다.
- Reliable reader (8.4.12.2): WriterProxy마다 SN별 ChangeFromWriter 상태 (MISSING, RECEIVED, LOST, UNKNOWN)
  - HEARTBEAT를 받으면 firstSN 미만은 LOST, lastSN까지는 MISSING으로 표시
  - heartbeatResponseDelay 뒤에 ACKNACK. F 플래그가 있고 빠진 것이 없으면 응답하지 않는다.
  - 순서대로 전달한다.
- 타이밍 파라미터 (8.4.7.1, 8.4.10.1): heartbeatPeriod, nackResponseDelay, nackSuppressionDuration, heartbeatResponseDelay, initialHeartbeatDelay. 기본값은 구현 자유 (SPDP 주기 제외)
- Count (8.4.15.7): HEARTBEAT / ACKNACK의 count는 단조 증가. 오래된 count는 무시
- 단편화: DATA_FRAG에 fragmentStartingNum (1부터), fragmentsInSubmessage, fragmentSize, sampleSize, 페이로드 조각. reader가 재조립. NACK_FRAG / HEARTBEAT_FRAG가 reliable 대응

## 5. 디스커버리 (8.5, 9.6.3.2)
- **SPDP** (8.5.3): best-effort, stateless 내장 writer / reader. SPDPdiscoveredParticipantData를 PL_CDR ParameterList로 보낸다.
  - PID_PROTOCOL_VERSION 0x15, PID_VENDORID 0x16, PID_PARTICIPANT_GUID 0x50, PID_PARTICIPANT_LEASE_DURATION 0x02
  - PID_DEFAULT_UNICAST_LOCATOR 0x31, PID_DEFAULT_MULTICAST_LOCATOR 0x48
  - PID_METATRAFFIC_UNICAST_LOCATOR 0x32, PID_METATRAFFIC_MULTICAST_LOCATOR 0x33
  - PID_BUILTIN_ENDPOINT_SET 0x58
  - 그 밖에 PID_DOMAIN_ID 0x0f, PID_EXPECTS_INLINE_QOS 0x43, PID_USER_DATA 0x2c, PID_ENTITY_NAME 0x62, PID_PROPERTY_LIST 0x59
  - 30초마다 공지 (9.6.2.4). lease가 끝나면 참가자를 잃은 것으로 본다. 실제 기본값은 구현마다 다르다: Cyclone `Discovery/LeaseDuration` 10초 (SPDP 주기는 그 80%인 8초), Fast DDS lease 20초, 공지 주기 3초 (2026-10-28 공식 문서 확인)
  - 키만 있는 DATA (K=1)는 PID_PARTICIPANT_GUID만 담는다. dispose에 쓴다.
- **SEDP** (8.5.4): reliable, stateful, transient-local 내장 엔드포인트 (publications, subscriptions, topics). DiscoveredWriterData / DiscoveredReaderData를 ParameterList로 보낸다.
  - PID_ENDPOINT_GUID 0x5a, PID_TOPIC_NAME 0x05, PID_TYPE_NAME 0x07
  - PID_RELIABILITY 0x1a, PID_DURABILITY 0x1d, PID_HISTORY 0x40, PID_DEADLINE 0x23, PID_LIVELINESS 0x1b, PID_OWNERSHIP 0x1f, PID_PARTITION 0x29, PID_DESTINATION_ORDER 0x25
  - PID_UNICAST_LOCATOR 0x2f, PID_MULTICAST_LOCATOR 0x30, PID_EXPECTS_INLINE_QOS 0x43
  - 매칭: 토픽 이름과 타입 이름 일치, reliability / durability 호환 (요청 <= 제공), 파티션 호환
- **BuiltinEndpointSet** (9.3.2.12, PID 0x58, u32 비트마스크):

| 비트 | 엔드포인트 |
|---|---|
| 0 | participant announcer |
| 1 | participant detector |
| 2 | publications announcer |
| 3 | publications detector |
| 4 | subscriptions announcer |
| 5 | subscriptions detector |
| 6 ~ 9 | 2.4에서 deprecated |
| 10 | participant message data writer |
| 11 | participant message data reader |
| 12 ~ 15 | XTypes |
| 16 ~ 27 | Security |
| 28 | topics announcer |
| 29 | topics detector |

  최소 구성은 0x3f. liveliness를 구현하면 비트 10, 11 추가

## 6. UDP 포트 (9.6.2, Table 9.15, 9.16)
기본값: PB=7400, DG=250, PG=2, d0=0, d1=10, d2=1, d3=11

| 용도 | 공식 | 도메인 0, participantId 0 |
|---|---|---|
| 디스커버리 멀티캐스트 (SPDP) | PB + DG*domainId + d0 | 7400 |
| 디스커버리 유니캐스트 (SPDP, SEDP) | PB + DG*domainId + d1 + PG*participantId | 7410 |
| 사용자 멀티캐스트 | PB + DG*domainId + d2 | 7401 |
| 사용자 유니캐스트 | PB + DG*domainId + d3 + PG*participantId | 7411 |

- 기본 멀티캐스트 주소 239.255.0.1. 모든 참가자가 여기로 공지하고 듣는다 (9.6.2.4).
- SEDP는 기본적으로 SPDP와 같은 포트를 쓰고, 실제 포트는 SPDP 데이터의 metatraffic 로케이터로 알린다.
- participantId는 0부터 유니캐스트 포트 바인딩을 시도해 정한다 [추정]
- 사용하는 모든 인터페이스에서 멀티캐스트 그룹에 가입하고, 멀티캐스트 소켓에 SO_REUSEADDR / SO_REUSEPORT를 설정한다.

## 7. 버전 차이 (일부 [추정])
- 2.2 -> 2.3: 주로 명확화와 Security 관련 예약 (엔티티 ID, 비트 16 ~ 27, 서브메시지 0x30 ~ 0x3f), XTypes 엔드포인트 ID
- 2.4: 내장 엔드포인트 비트 6 ~ 9 deprecated (2.5 원문에서 확인). 나머지 [추정]
- 2.5: RTPS_HE (0x00) HeaderExtension 추가 (8.3.7, 9.4.5.2). 포트, SPDP / SEDP, 다른 서브메시지는 그대로
- 상호운용 관점에서 2.1 ~ 2.5의 와이어 핵심은 같다. 모든 2.x 버전을 받고 모르는 서브메시지는 무시한다.

## 8. 상호운용 함정 [추정]
- 피어가 벤더 전용 PID를 넣는다. 길이로 건너뛴다.
- 새 매칭이 생기면 transient-local SEDP 이력이 다시 전송될 것을 기대한다.
- Cyclone / Fast DDS의 사용자 데이터는 버전과 타입에 따라 XCDR1 또는 XCDR2다. 표현 ID를 확인한다.
- Wireshark RTPS 필드 문서: https://www.wireshark.org/docs/dfref/r/rtps.html
