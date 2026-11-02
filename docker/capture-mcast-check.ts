#!/usr/bin/env -S deno run --allow-run=docker --allow-read --allow-write
/**
 * mcast-check 로 두 벤더 컨테이너 사이의 멀티캐스트를 주고받으며 캡처와 로그를 captures/mcast-check/ 에 남긴다.
 * DDS 는 실행하지 않는다. 네트워크가 멀티캐스트를 전달하는지와 그룹 가입 (IGMP) 을 보는 참고 캡처다.
 *
 *   ./capture-mcast-check.ts <이름>           예: ./capture-mcast-check.ts join
 *   ./capture-mcast-check.ts --list          시나리오 이름 목록
 *   ./capture-mcast-check.ts --force <이름>  기존 결과를 덮어쓴다
 *
 * 시나리오:
 *   join    fastdds 가 239.255.0.1:7400 에 가입해 받고, cyclone 이 3번 보낸다. 양쪽 eth0 을 캡처한다.
 *   nojoin  아무도 가입하지 않은 채 cyclone 이 3번 보낸다. fastdds eth0 을 캡처한다.
 *
 * 결과 (호스트 경로. 컨테이너 안에서는 /captures/mcast-check/):
 *   captures/mcast-check/<이름>.<컨테이너>.pcap   그 컨테이너 eth0 의 igmp, udp port 7400
 *   captures/mcast-check/<이름>.recv.log          mcast-check recv 출력 (join 만)
 *   captures/mcast-check/<이름>.send.log          mcast-check send 출력
 *
 * 전제: docker/ 에서 docker compose up -d 와 docker compose build tools 를 마쳤다.
 * mcast-check 는 벤더 이미지에 없다. 도구 이미지 mdds/tools 를 벤더 컨테이너의 네트워크 네임스페이스에
 * 붙여 실행한다 (mcast-check.sh 와 같은 방식, docs/environment/docker-network.md "검증 명령").
 * 런타임: Deno 2 (외부 모듈 없음). 권한은 docker 실행과 파일 읽기/쓰기뿐이다.
 */

// ---------------------------------------------------------------------------
// 시나리오 표
// ---------------------------------------------------------------------------

type Vendor = "cyclone" | "fastdds";

interface Scenario {
  /** 받는 쪽. null 이면 아무도 그룹에 가입하지 않는다. */
  recv: Vendor | null;
  /** 보내는 쪽 */
  send: Vendor;
  /** eth0 을 캡처할 컨테이너 */
  capture: Vendor[];
}

const SCENARIOS: Record<string, Scenario> = {
  "join": { recv: "fastdds", send: "cyclone", capture: ["fastdds", "cyclone"] },
  "nojoin": { recv: null, send: "cyclone", capture: ["fastdds"] },
};

// ---------------------------------------------------------------------------
// docker 호출
// ---------------------------------------------------------------------------

const DOCKER_DIR = import.meta.dirname!; // compose.yaml 이 있는 곳
const HOST_OUT = `${DOCKER_DIR}/../captures/mcast-check`;
const OUT = "/captures/mcast-check"; // 컨테이너 안 경로 (바인드 마운트)
const TOOLS_IMAGE = "mdds/tools";
const RECV_NAME = "mdds-mcast-check-recv"; // 받는 쪽 도구 컨테이너 이름 (신호를 보내려고 고정한다)

/** docker 명령 결과. ok 는 종료 코드 0 여부, stdout 은 앞뒤 공백을 지운 표준 출력이다. */
interface CommandResult {
  ok: boolean;
  stdout: string;
}

/** `docker <args>` 를 실행하고 결과를 돌려준다. stderr 는 그대로 터미널로 보낸다. */
async function docker(...args: string[]): Promise<CommandResult> {
  const { code, stdout } = await new Deno.Command("docker", {
    args,
    cwd: DOCKER_DIR,
    stdout: "piped",
    stderr: "inherit",
  }).output();
  return { ok: code === 0, stdout: new TextDecoder().decode(stdout).trim() };
}

/** 실행 중인 벤더 컨테이너의 id. 없으면 빈 문자열이다. */
async function containerId(c: Vendor): Promise<string> {
  return (await docker("compose", "ps", "-q", "--status", "running", c)).stdout;
}

/**
 * 도구 컨테이너를 벤더 컨테이너의 네트워크 네임스페이스에 붙여 실행한다 (같은 eth0, 같은 IP).
 * --init: mcast-check 가 PID 1 이 되면 SIGINT 기본 동작이 무시되어 신호로 끝나지 않는다.
 */
