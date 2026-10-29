# TypeScript 규약 (보조 스크립트)

[규약 목록](README.md)

프로토콜 코드가 아닌 보조 스크립트 (예: [docker/capture-shape-scenario.ts](../../docker/capture-shape-scenario.ts): shape_main 쌍을 두 벤더 컨테이너에서 돌리며 캡처)에 적용한다.

**결정: 보조 스크립트는 TypeScript + Deno로 쓴다.**
- 왜: 설정을 객체로 두면 기본값과 차이가 분리되어 표처럼 읽힌다. 외부 명령의 인자를 배열로 넘기므로 셸의 단어 분리와 이중 따옴표 문제가 없다. Deno는 실행 파일 하나이고, 외부 모듈 없이 표준 API (`Deno.Command`)만으로 충분하다.
- 비용: 호스트에 Deno 2가 필요하다 (`brew install deno`).
- 버린 대안: POSIX sh (설정이 `case` 한 줄에 몰리고 옵션을 문자열로 이어 붙여 읽기 어렵다), Rust로 `tools/`에 두기 (프로젝트 언어와 같지만 프로세스 호출 코드가 길어져 흐름이 오히려 안 보인다).

## 작성
- 외부 모듈을 쓰지 않는다. Deno 표준 API만 쓴다.
- 실행 파일로 둔다. shebang에 필요한 권한만 적는다. 예: `#!/usr/bin/env -S deno run --allow-run=docker --allow-read --allow-write`
- 모든 함수에 반환 타입을 적는다 (함수 머리만 보고 무엇을 돌려주는지 알 수 있게. Rust와 같은 습관). lint 규칙으로 강제한다.

## 파일 이름
**결정: 스크립트 이름은 "하는 일 + 대상"으로 짓는다 (예: `capture-shape-scenario.ts`).**
- 왜: 같은 디렉터리 (`docker/`) 를 여러 chapter에서 계속 쓴다. 이름만 보고 무엇을 하는지 알 수 있어야 하고, 같은 종류가 늘어나면 `capture-<대상>.ts`처럼 나란히 선다. chapter나 번호를 이름에 넣지 않는다. 번호는 바뀌고, 한 스크립트가 여러 chapter에서 쓰인다.
- 버린 대안: chapter 번호 접두 (`ch01-capture.ts`). 번호가 바뀌면 파일 이름과 모든 링크를 고쳐야 한다. 일반 이름 (`run-scenario.ts`, `capture.ts`). 무엇의 시나리오인지, 무엇을 캡처하는지 드러나지 않는다.

## 설정 (`deno.json`)
스크립트가 있는 디렉터리에 둔다. 지금은 [docker/deno.json](../../docker/deno.json) 하나다.
- fmt: 대상은 `*.ts`와 `deno.json`만 (Markdown은 다시 줄바꿈하지 않게 뺀다). 줄 폭 100.
- lint: `recommended` 규칙에 `explicit-function-return-type`을 더한다.

## 통과해야 하는 검사
스크립트 디렉터리에서:
```
deno fmt --check
deno lint
deno check <파일>.ts
```
