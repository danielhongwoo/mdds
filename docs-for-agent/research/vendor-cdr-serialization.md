# 조사 노트: 두 벤더의 XCDR1 직렬화 구현

[roadmap](../roadmap.md) | 쓰는 chapter: 6 (6절), 7, 13, 16

- 조사일: 2026-11-04. 고정 커밋의 소스를 직접 읽었다 (실행하지 않음).
- Cyclone DDS 11.0.1 `f2aaccb4` (`git -C ~/practice/cyclonedds show f2aaccb4:<path>`), Fast DDS `v3.6.1` (`4e81e8b7`), Fast-CDR `v2.3.5` (`7d33a3b5`), Fast-DDS-Gen `v4.3.0` (`cc0072b8`). 줄 번호는 그 커밋 기준.
- 확인하지 못한 것은 "확인 못 함"으로 표시했다.

## Cyclone DDS

| 항목 | 확인한 것 | 위치 |
|---|---|---|
| 헤더를 쓰는 곳 | 샘플 버퍼 구조체 `dds_serdata_default`의 `hdr` 필드 (identifier, options 각 u16) 바로 뒤에 `data[]`. `serdata_default_init`이 식별자를 정하고 options를 0으로 둔다. 보낼 때 `serdata_default_to_ser`가 hdr와 data를 한 번에 memcpy | `dds__serdata_default.h` 107-113, `dds_cdrstream.h` 75-78, `dds_serdata_default.c` 189-201, 772-778 |
| 정렬 기준점 | `m_index`는 구조체 시작 기준이지만 `data`의 offset이 8의 배수로 강제된다 (static assert). 그래서 헤더 바로 뒤 기준과 같다. 받을 때 `dds_stream_normalize`는 `d->data`를 offset 0으로 둔다. XCDR1 상한 8, XCDR2 상한 4 | `dds_serdata_default.c` 619-630, `dds_cdrstream.c` 358-374 (`dds_cdr_alignto_clear_and_resize_base`), 259-270, 5538-5548, `dds__serdata_default.h` 113 |
| options 패딩 비트 | 쓴다. `ostream_add_to_serdata_default`가 4의 배수까지 0으로 채우고 그 수를 `hdr.options`에 적는다 (SDK_DATA, SDK_KEY 모두). 받을 때 `options & 0x3`만큼 길이에서 빼고, 길이보다 크면 오류 | `dds_serdata_default.c` 632-643, 396-407, `dds_cdrstream.h` 48 |
| 모르는 식별자 | `is_valid_xcdr_id`의 허용 목록 (CDR, PL_CDR, CDR2, D_CDR2, PL_CDR2의 BE / LE) 밖이면 오류. encoding format이 sertype과 다르면 (final에 PL_CDR) 오류. XCDR1 / 2 모두 받고 버전은 식별자로 정한다. 오류이면 `remote_make_sample`이 "deserialization ... failed" 경고를 남기고 샘플을 버린다 | `dds_serdata_default.c` 229-245, 358-432, `ddsi_sertype.c` 223-283, `ddsi_receive.c` 2015, 2090-2104 |
| 타입 기술 | idlc가 `dds_topic_descriptor_t`의 `m_ops` (opcode 배열)를 만들고 `dds_stream_write_impl` / `dds_stream_normalize`가 해석한다. XCDR1에서 DLC (appendable)는 DHEADER 없이 멤버만 쓴다 | `dds_public_impl.h` 155-172, `dds_opcodes.h` 228, 430, `dds_cdrstream_write.part.h` 909-954, 928-932 |
| 길이 검사 | 읽기 전에 `dds_stream_normalize`가 전체를 검증 (필요하면 바이트 순서도 바꿈). `normalize_string`: 길이 0 오류, 남은 길이 부족 오류, 마지막 바이트 NUL 아니면 오류, bound 초과는 try-construct 정책 (기본값은 확인 못 함) | `dds_cdrstream.c` 3810-3823, 3858-3866, 4126-4160 |
| 틈의 값 | 직렬화기가 틈과 끝 패딩을 직접 0으로 쓴다 | `dds_cdrstream.c` 358-384 |

## Fast DDS (Fast-CDR, Fast-DDS-Gen)

