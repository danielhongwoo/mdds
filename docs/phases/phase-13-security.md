# Phase 13 - DDS-Security (선택)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 12](phase-12-idl-compiler.md) | 다음: [Phase 14](phase-14-extras.md)

## 읽을 규격
- DDS-Security Sec. 7.3 (와이어 확장: SEC_PREFIX / SEC_BODY / SEC_POSTFIX, SRTPS_PREFIX / SRTPS_POSTFIX)
- Sec. 8 (플러그인 SPI), Sec. 9 (내장 플러그인: PKI-DH, Permissions/Governance XML, AES-GCM-GMAC)

## 구현 순서
1. 정적 키 CryptoTransform
2. PKI-DH 핸드셰이크 (ParticipantStatelessMessage)
3. 키 교환 (ParticipantVolatileMessageSecure)
4. Governance / Permissions (S/MIME 서명)
5. 보안 디스커버리

## 크레이트
- RustCrypto: `aes-gcm`, `p256`, `rsa`, `x509-cert`, `cms`

## 확인할 것
- DDS-Security 1.2의 정식 공개 날짜와 변경점

## 완료 기준
- Cyclone 보안 활성화 구성과 상호운용

## 진행 기록
- (진행하면서 배운 점, 결정 사항, 막힌 점을 날짜와 함께 적는다)