function tool(vendorId: string, extra: string[], ...cmd: string[]): Promise<CommandResult> {
  return docker(
    "run",
    "--init",
    "--network",
    `container:${vendorId}`,
    ...extra,
    TOOLS_IMAGE,
    ...cmd,
  );
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
    console.error(`usage: capture-mcast-check.ts [--force] <scenario> | --list`);
    console.error(`scenarios: ${names.join(" ")}`);
    Deno.exit(2);
  }
  const name = argv[0];
  const s = SCENARIOS[name];
  const files = [
    ...s.capture.map((c) => `${name}.${c}.pcap`),
    ...(s.recv ? [`${name}.recv.log`] : []),
    `${name}.send.log`,
  ];

  // --- 사전 검사 ---
  const ids = {} as Record<Vendor, string>;
  for (const c of ["cyclone", "fastdds"] as const) {
    ids[c] = await containerId(c);
    if (!ids[c]) fail(`container '${c}' is not running. Run: docker compose up -d`);
  }
  if (!(await docker("image", "inspect", "-f", "{{.Id}}", TOOLS_IMAGE)).ok) {
    fail(`image ${TOOLS_IMAGE} not found. Run: docker compose build tools`);
  }
  for (const c of ["cyclone", "fastdds"] as const) {
    if ((await docker("compose", "exec", "-T", c, "pgrep", "-x", "tcpdump")).ok) {
      fail(`tcpdump is still running in ${c}. Run: docker compose exec ${c} pkill -INT tcpdump`);
    }
  }
  // 캡처는 커밋되는 참고 자료라 실수로 덮어쓰지 않는다.
  if (!force) {
    for (const f of files) {
      if (await exists(`${HOST_OUT}/${f}`)) {
        fail(`captures/mcast-check/${f} exists. Use --force to overwrite.`);
      }
    }
  }
  // 호스트에서 만든다 (컨테이너가 만들면 Linux 호스트에서는 root 소유가 된다)
  await Deno.mkdir(HOST_OUT, { recursive: true });
  // 앞 실행에서 남은 받는 쪽 도구 컨테이너가 있으면 지운다 (없으면 오류를 무시한다)
  await new Deno.Command("docker", {
    args: ["rm", "-f", RECV_NAME],
    stdout: "null",
    stderr: "null",
  })
    .output();

  log(`${name}: recv=${s.recv ?? "none"} send=${s.send} capture=${s.capture.join(",")}`);

  // 1. 캡처를 먼저 켠다. 받는 쪽의 첫 IGMP 가입부터 잡힌다.
  //    -U: 패킷마다 바로 쓰기, -Z root: 권한을 낮추다 바인드 마운트에 못 쓰는 문제 방지
  for (const c of s.capture) {
    const { ok } = await docker(
      "compose",
      "exec",
      "-T",
      "-d",
      c,
      "tcpdump",
      "-i",
      "eth0",
      "-U",
      "-Z",
      "root",
      "-w",
      `${OUT}/${name}.${c}.pcap`,
      "igmp or udp port 7400",
    );
    if (!ok) fail(`failed to start tcpdump in ${c}`);
  }
  await sleep(1);

  // 2. 받는 쪽: bind -> 그룹 가입 -> 대기. 로그를 나중에 읽으려고 --rm 없이 띄운다.
  if (s.recv) {
    const { ok } = await tool(ids[s.recv], ["-d", "--name", RECV_NAME], "mcast-check", "recv");
    if (!ok) fail(`failed to start mcast-check recv in ${s.recv}`);
    await sleep(2);
  }

  // 3. 보내는 쪽: 1초 간격으로 3번. 끝날 때까지 기다린다.
  const sent = await tool(ids[s.send], ["--rm"], "mcast-check", "send");
  await Deno.writeTextFile(`${HOST_OUT}/${name}.send.log`, sent.stdout + "\n");
  if (!sent.ok) fail(`mcast-check send failed in ${s.send}`);
  await sleep(1);

  // 4. 받는 쪽 종료. SIGINT 로 끝내야 소켓이 닫히며 커널이 그룹을 떠난다 (IGMP leave 가 잡힌다).
  if (s.recv) {
    await docker("kill", "-s", "INT", RECV_NAME);
    await docker("wait", RECV_NAME);
    const { stdout } = await docker("logs", RECV_NAME);
    await Deno.writeTextFile(`${HOST_OUT}/${name}.recv.log`, stdout + "\n");
    await docker("rm", RECV_NAME);
  }

  // 5. leave 재전송 (1초 안의 무작위 시점) 까지 잡히게 기다린 뒤 tcpdump 를 끝낸다.
  await sleep(3);
  for (const c of s.capture) {
    await docker("compose", "exec", "-T", c, "pkill", "-INT", "-x", "tcpdump");
  }
  await sleep(1);

  // 결과 요약
  for (const f of files) {
    const path = `${HOST_OUT}/${f}`;
    const size = (await exists(path)) ? `${(await Deno.stat(path)).size} bytes` : "missing";
    console.log(`  captures/mcast-check/${f.padEnd(24)} ${size}`);
  }
  log("done");
}

await main(Deno.args);
