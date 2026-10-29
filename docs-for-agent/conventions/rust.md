# Rust 규약

[규약 목록](README.md)

## 툴체인
- edition 2024.
- `rust-toolchain.toml`에 stable 버전을 고정하고 (지금 1.99.1) 그 버전을 MSRV로 삼는다. `docker/tools/Dockerfile`의 Rust 이미지 버전도 같게 둔다. 올릴 때는 두 곳을 함께 고친다.

## 통과해야 하는 검사
저장소 루트에서:
```
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
- `rustfmt`는 기본 설정을 쓴다 (`rustfmt.toml` 없음).

## 테스트 배치
- 단위 테스트: 모듈 안 `#[cfg(test)]`.
- pcap 기반 골든 테스트와 상호운용 테스트: 크레이트의 `tests/`. 입력 캡처는 저장소 루트의 `tests/captures/` ([captures.md](captures.md)).

## 설계 제약
- 프로토콜 코어는 I/O-free core로 만든다. 소켓, 시계, 스레드를 직접 쓰지 않는다 ([roadmap 설계 원칙](../roadmap.md#4-설계-원칙)).
- 직렬화는 serde를 쓰지 않고 자체 trait으로 만든다.

## 주석
- 규격 근거를 남긴다. 형식: `// RTPS 2.5 Sec. 9.4.5.3`
- ASCII와 한글만 쓴다.
