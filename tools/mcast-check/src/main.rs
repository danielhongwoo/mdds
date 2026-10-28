//! 컨테이너 사이에 UDP 멀티캐스트가 전달되는지 확인한다.
//!
//! DDS를 띄우기 전에 네트워크만 따로 시험하는 용도다. 존재 이유와 범위는 README.md에 있다.
//!
//!   mcast-check recv [--group A] [--port P] [--iface A]
//!   mcast-check send [--group A] [--port P] [--count N] [--interval-ms MS]
//!
//! 두 명령이 하는 소켓 호출과, 그때 커널과 bridge에서 일어나는 일 (괄호 안은 같은 일을 하는 C 호출):
//!
//!   recv (받는 쪽, 예: fastdds 컨테이너)              send (보내는 쪽, 예: cyclone 컨테이너)
//!   ----------------------------------------          ----------------------------------------
//!   UdpSocket::bind(0.0.0.0:7400)                     UdpSocket::bind(0.0.0.0:0)   임의 포트
//!     (socket(AF_INET, SOCK_DGRAM, 0) + bind)           (socket + bind, sin_port = 0)
//!   join_multicast_v4(239.255.0.1, eth0 주소)         set_multicast_ttl_v4(1)
//!     (setsockopt IP_ADD_MEMBERSHIP)                    (setsockopt IP_MULTICAST_TTL)
//!     -> 커널: eth0 그룹 목록에 추가 (ip maddr)
//!     -> eth0 으로 IGMP Membership Report
//!     -> bridge: 이 포트가 그룹을 원한다고 학습
//!   recv_from() 에서 대기 (recvfrom)                  send_to("hello ...", 239.255.0.1:7400) (sendto)
//!                                                       -> eth0 으로 나가 bridge 에 도착
//!                                                       -> bridge 가 가입한 포트로 전달
//!   <- 패킷 수신, 출발 주소와 내용 출력
//!   (끝나면 소켓이 drop 되며 close)                    (끝나면 close)
//!
//! 받는 쪽이 하는 일 (bind -> 그룹 가입 -> 수신)은 DDS 참가자가 SPDP를 받을 때와 같다.
//! 보내는 쪽은 그룹에 가입하지 않는다. 멀티캐스트는 그룹 주소로 보내기만 하면 된다.
//!
//! C에 익숙한 사람을 위한 Rust 읽는 법:
//! - `UdpSocket`은 소켓 fd 하나를 감싼 값이다. 변수가 범위를 벗어나면 (drop) 자동으로 close(fd)된다.
//! - `io::Result<T>`는 "성공 값 T 또는 오류 (errno를 담은 io::Error)"다. C의 "-1을 돌려주고 errno 설정"에 해당한다.
//! - 호출 뒤의 `?`는 C의 `if (ret < 0) return -1;`과 같다. 오류면 그 자리에서 호출한 쪽으로 오류를 넘긴다.
//! - 주소는 `struct sockaddr_in` 대신 `(Ipv4Addr, u16)` 이나 `SocketAddrV4`로 넘긴다. htons / inet_pton 같은
//!   바이트 순서 변환은 표준 라이브러리가 대신 한다.
//!
//! 표준 라이브러리만 쓴다. 그래서 SO_REUSEADDR를 걸지 못한다. C라면 socket()과 bind() 사이에
//! setsockopt(SO_REUSEADDR)를 넣지만, std의 `UdpSocket::bind`는 socket()과 bind()를 한 번에 해서
//! 그 사이에 옵션을 넣을 틈이 없다. 같은 포트를 이미 쓰는 DDS 프로세스가 있으면 recv의 bind가
//! 실패하므로 DDS를 띄우기 전에 쓴다.

use std::env;
use std::io;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::process::ExitCode;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

// RTPS 2.5 Sec. 9.6.2.4: SPDP 기본 멀티캐스트 주소. 모든 참가자가 이 주소로 알리고 이 주소에서 듣는다.
// 239.0.0.0/8은 관리 범위 (administratively scoped, RFC 2365) 멀티캐스트 대역이고,
// 그 안의 239.255.0.0/16은 가장 작은 범위인 Local Scope다.
const DEFAULT_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 1);
// RTPS 2.5 Sec. 9.6.2.1 Table 9.15, Sec. 9.6.2.3: SPDP 멀티캐스트 포트 = PB + DG * domainId + d0
// = 7400 + 250 * 0 + 0. 도메인 0 기준이다. 도메인 1이면 7650이 된다.
const DEFAULT_PORT: u16 = 7400;

