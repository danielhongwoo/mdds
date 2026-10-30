#!/usr/bin/env -S deno run --allow-run=docker --allow-read --allow-write
/**
 * shape_main 시나리오 하나를 두 벤더 컨테이너에서 실행하고 캡처와 로그를 captures/cyclone-fastdds/ 에 남긴다.
 *
 *   ./capture-shape-scenario.ts <이름>           예: ./capture-shape-scenario.ts rel-cyclone-pub
 *   ./capture-shape-scenario.ts --list          시나리오 이름 목록
 *   ./capture-shape-scenario.ts --force <이름>  기존 결과를 덮어쓴다
 *
 * 설계 (실행 흐름, 시나리오 표): docs-for-agent/notes/03-first-communication.md
 * 표를 바꾸면 아래 SCENARIOS 도 함께 고친다.
 *
 * 결과 (호스트 경로. 컨테이너 안에서는 /captures/cyclone-fastdds/):
 *   captures/cyclone-fastdds/<이름>.pcap      구독자 컨테이너 eth0 의 udp, igmp
 *   captures/cyclone-fastdds/<이름>.sub.log   구독자 stdout/stderr
 *   captures/cyclone-fastdds/<이름>.pub.log   발행자 stdout/stderr
 *
 * 전제: docker/ 에서 docker compose up -d 로 cyclone, fastdds 컨테이너가 떠 있다.
 * 런타임: Deno 2 (외부 모듈 없음). 권한은 docker 실행과 파일 읽기/쓰기뿐이다.
 */

// ---------------------------------------------------------------------------
// 시나리오 표
// ---------------------------------------------------------------------------

type Vendor = "cyclone" | "fastdds";

interface Scenario {
  /** 발행 컨테이너. 구독은 항상 상대 벤더이고, 캡처도 구독자 쪽에서 한다. */
  pub: Vendor;
  /** -r: RELIABLE, -b: BEST_EFFORT */
  reliability: "-r" | "-b";
  /** -x: 데이터 표현. 1 = XCDR1, 2 = XCDR2. 기본 1 */
  representation?: 1 | 2;
  /** 발행 횟수 (--num-iterations). null 이면 스스로 끝나지 않는다. 기본 100 */
  iterations?: number | null;
  /** 발행 전용 옵션 추가분 */
  pubExtra?: string[];
  /** 발행자 종료 방식. wait: 스스로 끝날 때까지 기다림, kill9: 10초 뒤 SIGKILL. 기본 wait */
  end?: "wait" | "kill9";
  /** 발행자 종료 뒤 관찰 시간 (초). 기본 5 */
  after?: number;
}

const SCENARIOS: Record<string, Scenario> = {
  "be-cyclone-pub": { pub: "cyclone", reliability: "-b" },
  "be-fastdds-pub": { pub: "fastdds", reliability: "-b" },
  "rel-cyclone-pub": { pub: "cyclone", reliability: "-r" },
  "rel-fastdds-pub": { pub: "fastdds", reliability: "-r" },
  "dispose-on-exit": {
    pub: "cyclone",
    reliability: "-r",
    pubExtra: ["--final-instance-state", "d"],
    after: 10,
  },
  "lease-expiry": {
    pub: "cyclone",
    reliability: "-r",
    iterations: null,
    end: "kill9",
    after: 25, // Cyclone lease 10초 + 여유
  },
  "large-sample": {
    pub: "cyclone",
    reliability: "-r",
    iterations: 30,
    pubExtra: ["--additional-payload-size", "100000"],
  },
  "xcdr2": { pub: "cyclone", reliability: "-r", representation: 2 },
};

/** 시나리오 한 줄을 푼 실행 계획. 기본값이 모두 채워져 있다. */
interface Plan {
  name: string;
  pub: Vendor;
  sub: Vendor;
  /** 실행 파일 이름 (cyclone_shape / fastdds_shape) */
  pubBin: string;
  subBin: string;
  /** shape_main 인자 전체 (-P / -S 포함) */
  pubArgs: string[];
  subArgs: string[];
  end: "wait" | "kill9";
  /** 초 */
  after: number;
}

/** 시나리오 한 줄을 실행에 필요한 값으로 푼다. */
function plan(name: string, s: Scenario): Plan {
  const sub: Vendor = s.pub === "cyclone" ? "fastdds" : "cyclone";
  const iterations = s.iterations === undefined ? 100 : s.iterations;

  // 양쪽 공통. -c 는 넣지 않는다: 구독자에게 주면 내용 필터
  // (ContentFilteredTopic)가 되어 토픽 이름과 SEDP 내용이 바뀐다.
  // deno-fmt-ignore
  const common = [
    "-d", "0",
    "-t", "Square",
    "-x", String(s.representation ?? 1),
    s.reliability,
  ];
  // deno-fmt-ignore
  const pubOnly = [
    "-c", "BLUE",
    "--write-period", "100", // 10 Hz
    ...(iterations === null ? [] : ["--num-iterations", String(iterations)]),
    ...(s.pubExtra ?? []),
  ];

  return {
    name,
    pub: s.pub,
    sub,
    pubBin: `${s.pub}_shape`,
    subBin: `${sub}_shape`,
    pubArgs: ["-P", ...common, ...pubOnly],
    subArgs: ["-S", ...common],
    end: s.end ?? "wait",
    after: s.after ?? 5,
  };
}

