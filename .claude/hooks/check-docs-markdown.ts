#!/usr/bin/env -S deno run --allow-read
/**
 * 독자용 Markdown (book/, docs/) 의 기계적으로 확인할 수 있는 규칙을 검사한다.
 * 규칙의 출처: docs-for-agent/conventions/book.md, markdown.md, 전역 문자 사용 규칙.
 *
 *   ./check-docs-markdown.ts <파일.md>...    파일을 직접 검사한다
 *   (인자 없음)                              stdin 의 Claude Code 훅 입력 (JSON) 에서 파일 경로를 읽는다
 *
 * 종료 코드: 0 = 통과 (주의만 있으면 stdout 에 JSON 으로 알린다), 2 = 위반 (stderr 에 목록).
 * 사실 확인, 객관성처럼 판단이 필요한 규칙은 여기서 보지 않는다 (.claude/skills/chapter-review).
 */

interface Finding {
  line: number;
  message: string;
}

interface Report {
  errors: Finding[];
  warnings: Finding[];
}

// ---------------------------------------------------------------------------
// 대상 파일 고르기
// ---------------------------------------------------------------------------

const ROOT = new URL("../../", import.meta.url).pathname;

function relativePath(path: string): string {
  return path.startsWith(ROOT) ? path.slice(ROOT.length) : path;
}

function isTarget(rel: string): boolean {
  return rel.endsWith(".md") && (rel.startsWith("book/") || rel.startsWith("docs/"));
}

function isChapter(rel: string): boolean {
  return /^book\/\d\d-[^/]+\/README\.md$/.test(rel);
}

async function pathsFromHookInput(): Promise<string[]> {
  const text = await new Response(Deno.stdin.readable).text();
  if (text.trim() === "") return [];
  const input = JSON.parse(text);
  const path = input?.tool_response?.filePath ?? input?.tool_input?.file_path;
  return typeof path === "string" ? [path] : [];
}

// ---------------------------------------------------------------------------
// 규칙
// ---------------------------------------------------------------------------

/** ASCII 와 한글 음절만 허용한다 (전역 문자 사용 규칙). */
function checkCharacters(lines: string[], report: Report): void {
  lines.forEach((line, i) => {
    for (const ch of line) {
      const code = ch.codePointAt(0)!;
      const ascii = code < 0x80;
      const hangul = code >= 0xac00 && code <= 0xd7a3;
      if (!ascii && !hangul) {
        report.errors.push({
          line: i + 1,
          message: `ASCII와 한글 밖의 문자 '${ch}' (U+${
            code.toString(16).toUpperCase().padStart(4, "0")
          }). 대체 표기: ->, /, -, Sec.`,
        });
        return;
      }
    }
  });
}

/** 바꾸기로 정한 표현. 패턴과 고칠 방향. */
const BANNED: { pattern: RegExp; message: string; scope: (rel: string) => boolean }[] = [
  {
    pattern: /C\+*판/,
    message: '"C판", "C++판" 대신 "C 언어 구현", "C++ 언어 구현"',
    scope: () => true,
  },
  {
    pattern: /\d+\s*장(?![a-z가-힣])|챕터/,
    message: '책의 단위는 "장"이나 "챕터"가 아니라 "chapter"',
    scope: (rel) => rel.startsWith("book/"),
  },
  {
    pattern: /chapter \d/,
    message: 'chapter 번호는 띄우지 않는다: "chapter3", "chapter0 3.3" (markdown.md chapter 참조)',
    scope: () => true,
  },
  {
    pattern: /\/Users\/|~\/work\//,
    message: '특정 OS나 개인의 경로 대신 "프로젝트 디렉터리"',
    scope: (rel) => rel.startsWith("book/"),
  },
];

function checkBanned(rel: string, lines: string[], report: Report): void {
  let inFence = false;
  lines.forEach((line, i) => {
    if (line.trimStart().startsWith("```")) inFence = !inFence;
    if (inFence) return;
    for (const rule of BANNED) {
      if (rule.scope(rel) && rule.pattern.test(line)) {
        report.errors.push({ line: i + 1, message: rule.message });
      }
    }
  });
}

/** 근거가 같은 문장에 있어야 쓰는 평가 형용사 (book.md 객관성). 판단은 사람이 한다. */
const EVALUATIVE = ["뛰어난", "강력한", "유일한", "최고의", "인지도", "널리 쓰이는"];

/** 기술서에 맞지 않는 말투 (book.md 언어). 독자의 말을 옮긴 인용은 예외라 판단은 사람이 한다. */
const COLLOQUIAL = [
  "띄우",
  "띄워",
  "띄운",
  "뜨자마자",
  "돈다",
  "돌고 있",
  "죽는다", "죽거나", "죽어도",
  "켜고",
  "켜 두",
];

function checkColloquial(lines: string[], report: Report): void {
  let inFence = false;
  lines.forEach((line, i) => {
    if (line.trimStart().startsWith("```")) inFence = !inFence;
    if (inFence) return;
    for (const word of COLLOQUIAL) {
      if (line.includes(word)) {
        report.warnings.push({
          line: i + 1,
          message: `말투 "${word}": "실행한다", "시작한다" 같은 글말로 바꿀 수 있는지 확인`,
        });
      }
    }
  });
}

function checkEvaluative(lines: string[], report: Report): void {
  lines.forEach((line, i) => {
    for (const word of EVALUATIVE) {
      if (line.includes(word)) {
        report.warnings.push({
          line: i + 1,
          message: `평가 표현 "${word}": 같은 문장에 근거 (출처, 수치)가 있는지 확인`,
        });
      }
    }
  });
}

