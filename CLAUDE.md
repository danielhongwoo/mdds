# mdds 프로젝트 지침

OMG DDS 표준을 Rust로 직접 구현하며 이해하는 학습용 프로젝트다. 검증 기준은 Cyclone DDS / Fast DDS와의 상호운용이다. 그 과정을 다른 학습자도 따라올 수 있게 교재 (`book/`)로 쓴다.

## 문서
세 갈래로 나눈다. 독자용 (`book/`), 공용 참고 (`docs/`), 저자와 agent용 (`docs-for-agent/`).
- `book/`: 학습자용 교재. 장 (chapter) 이 진행 단위이자 읽는 순서다. 장 하나가 디렉터리 하나 (`NN-slug/README.md`, 코드가 있으면 `code/`). 독자용 본문만 둔다. 규칙은 `docs-for-agent/conventions/book.md`.
- `docs/environment/`: 실행 환경 문서 (여러 장에서 같이 쓴다). `vendor-build.md` (벤더 DDS와 shape_main 빌드), `docker-network.md` (실행 호스트, 네트워크, 주소 배정).
- `docs/glossary.md`: 용어. 새 용어가 나오면 추가한다.
- `docs-for-agent/roadmap.md`: 전체 지도와 장 현황 표. 장의 작업 상태가 바뀌면 이 표와 해당 저자 노트의 "상태:" 줄을 함께 고친다.
- `docs-for-agent/notes/<주제>.md`: 장별 저자 노트 (읽을 규격, 구현 체크리스트, 확인할 것, 완료 기준, 진행 기록). 주제 이름으로 두고 번호를 붙이지 않는다. 한 노트가 여러 장을 맡을 수 있다 (예: `lab-setup.md`는 1-4장).
- `docs-for-agent/research/`: 조사 노트. 원문으로 확인한 내용과 추정을 구분해 표시한다.
- `docs-for-agent/conventions/`: 프로젝트 규약. 대상별로 나눈다 (`rust.md`, `typescript.md`, `markdown.md`, `captures.md`, `book.md`).
- `omg-specs/`: OMG 규격 PDF와 `pdftotext -layout` 추출본. git에 넣지 않는다. 규격 내용을 인용할 때는 기억이 아니라 이 원문을 확인한다.
- 문서 갱신 규칙
  - 본문은 항상 현재 상태만 적는다. 바뀌면 덮어쓰고, 폐기된 내용은 지운다. 과거 경위는 git 히스토리 (커밋 메시지)에 둔다.
  - 결정은 그 결정이 적용되는 문서에 "무엇을 / 왜 / 버린 대안과 이유"로 적는다. 버린 대안은 누군가 다시 시도할 만한 것만 적는다.
  - 저자 노트의 "진행 기록"은 한 줄짜리 색인이다: 날짜, 결정이나 막힌 점 한 줄, 근거 문서 링크. 막힌 점은 해결되면 결정으로 옮기거나 지운다.
  - 검토 기록은 따로 두지 않는다. 반영한 지적은 문서에 녹이고, 반영하지 않기로 한 지적만 이유와 함께 결정으로 남긴다.
- 산출물 (스크립트, 도구, 캡처, crate) 은 번호가 아니라 하는 일과 대상으로 이름 짓고, 종류별 디렉터리 (`docker/`, `tools/`, `tests/captures/`, `crates/`) 에 둔다. 장은 그것을 링크한다.

## 코드
- 프로토콜 코어는 I/O-free core로 만든다 (소켓, 시계, 스레드를 직접 쓰지 않는다). 자세한 내용은 `docs-for-agent/roadmap.md` 설계 원칙.
- 규격 근거를 주석으로 남긴다. 형식: `// RTPS 2.5 Sec. 9.4.5.3`
- 직렬화는 serde를 쓰지 않고 자체 trait으로 만든다.
- 그 밖의 규약 (라이선스, 툴체인, 검사 명령, 테스트 배치, 보조 스크립트, 문서 형식, 캡처)은 `docs-for-agent/conventions/`를 따른다.

## 실행 환경
- Cyclone DDS / Fast DDS는 Docker로 실행한다 (`docker/`). shape_main과 두 벤더 라이브러리는 이미지 안에서 소스로 빌드한다 (버전과 Dockerfile 설계는 `docs/environment/vendor-build.md`, 실행 호스트와 네트워크는 `docs/environment/docker-network.md`). **Mac (arm64)의 Docker Desktop**에서 빌드, 실행, 캡처를 모두 하는 것이 기본이다 (네이티브 arm64 빌드). 벤더마다 이미지와 컨테이너가 따로 있다 (`docker/cyclone/`, `docker/fastdds/`). 캡처는 바인드 마운트로 `tests/captures/`에 바로 생긴다. DDS 트래픽은 Docker Desktop VM 안의 bridge에서만 돌고 Mac 호스트는 DDS 통신에 참여하지 않는다. 별도 x86_64 Linux 머신은 선택 사항이다.
