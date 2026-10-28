# Phase 14 - DDS-Security (선택)

상태: 대기

[roadmap](../roadmap.md) | 이전: [Phase 13](phase-13-idl-compiler.md) | 다음: [Phase 15](phase-15-extras.md)

## 읽을 규격
- DDS-Security **1.2** 기준 (1.2에서 장 번호가 바뀌었다. 원문 확인함, 상세 지도: [research/xtypes-cdr-idl-security.md](../research/xtypes-cdr-idl-security.md))
  - Sec. 7.4 와이어 보안: 서브메시지 SEC_BODY 0x30, SEC_PREFIX 0x31, SEC_POSTFIX 0x32, SRTPS_PREFIX 0x33, SRTPS_POSTFIX 0x34 (7.4.6.3 ~ 7.4.6.7, 7.4.7.5 ~ 7.4.7.9)
  - Sec. 7.5 보안 내장 토픽 (StatelessMessage 7.5.3, VolatileMessageSecure 7.5.4)
  - Sec. 9 플러그인 구조 (Authentication 9.3, AccessControl 9.4, Cryptographic 9.5)
  - Sec. 10 내장 플러그인 (PKI-DH 10.3, Permissions 10.4, AES-GCM-GMAC 10.5)

## 구현 순서
1. 정적 키 CryptoTransform
2. PKI-DH 핸드셰이크 (ParticipantStatelessMessage, Request -> Reply -> Final, GUID가 작은 쪽이 시작)
3. 키 교환 (ParticipantVolatileMessageSecure)
4. Governance / Permissions (S/MIME 서명)
5. 보안 디스커버리

## 크레이트
- RustCrypto: `aes-gcm`, `p256`, `rsa`, `x509-cert`, `cms`

## 참고
- 1.2에서 RSA-2048과 일부 ECDH / ECDSA가 deprecated 되었고 PSK 플러그인이 추가되었다. 상대 구현 (Cyclone)이 지원하는 버전과 알고리즘에 맞춘다.

## 완료 기준
- Cyclone 보안 활성화 구성과 상호운용

## 진행 기록
- (날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 규칙은 CLAUDE.md "문서 갱신 규칙")
