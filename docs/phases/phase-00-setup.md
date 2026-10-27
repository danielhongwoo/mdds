# Phase 00 - 준비 (구현 없음, 관찰 단계)

상태: 대기

[roadmap](../roadmap.md) | 다음: [Phase 01](phase-01-dcps-concepts.md)

## 목표
규격을 읽기 전에 실제 DDS 트래픽을 눈으로 본다. 이때 확보한 캡처가 이후 단계의 테스트 데이터(정답 바이트)가 된다.

## 할 일
- [ ] 도구 설치: Wireshark (`rtps` 필터), Eclipse Cyclone DDS(+`ddsperf`), Fast DDS + ShapesDemo, RTI Shapes Demo(무료), `omg-dds/dds-rtps` 저장소
- [ ] Cyclone <-> Fast DDS 통신 확인
- [ ] 시나리오별 패킷 캡처 후 `tests/fixtures/*.pcapng`로 저장
  - [ ] best-effort
  - [ ] reliable
  - [ ] keyed 토픽의 dispose
  - [ ] 참가자 종료 (lease 만료)
  - [ ] 큰 샘플 (DATA_FRAG)
- [ ] Cargo 워크스페이스 뼈대 생성 (빈 crate만)

## 참고 코드
- Cyclone DDS: RTPS 상태 머신, 디스커버리 로직 읽기 좋음
- RustDDS: Rust식 구조
- dust_dds: XTypes, 워크스페이스 구성

## 주의
- macOS는 루프백 인터페이스에서 멀티캐스트가 제한될 수 있다. 문제가 생기면 Docker의 Linux 컨테이너 두 개에서 실행하고 브리지 네트워크를 캡처한다.

## 완료 기준
- Wireshark 캡처에서 SPDP / SEDP / DATA / HEARTBEAT / ACKNACK를 눈으로 식별할 수 있다.
- 위 시나리오의 캡처가 저장소에 들어 있다.

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
