# DDS를 직접 만들며 배우기

OMG DDS를 Rust로 처음부터 구현하면서 규격과 동작을 배우는 교재다. 참고 구현인 Cyclone DDS와 Fast DDS를 컨테이너로 띄워 실제 트래픽을 보고, 규격을 읽고, 같은 일을 하는 코드를 직접 쓰고, 두 벤더와 통신이 되는지로 확인한다.

본문 라이선스는 [CC BY-SA 4.0](LICENSE.md)이고, 코드 (chapter 안의 `code/` 포함)는 저장소와 같은 `MIT OR Apache-2.0`이다.

## 누구를 위한 책인가
- Rust 코드를 읽고 쓸 수 있다 (소유권, trait, 모듈 정도).
- IP, UDP, 멀티캐스트가 무엇인지 안다. 패킷을 Wireshark로 본 적이 있으면 좋지만 없어도 된다.
- DDS는 몰라도 된다. 이 책이 처음부터 설명한다.

## 이 책의 방식
chapter마다 같은 순서로 간다.
1. **왜 필요한가**: 이 chapter의 기능이 없으면 무엇이 안 되는지.
2. **캡처에서 보기**: 두 벤더가 주고받은 실제 패킷 ([captures](../captures/)) 에서 그 기능을 찾는다.
3. **규격 읽기**: 읽을 절을 정해 주고, 핵심 문장을 짧게 인용해 풀어 쓴다.
4. **구현**: mdds에 그 기능을 넣는다. chapter가 끝난 시점의 코드가 `code/`에 있다.
5. **벤더와 맞춰 보기**: Cyclone DDS, Fast DDS와 실제로 통신시켜 확인한다.
6. **두 벤더는 어떻게 하는가**: 같은 기능을 두 벤더가 어떻게 구현했는지 소스와 동작을 살핀다.
7. **연습 문제**: 캡처에서 보이지만 본문이 다루지 않은 것들.

직접 실행해 보는 부분은 **lab**으로 나눈다. Lab 1-1은 chapter 1의 첫 번째 lab이다. lab마다 목표, 준비, 단계, 기대 결과, 확인 질문이 있어서 무엇이 보이면 성공인지 알 수 있다.

벤더 DDS는 관찰 대상이자 검증 기준이다. 벤더 사용법을 가르치는 책은 아니다.

## 준비물
- Docker Desktop (Mac arm64 기준. 다른 Docker 호스트도 된다): 벤더 DDS 두 개를 컨테이너로 띄운다.
- Deno 2: 캡처 스크립트 실행.
- Rust (버전은 저장소의 `rust-toolchain.toml`이 정한다).
- Wireshark: 캡처를 열어 본다.
설치와 설정은 chapter 1에서 한다.

## 목차
상태: 집필 전 / 초안 / 완료. 뒤쪽 chapter의 제목과 나눔은 그 chapter의 작업을 하면서 바뀔 수 있다. 저자 노트는 chapter의 작업 계획이다 (독자는 읽지 않아도 된다).

| chapter | 제목 | 저자 노트 | 상태 |
|---|---|---|---|
| 00 | 들어가며: DDS란 무엇이고 왜 직접 만드는가 | [introduction](../docs-for-agent/notes/introduction.md) | 집필 전 |
| 01 | 실습 환경: 벤더 DDS 두 개를 컨테이너로 | [environment-and-captures](../docs-for-agent/notes/environment-and-captures.md) | 집필 전 |
| 02 | 멀티캐스트가 오가는 길 | [environment-and-captures](../docs-for-agent/notes/environment-and-captures.md) | 집필 전 |
| 03 | 첫 DDS 통신 들여다보기 | [environment-and-captures](../docs-for-agent/notes/environment-and-captures.md) | 집필 전 |
| 04 | 같은 규격, 다른 구현 | [environment-and-captures](../docs-for-agent/notes/environment-and-captures.md) | 집필 전 |
| 05 | DCPS 개념 모델 | [dcps-concepts](../docs-for-agent/notes/dcps-concepts.md) | 집필 전 |
| 06 | CDR 직렬화 | [cdr](../docs-for-agent/notes/cdr.md) | 집필 전 |
| 07 | RTPS 메시지 | [rtps-messages](../docs-for-agent/notes/rtps-messages.md) | 집필 전 |
| 08 | Best-effort 전송과 UDP | [best-effort-udp](../docs-for-agent/notes/best-effort-udp.md) | 집필 전 |
| 09 | 참가자 디스커버리 (SPDP) | [spdp](../docs-for-agent/notes/spdp.md) | 집필 전 |
| 10 | Reliable 프로토콜 | [reliable](../docs-for-agent/notes/reliable.md) | 집필 전 |
| 11 | 엔드포인트 디스커버리와 매칭 (SEDP): 첫 상호운용 | [sedp-matching](../docs-for-agent/notes/sedp-matching.md) | 집필 전 |
| 12 | ROS 2와 통신하기 | [ros2-interop](../docs-for-agent/notes/ros2-interop.md) | 집필 전 |
| 13 | 키 인스턴스와 수명주기 | [instances](../docs-for-agent/notes/instances.md) | 집필 전 |
| 14 | DCPS API와 OMG 상호운용 테스트 | [dcps-api](../docs-for-agent/notes/dcps-api.md) | 집필 전 |
| 15 | 단편화와 성능 | [fragmentation-perf](../docs-for-agent/notes/fragmentation-perf.md) | 집필 전 |
| 16 | 타입 시스템 (XTypes) | [xtypes](../docs-for-agent/notes/xtypes.md) | 집필 전 |
| 17 | IDL 컴파일러 | [idl-compiler](../docs-for-agent/notes/idl-compiler.md) | 집필 전 |
| 18 | DDS-Security | [security](../docs-for-agent/notes/security.md) | 집필 전 |
| 19 | 그 밖의 것 (선택) | [extras](../docs-for-agent/notes/extras.md) | 집필 전 |

## 저자를 위한 안내
chapter를 쓰는 규칙 (틀, 번호, 코드 스냅샷, 언어, 라이선스)은 [conventions/book.md](../docs-for-agent/conventions/book.md)에 있다. 작업 계획과 결정은 [docs-for-agent/notes/](../docs-for-agent/notes/)에 두고, 이 디렉터리에는 독자용 본문만 둔다.