// ---------------------------------------------------------------------------
// docker 호출
// ---------------------------------------------------------------------------

const DOCKER_DIR = import.meta.dirname!; // compose.yaml 이 있는 곳
const HOST_OUT = `${DOCKER_DIR}/../captures/cyclone-fastdds`;
const OUT = "/captures/cyclone-fastdds"; // 컨테이너 안 경로 (바인드 마운트)

/** docker 명령 결과. ok 는 종료 코드 0 여부, stdout 은 앞뒤 공백을 지운 표준 출력이다. */
interface CommandResult {
  ok: boolean;
  stdout: string;
}

/** `docker compose <args>` 를 실행하고 결과를 돌려준다. stderr 는 그대로 터미널로 보낸다. */
async function compose(...args: string[]): Promise<CommandResult> {
  const { code, stdout } = await new Deno.Command("docker", {
    args: ["compose", ...args],
    cwd: DOCKER_DIR,
    stdout: "piped",
    stderr: "inherit",
  }).output();
  return { ok: code === 0, stdout: new TextDecoder().decode(stdout).trim() };
}

/** 컨테이너 안에서 명령을 실행하고 끝날 때까지 기다린다. -T: 스크립트라 TTY 가 없다. */
function exec(container: Vendor, ...cmd: string[]): Promise<CommandResult> {
  return compose("exec", "-T", container, ...cmd);
}

/** 컨테이너 안에서 명령을 띄워 두고 바로 돌아온다 (docker compose exec -d). */
async function spawn(container: Vendor, ...cmd: string[]): Promise<void> {
  const { ok } = await compose("exec", "-T", "-d", container, ...cmd);
  if (!ok) throw new Error(`failed to start in ${container}: ${cmd.join(" ")}`);
}

/**
 * shape_main 을 띄우고 stdout/stderr 를 컨테이너 안 로그 파일로 보낸다.
 * 리다이렉트는 컨테이너 안에서 해야 하므로 sh -c 를 거친다.
 * stdbuf -oL: 파일로 보내면 블록 버퍼링되어 SIGKILL 때 로그가 비는 것을 막는다.
 * exec: sh 를 남기지 않아 pgrep -x <벤더>_shape 로 바로 잡힌다.
 */
function spawnShape(
  container: Vendor,
  bin: string,
  args: string[],
  log: string,
): Promise<void> {
  const line = ["exec", "stdbuf", "-oL", bin, ...args].map(shQuote).join(" ");
  return spawn(container, "sh", "-c", `${line} > ${shQuote(log)} 2>&1`);
}

/** 프로세스 이름 전체 일치 (-x). shape_main 이름은 15자 이하라 -x 가 된다. */
async function isRunning(container: Vendor, process: string): Promise<boolean> {
  return (await exec(container, "pgrep", "-x", process)).ok;
}

/** 이름 전체 일치로 신호를 보낸다. 없으면 조용히 넘어간다. */
async function signal(
  container: Vendor,
  sig: "INT" | "TERM" | "KILL",
  process: string,
): Promise<void> {
  await exec(container, "pkill", `-${sig}`, "-x", process);
}