const USAGE: &str = "\
usage:
  mcast-check recv [--group A] [--port P] [--iface A]
  mcast-check send [--group A] [--port P] [--count N] [--interval-ms MS]

defaults: --group 239.255.0.1 --port 7400 --count 3 --interval-ms 1000
          --iface: the address of the interface that routes to the group (eth0 in a container)";

/// 명령줄 옵션. recv와 send가 같은 구조를 쓰고, 각자 필요한 필드만 읽는다.
struct Options {
    /// 멀티캐스트 그룹 주소 (224.0.0.0/4 안이어야 한다)
    group: Ipv4Addr,
    /// recv는 이 포트에서 듣고, send는 이 포트로 보낸다
    port: u16,
    /// recv 전용: 어느 인터페이스에서 그룹에 가입할지 (그 인터페이스의 IPv4 주소로 지정).
    /// 없으면 local_addr_toward로 정한다.
    iface: Option<Ipv4Addr>,
    /// send 전용: 보낼 횟수
    count: u32,
    /// send 전용: 보내는 간격
    interval: Duration,
}

/// 종료 코드: 0 성공, 1 실행 중 오류 (소켓 오류 등), 2 사용법 오류.
/// (`ExitCode::from`은 const 문맥에서 못 쓰므로 상수 대신 함수다.)
fn usage_error() -> ExitCode {
    ExitCode::from(2)
}

fn main() -> ExitCode {
    // 첫 인자가 명령 (recv / send), 나머지는 "--이름 값" 쌍이다.
    let args: Vec<String> = env::args().skip(1).collect();
    // 도움말은 어느 위치에 있어도 먼저 처리한다. 옵션 해석 뒤에 두면 "--help needs a value"가 된다.
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let Some((command, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return usage_error();
    };
    let opts = match parse_options(rest) {
        Ok(opts) => opts,
        Err(msg) => {
            eprintln!("error: {msg}\n\n{USAGE}");
            return usage_error();
        }
    };
    let result = match command.as_str() {
        "recv" => recv(&opts),
        "send" => send(&opts),
        other => {
            eprintln!("error: unknown command '{other}'\n\n{USAGE}");
            return usage_error();
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            // 가장 흔한 실패: 같은 컨테이너의 DDS가 이미 7400을 bind한 경우 (모듈 주석 참고)
            if e.kind() == io::ErrorKind::AddrInUse {
                eprintln!("hint: another process (DDS?) already uses this port. Stop it first.");
            }
            ExitCode::FAILURE
        }
    }
}

/// "--이름 값" 쌍을 차례로 읽는다. 모르는 옵션이나 값이 빠진 옵션은 오류다.
/// 옵션 수가 적어서 인자 해석 라이브러리를 쓰지 않는다 (외부 의존성 없음).
fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut opts = Options {
        group: DEFAULT_GROUP,
        port: DEFAULT_PORT,
        iface: None,
        count: 3,
        interval: Duration::from_millis(1000),
    };
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--group" => opts.group = parse_value(flag, value)?,
            "--port" => opts.port = parse_value(flag, value)?,
            "--iface" => opts.iface = Some(parse_value(flag, value)?),
            "--count" => opts.count = parse_value(flag, value)?,
            "--interval-ms" => opts.interval = Duration::from_millis(parse_value(flag, value)?),
            _ => return Err(format!("unknown option '{flag}'")),
        }
    }
    // 유니캐스트 주소를 주면 join_multicast_v4가 알아보기 어려운 오류를 내므로 미리 막는다.
    if !opts.group.is_multicast() {
        return Err(format!("{} is not a multicast address", opts.group));
    }
    Ok(opts)
}