/** chapter 본문의 절과 소절 번호 (book.md chapter). */
function checkHeadings(lines: string[], report: Report): void {
  lines.forEach((line, i) => {
    if (line.startsWith("## ") && !/^## (\d+\. .+|참고 자료)$/.test(line)) {
      report.errors.push({
        line: i + 1,
        message: '절 제목은 "## N. 제목" (참고 자료만 번호 없음)',
      });
    }
    if (line.startsWith("### ") && !/^### \d+\.\d+ .+$/.test(line)) {
      report.errors.push({
        line: i + 1,
        message: '소절 제목은 "### N.M 제목" (lab은 "### N.M Lab C-N: 제목")',
      });
    }
    if (line.startsWith("#### ") && !/^#### \d+\.\d+\.\d+ .+$/.test(line)) {
      report.errors.push({ line: i + 1, message: '제목은 "#### N.M.K 제목"' });
    }
    if (line.startsWith("##### ") && !/^##### \d+\.\d+\.\d+\.\d+ .+$/.test(line)) {
      report.errors.push({ line: i + 1, message: '제목은 "##### N.M.K.L 제목"' });
    }
  });
}

/** chapter 끝의 참고 자료에는 외부 자료만 (book.md chapter). */
function checkReferences(lines: string[], report: Report): void {
  const start = lines.findIndex((l) => l === "## 참고 자료");
  if (start < 0) return;
  for (let i = start + 1; i < lines.length && !lines[i].startsWith("## "); i++) {
    for (const m of lines[i].matchAll(/\]\(([^)\s]+)\)/g)) {
      if (!/^https?:\/\//.test(m[1])) {
        report.errors.push({
          line: i + 1,
          message: `참고 자료에 저장소 안의 문서 (${m[1]}). 본문의 필요한 자리에 링크한다`,
        });
      }
    }
  }
}

/** 용어 정리는 자기 완결적이다: 책 본문과 저자 문서를 가리키지 않는다 (book.md 용어 정리). */
function checkGlossary(lines: string[], report: Report): void {
  lines.forEach((line, i) => {
    if (/docs-for-agent|\]\(\d\d-|chapter ?\d/.test(line)) {
      report.errors.push({
        line: i + 1,
        message: "용어 정리에서 chapter나 저자 문서를 가리킨다. 규격만 인용한다",
      });
    }
  });
}

/** GitHub 방식의 제목 anchor. */
function slug(heading: string): string {
  return heading.trim().toLowerCase().replace(/[^\p{L}\p{N}_\- ]/gu, "").replaceAll(" ", "-");
}

async function anchorsOf(path: string): Promise<Set<string>> {
  const text = await Deno.readTextFile(path);
  return new Set([...text.matchAll(/^#+ (.*)$/gm)].map((m) => slug(m[1])));
}

/** 상대 링크의 파일과 anchor 가 있는지. */
async function checkLinks(path: string, lines: string[], report: Report): Promise<void> {
  const dir = path.slice(0, path.lastIndexOf("/") + 1);
  for (const [i, line] of lines.entries()) {
    for (const m of line.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = m[1];
      if (/^(https?:|mailto:)/.test(target)) continue;
      const [file, anchor] = target.split("#", 2);
      const resolved = file === "" ? path : new URL(file, `file://${dir}`).pathname;
      let exists = true;
      try {
        await Deno.stat(resolved);
      } catch {
        exists = false;
      }
      if (!exists) {
        report.errors.push({ line: i + 1, message: `링크 대상이 없다: ${target}` });
        continue;
      }
      if (anchor && resolved.endsWith(".md") && !(await anchorsOf(resolved)).has(anchor)) {
        report.errors.push({ line: i + 1, message: `anchor가 없다: ${target}` });
      }
    }
  }
}

// ---------------------------------------------------------------------------
// 실행
// ---------------------------------------------------------------------------

async function checkFile(path: string): Promise<Report> {
  const rel = relativePath(path);
  const lines = (await Deno.readTextFile(path)).split("\n");
  const report: Report = { errors: [], warnings: [] };
  checkCharacters(lines, report);
  checkBanned(rel, lines, report);
  await checkLinks(path, lines, report);
  if (rel.startsWith("book/")) {
    checkEvaluative(lines, report);
    checkColloquial(lines, report);
  }
  if (isChapter(rel)) {
    checkHeadings(lines, report);
    checkReferences(lines, report);
  }
  if (rel === "book/glossary.md") checkGlossary(lines, report);
  return report;
}

function format(rel: string, findings: Finding[]): string {
  return findings.map((f) => `${rel}:${f.line}: ${f.message}`).join("\n");
}

async function main(): Promise<number> {
  const args = Deno.args.length > 0 ? Deno.args : await pathsFromHookInput();
  const paths = args.map((p) => (p.startsWith("/") ? p : `${Deno.cwd()}/${p}`))
    .filter((p) => isTarget(relativePath(p)));

  const errors: string[] = [];
  const warnings: string[] = [];
  for (const path of paths) {
    const rel = relativePath(path);
    const report = await checkFile(path);
    if (report.errors.length > 0) errors.push(format(rel, report.errors));
    if (report.warnings.length > 0) warnings.push(format(rel, report.warnings));
  }

  if (errors.length > 0) {
    console.error(`문서 규칙 위반 (고친 뒤 다시 저장한다):\n${errors.join("\n")}`);
    if (warnings.length > 0) console.error(`주의:\n${warnings.join("\n")}`);
    return 2;
  }
  if (warnings.length > 0) {
    console.log(JSON.stringify({
      hookSpecificOutput: {
        hookEventName: "PostToolUse",
        additionalContext: `문서 규칙 주의 (판단 필요):\n${warnings.join("\n")}`,
      },
    }));
  }
  return 0;
}

Deno.exit(await main());