/** sh -c 문자열 안에 넣을 수 있게 작은따옴표로 감싼다. */
function shQuote(s: string): string {
  return /^[\w./=:-]+$/.test(s) ? s : `'${s.replaceAll("'", `'\\''`)}'`;
}

const sleep = (sec: number): Promise<void> => new Promise<void>((r) => setTimeout(r, sec * 1000));

function log(msg: string): void {
  const t = new Date().toTimeString().slice(0, 8);
  console.log(`[${t}] ${msg}`);
}

function fail(msg: string): never {
  console.error(`error: ${msg}`);
  Deno.exit(1);
}

async function exists(path: string): Promise<boolean> {
  try {
    await Deno.stat(path);
    return true;
  } catch (e) {
    if (e instanceof Deno.errors.NotFound) return false;
    throw e;
  }
}

/** 컨테이너마다 자기 벤더 실행 파일만 있다. 남아 있는 shape_main 과 tcpdump 를 찾는다. */
async function leftovers(): Promise<string[]> {
  const found: string[] = [];
  for (const c of ["cyclone", "fastdds"] as const) {
    for (const p of [`${c}_shape`, "tcpdump"]) {
      if (await isRunning(c, p)) found.push(`${c}:${p}`);
    }
  }
  return found;
}

// ---------------------------------------------------------------------------
// 실행
// ---------------------------------------------------------------------------

async function main(args: string[]): Promise<void> {
  const names = Object.keys(SCENARIOS);
  const force = args.includes("--force");
  const argv = args.filter((a) => a !== "--force");
  if (argv[0] === "--list") {
    console.log(names.join("\n"));
    return;
  }
  if (argv.length !== 1 || !(argv[0] in SCENARIOS)) {
    console.error(`usage: capture-shape-scenario.ts [--force] <scenario> | --list`);
    console.error(`scenarios: ${names.join(" ")}`);
    Deno.exit(2);
  }
  const p = plan(argv[0], SCENARIOS[argv[0]]);
  const file = (ext: string): string => `${p.name}.${ext}`;

  // --- 사전 검사 ---
  for (const c of ["cyclone", "fastdds"] as const) {
    const { stdout } = await compose("ps", "-q", "--status", "running", c);
    if (!stdout) fail(`container '${c}' is not running. Run: docker compose up -d`);
  }
  const left = await leftovers();
  if (left.length) {
    fail(
      `processes from a previous run are still alive: ${left.join(" ")}\n` +
        `  stop them first, e.g.: docker compose exec cyclone pkill -INT -x cyclone_shape`,
    );
  }
  // 캡처는 커밋되는 정답 데이터라 실수로 덮어쓰지 않는다.
  if (!force) {
    for (const ext of ["pcap", "sub.log", "pub.log"]) {
      if (await exists(`${HOST_OUT}/${file(ext)}`)) {
        fail(`captures/cyclone-fastdds/${file(ext)} exists. Use --force to overwrite.`);
      }
    }
  }
  // 호스트에서 만든다 (컨테이너가 만들면 Linux 호스트에서는 root 소유가 된다)
  await Deno.mkdir(HOST_OUT, { recursive: true });

  log(`${p.name}: pub=${p.pub} sub=${p.sub} end=${p.end} after=${p.after}s`);
  log(`  sub: ${p.subBin} ${p.subArgs.join(" ")}`);
  log(`  pub: ${p.pubBin} ${p.pubArgs.join(" ")}`);

  // 번호는 03-first-communication.md "시나리오 실행 흐름" 과 같다.

  // 1-2. 캡처를 먼저 켠다. 첫 IGMP 가입과 첫 SPDP 부터 잡힌다.
  //      -U: 패킷마다 바로 쓰기, -Z root: 권한을 낮추다 바인드 마운트에 못 쓰는 문제 방지
  await spawn(
    p.sub,
    "tcpdump",
    "-i",
    "eth0",
    "-U",
    "-Z",
    "root",
    "-w",
    `${OUT}/${file("pcap")}`,
    "udp or igmp",
  );
  await sleep(1);

  // 3-4. 구독자를 먼저 띄워 SPDP 를 먼저 내보내게 한다.
  await spawnShape(p.sub, p.subBin, p.subArgs, `${OUT}/${file("sub.log")}`);
  await sleep(3);

  // 5. 발행자
  log("publisher start");
  await spawnShape(p.pub, p.pubBin, p.pubArgs, `${OUT}/${file("pub.log")}`);
  await sleep(1);

  // 6. 발행자 종료
  if (p.end === "wait") {
    while (await isRunning(p.pub, p.pubBin)) await sleep(1);
    log("publisher exited by itself");
  } else {
    await sleep(10);
    await signal(p.pub, "KILL", p.pubBin);
    log("publisher killed with SIGKILL");
  }

  // 7. dispose 전파나 lease 만료를 관찰할 시간
  await sleep(p.after);

  // 8. 구독자 정상 종료. shape_main 은 SIGINT 핸들러만 있어 SIGTERM 이면 정리 없이 죽는다.
  await signal(p.sub, "INT", p.subBin);
  await sleep(2);

  // 9. tcpdump 는 SIGTERM 을 받으면 파일을 닫고 끝난다.
  await signal(p.sub, "TERM", "tcpdump");
  await sleep(1);

  // 10. 아무것도 남지 않아야 한다.
  const still = await leftovers();
  if (still.length) console.error(`warning: still running after the scenario: ${still.join(" ")}`);

  // 결과 요약
  for (const ext of ["pcap", "pub.log", "sub.log"]) {
    const path = `${HOST_OUT}/${file(ext)}`;
    const size = (await exists(path)) ? `${(await Deno.stat(path)).size} bytes` : "missing";
    console.log(`  captures/cyclone-fastdds/${file(ext).padEnd(28)} ${size}`);
  }
  log("done");
}

await main(Deno.args);