/// 문자열을 대상 타입 (주소, 포트, 숫자)으로 바꾼다. 실패하면 어느 옵션이 틀렸는지 알려 준다.
fn parse_value<T: FromStr>(flag: &str, value: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

/// 그룹으로 보낼 때 커널이 고르는 출발 주소, 곧 그 경로가 나가는 인터페이스의 주소를 구한다.
///
/// 표준 라이브러리에는 "인터페이스 목록과 주소"를 구하는 API가 없다. 대신 UDP 소켓의 connect를
/// 이용한다. UDP의 connect는 패킷을 보내지 않고, 목적지를 기억하면서 커널의 경로 선택만 한다.
/// 그 결과로 정해진 출발 주소 (local_addr)가 곧 그 경로가 나가는 인터페이스의 주소다.
///
/// 컨테이너에는 멀티캐스트 전용 경로가 없으므로 기본 경로 (eth0, gateway 172.28.0.1)가 쓰이고,
/// eth0의 주소 (cyclone이면 172.28.0.10)가 나온다. 인터페이스가 여러 개인 호스트에서는
/// 원하는 인터페이스가 아닐 수 있으므로 recv의 --iface로 직접 지정한다.
///
/// C로 쓰면:
///   fd = socket(AF_INET, SOCK_DGRAM, 0);  bind(fd, 0.0.0.0:0);
///   connect(fd, group:port);              getsockname(fd, &local);   -> local.sin_addr
///   close(fd);
fn local_addr_toward(group: Ipv4Addr, port: u16) -> io::Result<Ipv4Addr> {
    let probe = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    probe.connect((group, port))?;
    // 함수가 끝나면 probe가 drop되어 close된다.
    match probe.local_addr()?.ip() {
        IpAddr::V4(addr) => Ok(addr),
        // IPv4 소켓을 IPv4 목적지로 connect했으므로 여기로 오지 않는다.
        IpAddr::V6(addr) => Err(io::Error::other(format!("unexpected IPv6 address {addr}"))),
    }
}

/// 그룹에 가입하고, 받은 패킷을 끝없이 출력한다. Ctrl+C (SIGINT)로 끝낸다.
///
/// 프로세스가 끝나면 커널이 소켓을 닫으면서 그룹 가입도 풀고 IGMP Leave를 보낸다.
/// 그래서 끝낸 뒤에는 `ip maddr`에서 그룹이 사라진다.
fn recv(opts: &Options) -> io::Result<()> {
    let iface = match opts.iface {
        Some(addr) => addr,
        None => local_addr_toward(opts.group, opts.port)?,
    };

    // 0.0.0.0 (모든 주소)에 bind한다. 받는 패킷의 목적지는 eth0 주소가 아니라 그룹 주소이므로,
    // eth0 주소 (172.28.0.11)에 bind하면 멀티캐스트 패킷이 이 소켓에 들어오지 않는다.
    // 부수 효과로 이 포트로 오는 유니캐스트도 받는다. 시험에는 상관없다.
    // C:
    //   fd = socket(AF_INET, SOCK_DGRAM, 0);
    //   struct sockaddr_in a = { .sin_family = AF_INET, .sin_port = htons(7400),
    //                            .sin_addr.s_addr = htonl(INADDR_ANY) };
    //   bind(fd, (struct sockaddr *)&a, sizeof a);
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, opts.port))?;

    // IP_ADD_MEMBERSHIP 소켓 옵션. 인자는 (그룹 주소, 가입할 인터페이스의 주소)다.
    // C:
    //   struct ip_mreq m;
    //   inet_pton(AF_INET, "239.255.0.1", &m.imr_multiaddr);   // 그룹
    //   inet_pton(AF_INET, "172.28.0.11", &m.imr_interface);   // 가입할 인터페이스 (eth0의 주소)
    //   setsockopt(fd, IPPROTO_IP, IP_ADD_MEMBERSHIP, &m, sizeof m);
    // 두 번째 인자 (imr_interface)는 소켓의 bind 주소가 아니라 "어느 인터페이스에서 듣는가"를 정한다.
    // 이 호출로 커널이
    //   1. 그 인터페이스의 그룹 목록에 그룹을 올리고 (`ip maddr show eth0`에 보인다),
    //   2. IGMP Membership Report를 내보낸다 (bridge의 멀티캐스트 스누핑이 이것을 보고 전달할 포트를 정한다).
    // 가입은 bind와 별개다. bind는 포트를 정하고, 가입은 어떤 그룹 주소의 패킷을 받을지 정한다.
    socket.join_multicast_v4(&opts.group, &iface)?;
    println!(
        "joined {} on {iface}, listening on port {} (Ctrl+C to stop)",
        opts.group, opts.port
    );

    // UDP 데이터그램 하나는 IPv4에서 최대 65507바이트이므로 64 KiB 버퍼면 잘리지 않는다.
    let mut buf = [0u8; 65536];
    loop {
        // 패킷이 올 때까지 막힌다 (blocking). from은 보낸 쪽의 주소와 포트다.
        // send 쪽은 임의 포트로 bind하므로 포트 번호는 실행할 때마다 다르다.
        // C:
        //   struct sockaddr_in from; socklen_t fl = sizeof from;
        //   ssize_t len = recvfrom(fd, buf, sizeof buf, 0, (struct sockaddr *)&from, &fl);
        // Rust는 (받은 길이, 보낸 쪽 주소)를 튜플로 돌려준다.
        let (len, from) = socket.recv_from(&mut buf)?;
        println!("{len} bytes from {from}: {}", printable(&buf[..len]));
    }
}