| 항목 | 확인한 것 | 위치 |
|---|---|---|
| 헤더를 쓰는 곳 | fastddsgen이 만든 `<Type>PubSubType::serialize()`가 `Cdr` 객체를 만들고 (`XCDRv1`이면 final / appendable은 PLAIN_CDR), `serialize_encapsulation()`, `ser << value`, `set_dds_cdr_options({0, 0})` 순서로 부른다. 헤더 바이트는 Fast-CDR `Cdr::serialize_encapsulation()`이 쓴다 (0, `encoding_flag | endianness`, options). 표현 QoS가 비어 있으면 XCDR1 | `DDSPubSubTypeSource.stg` 83-118, `Cdr.cpp` 302-345, `CdrEncoding.hpp` 40-48, `DataWriterImpl.cpp` 376-378, 1068 |
| 정렬 기준점 | `serialize_encapsulation()`과 `read_encapsulation()`의 끝에서 `reset_alignment()`가 `origin_ = offset_` (헤더 바로 뒤). `alignment()`는 `(offset_ - origin_) % size`. XCDRv1 `align64_ = 8`, v2는 4 | `Cdr.cpp` 298, 343, 233-259, `Cdr.h` 306-310, 3017-3021 |
| options 패딩 비트 | 쓴다. `set_dds_cdr_options`가 `((len + 3) & ~3) - len`을 options[1]의 하위 2비트에 적는다. 패딩 바이트는 Fast-CDR이 아니라 RTPS 층이 DATA 서브메시지를 4바이트에 맞출 때 0으로 붙인다. 받을 때 `read_encapsulation`이 `end_`를 그만큼 줄인다 | `Cdr.cpp` 380-403, 275-287, `DataMsg.hpp` 338-350, `RTPSMessageGroup.hpp` 376 |
| 모르는 식별자 | `read_encapsulation`이 첫 바이트가 0이 아니거나 flag가 목록 밖이면 `BadParamException`. 생성된 `deserialize()`의 `Cdr`은 기본 XCDRv2라 XCDR1 / 2 식별자를 모두 받는다. 예외는 `deserialize()`가 받아 false. 역직렬화는 read / take 시점이고 (`ReadTakeCommand.hpp`), 실패한 샘플은 history에서 지운다 | `Cdr.cpp` 198-300 (212, 242, 254, 266), `Cdr.h` 165-168, `DDSPubSubTypeSource.stg` 120-150, `ReadTakeCommand.hpp` 148-175, 378-433 |
| 타입 기술 | fastddsgen이 타입마다 `<Type>CdrAux.ipp` (`serialize` / `deserialize` / `calculate_serialized_size`)와 `<Type>PubSubTypes.cxx`를 만든다. XCDR1 PLAIN_CDR이면 `xcdr1_begin_serialize_type` / `end`가 아무 헤더도 쓰지 않는다 (appendable = final) | `TypesCdrAuxHeaderImpl.stg` 53-112, `fastddsgen.java` 1044-1061, `Cdr.cpp` 3147-3177 |
| 길이 검사 | 원시 값마다 남은 길이를 보고 모자라면 `NotEnoughMemoryException`. `read_string`은 길이만 검사하고 NUL이 없어도 오류가 아니다. `fixed_string::assign`은 bound 초과를 자른다. `string<128>`이 생성 코드에서 `fixed_string`이 되는지는 확인 못 함 | `Cdr.cpp` 1372-1403, 1681-1710, `fixed_size_string.hpp` 78-90 |
| 틈의 값 | `make_alignment`는 `offset_ += align`만 한다 (쓰지 않음). 버퍼는 calloc / memset 0으로 받아 캡처에서 0으로 보인다. 풀에서 버퍼를 다시 쓸 때 0으로 채우는 코드는 찾지 못했다 (이전 샘플의 바이트가 남을 수 있음 [추정]) | `Cdr.h` 3027-3032, `TopicPayloadPool.hpp` 146-181, `SerializedPayload.cpp` 104-147 |

## chapter별로 쓰는 것
- chapter6 6절: 위 표 전체.
- chapter7: Cyclone은 받는 시점에 검증하고 Fast DDS는 read / take 시점에 역직렬화한다. 벤더 전용 식별자 (첫 바이트 0xC0 ~ 0xFF)는 둘 다 거부한다.
- chapter13: Cyclone은 key 페이로드 (SDK_KEY)도 같은 경로로 패딩 비트를 채운다.
- chapter16: 두 벤더 모두 한 reader가 XCDR1과 XCDR2 식별자를 받는다. DATA_REPRESENTATION QoS 불일치를 매칭 단계에서 거르는지는 확인 못 함.
