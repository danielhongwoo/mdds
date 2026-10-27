# mdds 프로젝트 지침

OMG DDS 표준을 Rust로 직접 구현하며 이해하는 학습용 프로젝트다. 검증 기준은 Cyclone DDS / Fast DDS와의 상호운용이다.

## 문서
- `docs/roadmap.md`: 전체 지도와 단계 현황 표. 단계 상태가 바뀌면 이 표와 해당 단계 파일의 "상태:" 줄을 함께 고친다.
- `docs/phases/phase-NN-*.md`: 단계별 계획. 진행하면서 배운 점, 결정 사항, 막힌 점은 그 파일의 "진행 기록"에 날짜와 함께 적는다.
- `docs/research/`: 조사 노트. 원문으로 확인한 내용과 추정을 구분해 표시한다.
- `docs/glossary.md`: 새 용어가 나오면 추가한다.
- `omg-specs/`: OMG 규격 PDF와 `pdftotext -layout` 추출본. git에 넣지 않는다. 규격 내용을 인용할 때는 기억이 아니라 이 원문을 확인한다.

## 코드
- 프로토콜 코어는 I/O-free core로 만든다 (소켓, 시계, 스레드를 직접 쓰지 않는다). 자세한 내용은 `docs/roadmap.md` 설계 원칙.
- 규격 근거를 주석으로 남긴다. 형식: `// RTPS 2.5 Sec. 9.4.5.3`
- 직렬화는 serde를 쓰지 않고 자체 trait으로 만든다.
- 그 밖의 규약 (라이선스, 툴체인, 테스트 배치)은 `docs/phases/phase-00-setup.md`의 "프로젝트 규약"을 따른다.

## 실행 환경
- Cyclone DDS / Fast DDS는 Docker로 실행한다 (`docker/`). OMG shape_main 실행 파일은 x86-64라 `--platform linux/amd64`가 필요하다.