/// 그룹 주소로 짧은 문자열을 count번 보낸다.
fn send(opts: &Options) -> io::Result<()> {
    // 메시지에 넣을 자기 주소. 받는 쪽 출력에서 누가 보냈는지 바로 보이게 한다.
    // 실제 출발 주소는 커널이 정하며, 같은 경로 선택을 하므로 이 값과 같다.
    let source = local_addr_toward(opts.group, opts.port)?;

    // 포트 0으로 bind하면 커널이 비어 있는 임의 포트를 준다. 보내는 쪽은 특정 포트가 필요 없다.
    // C: fd = socket(AF_INET, SOCK_DGRAM, 0); bind(fd, INADDR_ANY, sin_port = 0);
    //    (C에서는 bind를 생략해도 첫 sendto 때 커널이 임의 포트를 붙인다)
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;

    // IP_MULTICAST_TTL. 기본값도 1이지만, 라우터를 넘지 않고 같은 L2 세그먼트 (bridge) 안에서만
    // 돌게 한다는 의도를 남긴다. 같은 bridge 안의 전달에는 TTL 1이면 충분하다.
    // C: unsigned char ttl = 1; setsockopt(fd, IPPROTO_IP, IP_MULTICAST_TTL, &ttl, sizeof ttl);
    socket.set_multicast_ttl_v4(1)?;

    // 나가는 인터페이스는 지정하지 않는다. C라면 setsockopt(IP_MULTICAST_IF)로 정할 수 있지만
    // std에는 이 옵션이 없다. 커널이 경로 표로 고르며, 컨테이너에서는 eth0뿐이라 문제없다.
    for i in 1..=opts.count {
        let message = format!("hello {i}/{} from {source}", opts.count);
        // 목적지를 그룹 주소로 주는 것 말고는 유니캐스트 송신과 같다. 그룹 가입은 필요 없다.
        // C: sendto(fd, msg, strlen(msg), 0, (struct sockaddr *)&group_addr, sizeof group_addr);
        socket.send_to(message.as_bytes(), (opts.group, opts.port))?;
        println!("sent to {}:{}: {message}", opts.group, opts.port);
        if i < opts.count {
            thread::sleep(opts.interval);
        }
    }
    Ok(())
}

/// 받은 바이트의 앞부분을 화면에 찍을 수 있게 만든다.
///
/// 다른 컨테이너에서 DDS가 돌고 있으면 그쪽 SPDP 패킷 (RTPS 이진 데이터)이 들어올 수 있다. 터미널이 깨지지 않도록
/// 출력 가능한 ASCII가 아닌 바이트는 '.'으로 바꾸고, 64바이트를 넘으면 자른다.
/// RTPS 패킷이면 맨 앞에 "RTPS"가 보인다.
fn printable(bytes: &[u8]) -> String {
    const MAX: usize = 64;
    let mut out: String = bytes
        .iter()
        .take(MAX)
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            }
        })
        .collect();
    if bytes.len() > MAX {
        out.push_str("...");
    }
    out
}
