# 프로젝트 규약

[roadmap](../roadmap.md) | [CLAUDE.md](../../CLAUDE.md)

모든 장에서 지키는 규칙이다. 대상마다 문서를 나눈다. 문서 갱신 규칙 (현재 상태만 적기, 결정 형식, 진행 기록)은 [CLAUDE.md](../../CLAUDE.md)에 있다.

| 문서 | 대상 |
|---|---|
| [rust.md](rust.md) | Rust 코드: 툴체인, 형식과 정적 검사, 테스트 배치, 설계 제약 |
| [typescript.md](typescript.md) | 보조 스크립트 (TypeScript + Deno) |
| [markdown.md](markdown.md) | 문서: 문자, 규격 인용, 다이어그램 (Mermaid) |
| [captures.md](captures.md) | 패킷 캡처 파일과 캡처 메모 |
| [book.md](book.md) | 교재 (`book/`): 장 틀, 번호, 코드 스냅샷, 언어, 라이선스 |

## 공통
- **라이선스**: `MIT OR Apache-2.0` 이중 라이선스. 파일은 `LICENSE-MIT`, `LICENSE-APACHE`.
  - 왜: Rust 생태계의 표준 관례 (Rust 자체, tokio, serde 등). 사용자가 둘 중 하나를 고른다. Apache-2.0은 특허 허락 조항이 있고, MIT는 GPLv2 프로젝트와도 섞어 쓸 수 있다.
  - 참고 구현 중 Cyclone DDS는 EPL-2.0이다. 읽고 배우는 것은 괜찮지만 코드를 그대로 옮겨 오지 않는다.
  - 교재 (`book/`) 본문은 CC BY-SA 4.0이다. 코드는 그대로 `MIT OR Apache-2.0` ([book.md](book.md#라이선스)).
- **문자**: 문서, 커밋 메시지, 코드 주석에는 ASCII와 한글만 쓴다. 대체 표기는 [markdown.md](markdown.md#문자).
- **규격 근거**: 규격 내용은 기억이 아니라 `omg-specs/`의 원문으로 확인하고 절 번호를 적는다. 코드 주석은 `// RTPS 2.5 Sec. 9.4.5.3`, 문서는 `(RTPS 2.5 Sec. 9.4.5.3)`.
- **CI**: 원격 저장소를 연결하면 GitHub Actions로 각 문서의 "통과해야 하는 검사"를 돌린다.
