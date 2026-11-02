//! 매칭 여부를 정하는 QoS 정책과 호환성 검사.
//!
//! 여기 있는 타입은 DDS의 DataWriterQos / DataReaderQos가 아니라, 디스커버리 (SEDP)로
//! 상대에게 알려지는 QoS 부분이다 (RTPS 2.5 표 9.19의 PublicationBuiltinTopicData /
//! SubscriptionBuiltinTopicData). 매칭 조건 가운데 QoS에 관한 것은 이 부분만으로 정해진다
//! (토픽 이름과 타입은 QoS가 아니고 디스커버리 데이터의 다른 필드다).
//!
//! 규칙의 출처는 DDS 1.4 Sec. 2.2.3 (정책별 "offered ... compatible with requested if and
//! only if ...")과 XTypes 1.3 Sec. 7.6.3.1.1 (DATA_REPRESENTATION)이다.

/// 시간 길이. DDS 1.4의 Duration_t 의미이고, 무한대가 가장 크다 (선언 순서로 `Ord`를 파생).
///
/// RTPS의 와이어 표현 `Duration_t` (seconds + fraction, RTPS 2.5 Sec. 9.3.2.3)와는 다른 타입이다.
/// 와이어 변환은 디스커버리 쪽에서 한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Duration {
    Finite(core::time::Duration),
    Infinite,
}

impl Duration {
    pub const ZERO: Duration = Duration::Finite(core::time::Duration::ZERO);

    pub const fn from_secs(secs: u64) -> Duration {
        Duration::Finite(core::time::Duration::from_secs(secs))
    }

    pub const fn from_millis(millis: u64) -> Duration {
        Duration::Finite(core::time::Duration::from_millis(millis))
    }
}

// 아래 kind enum들은 선언 순서가 규격이 정한 순서다 ("considered ordered such that ...").
// `Ord`를 파생하므로 `offered >= requested` 같은 비교를 그대로 쓸 수 있다.
// 판별값은 와이어 값이 아니다. RTPS는 BEST_EFFORT = 1, RELIABLE = 2로 보낸다
// (RTPS 2.5 Sec. 9.3.2). 와이어 변환은 디스커버리 쪽에서 한다.

/// DDS 1.4 Sec. 2.2.3.14. BEST_EFFORT < RELIABLE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

/// DDS 1.4 Sec. 2.2.3.4. VOLATILE < TRANSIENT_LOCAL < TRANSIENT < PERSISTENT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DurabilityKind {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

/// DDS 1.4 Sec. 2.2.3.11. AUTOMATIC < MANUAL_BY_PARTICIPANT < MANUAL_BY_TOPIC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LivelinessKind {
    Automatic,
    ManualByParticipant,
    ManualByTopic,
}

/// DDS 1.4 Sec. 2.2.3.17. BY_RECEPTION_TIMESTAMP < BY_SOURCE_TIMESTAMP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DestinationOrderKind {
    ByReceptionTimestamp,
    BySourceTimestamp,
}

/// DDS 1.4 Sec. 2.2.3.6. INSTANCE < TOPIC < GROUP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PresentationAccessScope {
    Instance,
    Topic,
    Group,
}

/// DDS 1.4 Sec. 2.2.3.9. 순서가 없다. 같아야 호환이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipKind {
    Shared,
    Exclusive,
}

/// 데이터 표현 식별자. XTypes 1.3 Sec. 7.6.3.1.1이 정한 2바이트 정수 값이다 (XCDR = 0, XML = 1,
/// XCDR2 = 2. 음수는 벤더용). 순서가 없다. writer가 제공하는 하나가 reader의 목록에 있어야 호환이다.
/// 와이어에서 모르는 값이 와도 담을 수 있게 enum이 아니라 값 그대로 둔다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataRepresentationId(pub i16);

impl DataRepresentationId {
    pub const XCDR1: DataRepresentationId = DataRepresentationId(0);
    pub const XML: DataRepresentationId = DataRepresentationId(1);
    pub const XCDR2: DataRepresentationId = DataRepresentationId(2);
}

/// DDS 1.4 Sec. 2.2.3.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Presentation {
    pub access_scope: PresentationAccessScope,
    pub coherent_access: bool,
    pub ordered_access: bool,
}

impl Default for Presentation {
    fn default() -> Self {
        Presentation {
            access_scope: PresentationAccessScope::Instance,
            coherent_access: false,
            ordered_access: false,
        }
    }
}

/// DDS 1.4 Sec. 2.2.3.11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Liveliness {
    pub kind: LivelinessKind,
    pub lease_duration: Duration,
}

impl Default for Liveliness {
    fn default() -> Self {
        Liveliness {
            kind: LivelinessKind::Automatic,
            lease_duration: Duration::Infinite,
        }
    }
}

/// writer가 제공하는 (offered) QoS. SEDP의 writer 정보에 실리는 부분이다.
///
/// `Default`는 writer 정보에 해당 PID가 없을 때의 값, 곧 DDS 1.4가 DataWriter에 정한 기본값이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationQos {
    pub durability: DurabilityKind,
    pub presentation: Presentation,
    pub deadline: Duration,
    pub latency_budget: Duration,
    pub ownership: OwnershipKind,
    pub liveliness: Liveliness,
    pub reliability: ReliabilityKind,
    pub destination_order: DestinationOrderKind,
    /// 빈 목록은 `""` 하나와 같다 (DDS 1.4 Sec. 2.2.3.13).
    pub partition: Vec<String>,
    /// 첫 원소가 writer가 제공하는 표현이다. 빈 목록은 `[XCDR1]`과 같다 (XTypes 1.3 Sec. 7.6.3.1.1).
    pub data_representation: Vec<DataRepresentationId>,
}

impl Default for PublicationQos {
    fn default() -> Self {
        PublicationQos {
            durability: DurabilityKind::Volatile,
            presentation: Presentation::default(),
            deadline: Duration::Infinite,
            latency_budget: Duration::ZERO,
            ownership: OwnershipKind::Shared,
            liveliness: Liveliness::default(),
            // DDS 1.4 Sec. 2.2.3.14: RELIABLE은 DataWriter의 기본값
            reliability: ReliabilityKind::Reliable,
            destination_order: DestinationOrderKind::ByReceptionTimestamp,
            partition: Vec::new(),
            data_representation: Vec::new(),
        }
    }
}

/// reader가 요구하는 (requested) QoS. SEDP의 reader 정보에 실리는 부분이다.
///
/// `Default`는 reader 정보에 해당 PID가 없을 때의 값, 곧 DDS 1.4가 DataReader에 정한 기본값이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionQos {
    pub durability: DurabilityKind,
    pub presentation: Presentation,
    pub deadline: Duration,
    pub latency_budget: Duration,
    pub ownership: OwnershipKind,
    pub liveliness: Liveliness,
    pub reliability: ReliabilityKind,
    pub destination_order: DestinationOrderKind,
    /// 빈 목록은 `""` 하나와 같다 (DDS 1.4 Sec. 2.2.3.13).
    pub partition: Vec<String>,
    /// reader가 받을 수 있는 표현의 목록. 빈 목록은 `[XCDR1]`과 같다 (XTypes 1.3 Sec. 7.6.3.1.1).
    pub data_representation: Vec<DataRepresentationId>,
}

impl Default for SubscriptionQos {
    fn default() -> Self {
        SubscriptionQos {
            durability: DurabilityKind::Volatile,
            presentation: Presentation::default(),
            deadline: Duration::Infinite,
            latency_budget: Duration::ZERO,
            ownership: OwnershipKind::Shared,
            liveliness: Liveliness::default(),
            // DDS 1.4 Sec. 2.2.3.14: BEST_EFFORT는 DataReader와 Topic의 기본값
            reliability: ReliabilityKind::BestEffort,
            destination_order: DestinationOrderKind::ByReceptionTimestamp,
            partition: Vec::new(),
            data_representation: Vec::new(),
        }
    }
}

/// QoS 정책의 번호. DDS 1.4 PSM (Sec. 2.3.3)의 `*_QOS_POLICY_ID`와 XTypes 1.3 Annex D의 23, 24.
///
/// 벤더의 incompatible_qos 로그에 찍히는 숫자가 이 값이다 (예: `11 (RELIABILITY)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum QosPolicyId {
    UserData = 1,
    Durability = 2,
    Presentation = 3,
    Deadline = 4,
    LatencyBudget = 5,
    Ownership = 6,
    OwnershipStrength = 7,
    Liveliness = 8,
    TimeBasedFilter = 9,
    Partition = 10,
    Reliability = 11,
    DestinationOrder = 12,
    History = 13,
    ResourceLimits = 14,
    EntityFactory = 15,
    WriterDataLifecycle = 16,
    ReaderDataLifecycle = 17,
    TopicData = 18,
    GroupData = 19,
    TransportPriority = 20,
    Lifespan = 21,
    DurabilityService = 22,
    DataRepresentation = 23,
    TypeConsistencyEnforcement = 24,
}

/// offered QoS가 requested QoS와 호환되지 않는 정책을 `QosPolicyId` 오름차순으로 돌려준다.
/// 비어 있으면 호환이다.
///
/// RxO 정책 8개 (DDS 1.4 Sec. 2.2.3)와 DATA_REPRESENTATION (XTypes 1.3 Sec. 7.6.3.1.1)을 본다.
/// PARTITION은 RxO 정책이 아니라 별도의 매칭 조건이므로 [`partitions_match`]로 따로 본다.
pub fn check_rxo(offered: &PublicationQos, requested: &SubscriptionQos) -> Vec<QosPolicyId> {
    let mut incompatible = Vec::new();

    // DDS 1.4 Sec. 2.2.3.4: offered kind >= requested kind
    if offered.durability < requested.durability {
        incompatible.push(QosPolicyId::Durability);
    }
    // DDS 1.4 Sec. 2.2.3.6: access_scope는 offered >= requested,
    // coherent_access와 ordered_access는 requested가 FALSE이거나 둘 다 TRUE
    if !presentation_compatible(offered.presentation, requested.presentation) {
        incompatible.push(QosPolicyId::Presentation);
    }
    // DDS 1.4 Sec. 2.2.3.7: offered period <= requested period
    if offered.deadline > requested.deadline {
        incompatible.push(QosPolicyId::Deadline);
    }
    // DDS 1.4 Sec. 2.2.3.8: offered duration <= requested duration
    if offered.latency_budget > requested.latency_budget {
        incompatible.push(QosPolicyId::LatencyBudget);
    }
    // DDS 1.4 Sec. 2.2.3.9: 같아야 한다
    if offered.ownership != requested.ownership {
        incompatible.push(QosPolicyId::Ownership);
    }
    // DDS 1.4 Sec. 2.2.3.11: offered kind >= requested kind, offered lease <= requested lease
    if offered.liveliness.kind < requested.liveliness.kind
        || offered.liveliness.lease_duration > requested.liveliness.lease_duration
    {
        incompatible.push(QosPolicyId::Liveliness);
    }
    // DDS 1.4 Sec. 2.2.3.14: offered kind >= requested kind
    if offered.reliability < requested.reliability {
        incompatible.push(QosPolicyId::Reliability);
    }
    // DDS 1.4 Sec. 2.2.3.17: offered kind >= requested kind
    if offered.destination_order < requested.destination_order {
        incompatible.push(QosPolicyId::DestinationOrder);
    }
    // XTypes 1.3 Sec. 7.6.3.1.1: writer가 제공하는 표현 (첫 원소)이 reader의 목록에 있어야 한다
    if !data_representation_compatible(&offered.data_representation, &requested.data_representation)
    {
        incompatible.push(QosPolicyId::DataRepresentation);
    }

    incompatible
}

fn presentation_compatible(offered: Presentation, requested: Presentation) -> bool {
    offered.access_scope >= requested.access_scope
        && (!requested.coherent_access || offered.coherent_access)
        && (!requested.ordered_access || offered.ordered_access)
}

fn data_representation_compatible(
    offered: &[DataRepresentationId],
    requested: &[DataRepresentationId],
) -> bool {
    let offered = offered
        .first()
        .copied()
        .unwrap_or(DataRepresentationId::XCDR1);
    if requested.is_empty() {
        offered == DataRepresentationId::XCDR1
    } else {
        requested.contains(&offered)
    }
}

/// 두 파티션 목록에 서로 맞는 이름이 하나라도 있으면 true (DDS 1.4 Sec. 2.2.3.13).
///
/// - 빈 목록은 `""` 하나와 같다.
/// - 이름은 와일드카드 `*` (0개 이상의 문자)와 `?` (문자 하나)를 가질 수 있다. 어느 쪽의 이름이든
///   패턴이 될 수 있지만, 두 이름이 모두 와일드카드를 가지면 그 쌍은 맞지 않는다.
/// - `[...]`는 아직 구현하지 않았고 문자 그대로 비교한다.
///
/// 파티션 불일치는 "incompatible QoS"가 아니다. [`check_rxo`]에 섞지 않는 이유다.
pub fn partitions_match<A: AsRef<str>, B: AsRef<str>>(offered: &[A], requested: &[B]) -> bool {
    let offered = normalize(offered);
    let requested = normalize(requested);
    offered.iter().any(|o| {
        requested
            .iter()
            .any(|r| name_matches(o, r) || name_matches(r, o))
    })
}

fn normalize<S: AsRef<str>>(names: &[S]) -> Vec<&str> {
    if names.is_empty() {
        vec![""]
    } else {
        names.iter().map(AsRef::as_ref).collect()
    }
}

/// `pattern`이 `name`과 맞는가. 패턴에 와일드카드가 없으면 같은 문자열이어야 한다.
fn name_matches(pattern: &str, name: &str) -> bool {
    if !has_wildcard(pattern) {
        return pattern == name;
    }
    if has_wildcard(name) {
        // DDS 1.4 Sec. 2.2.3.13: "no two names that both contain wildcards will ever be
        // considered to match"
        return false;
    }
    glob(pattern.as_bytes(), name.as_bytes())
}

fn has_wildcard(name: &str) -> bool {
    name.contains(['*', '?'])
}

/// `*`와 `?`만 있는 glob 매칭. 바이트 단위라 `?`는 한 바이트와 맞는다 (Cyclone `f2aaccb4`
/// `ddsi_qosmatch.c`도 C 문자열을 바이트 단위로 비교한다).
/// 재귀 대신 마지막 `*`의 위치를 기억했다가 거기로 되돌아가는 방식이다. 최악의 경우 패턴 길이와
/// 이름 길이의 곱에 비례하고, 재귀 구현처럼 지수 시간이 되지는 않는다.
fn glob(pattern: &[u8], name: &[u8]) -> bool {
    let (mut p, mut n) = (0, 0);
    let mut star: Option<(usize, usize)> = None; // (패턴에서 마지막 `*`의 다음, 그때의 name 위치)
    while n < name.len() {
        match pattern.get(p) {
            Some(b'*') => {
                star = Some((p + 1, n));
                p += 1;
            }
            Some(b'?') => {
                p += 1;
                n += 1;
            }
            Some(&c) if c == name[n] => {
                p += 1;
                n += 1;
            }
            _ => match star {
                // 마지막 `*`가 문자를 하나 더 맞춘 것으로 하고 그다음부터 다시 시도한다
                Some((sp, sn)) => {
                    p = sp;
                    n = sn + 1;
                    star = Some((sp, n));
                }
                None => return false,
            },
        }
    }
    pattern[p..].iter().all(|&c| c == b'*')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn offered() -> PublicationQos {
        PublicationQos::default()
    }

    fn requested() -> SubscriptionQos {
        SubscriptionQos::default()
    }

    // --- 순서와 번호 ---

    #[test]
    fn kind_order_matches_spec() {
        use DurabilityKind::*;
        assert!(ReliabilityKind::BestEffort < ReliabilityKind::Reliable);
        assert!(Volatile < TransientLocal && TransientLocal < Transient && Transient < Persistent);
        assert!(LivelinessKind::Automatic < LivelinessKind::ManualByParticipant);
        assert!(LivelinessKind::ManualByParticipant < LivelinessKind::ManualByTopic);
        assert!(
            DestinationOrderKind::ByReceptionTimestamp < DestinationOrderKind::BySourceTimestamp
        );
        assert!(PresentationAccessScope::Instance < PresentationAccessScope::Topic);
        assert!(PresentationAccessScope::Topic < PresentationAccessScope::Group);
    }

    #[test]
    fn policy_id_values_match_dds_1_4() {
        assert_eq!(QosPolicyId::Durability as u32, 2);
        assert_eq!(QosPolicyId::Partition as u32, 10);
        assert_eq!(QosPolicyId::Reliability as u32, 11);
        assert_eq!(QosPolicyId::DurabilityService as u32, 22);
        assert_eq!(QosPolicyId::DataRepresentation as u32, 23);
    }

    #[test]
    fn data_representation_values_match_xtypes_1_3() {
        assert_eq!(DataRepresentationId::XCDR1.0, 0);
        assert_eq!(DataRepresentationId::XML.0, 1);
        assert_eq!(DataRepresentationId::XCDR2.0, 2);
    }

    #[test]
    fn duration_infinite_is_greatest() {
        assert!(Duration::ZERO < Duration::from_millis(1));
        assert!(Duration::from_secs(u64::MAX) < Duration::Infinite);
        assert_eq!(Duration::Infinite, Duration::Infinite);
    }

    // --- 기본값 ---

    #[test]
    fn defaults_match_dds_1_4() {
        let w = offered();
        let r = requested();
        assert_eq!(w.reliability, ReliabilityKind::Reliable);
        assert_eq!(r.reliability, ReliabilityKind::BestEffort);
        assert_eq!(w.durability, DurabilityKind::Volatile);
        assert_eq!(r.durability, DurabilityKind::Volatile);
        assert_eq!(w.deadline, Duration::Infinite);
        assert_eq!(w.latency_budget, Duration::ZERO);
        assert_eq!(w.liveliness.lease_duration, Duration::Infinite);
        assert_eq!(w.ownership, OwnershipKind::Shared);
        assert_eq!(
            w.presentation.access_scope,
            PresentationAccessScope::Instance
        );
        assert!(w.partition.is_empty() && w.data_representation.is_empty());
    }

    #[test]
    fn default_writer_and_default_reader_are_compatible() {
        assert_eq!(check_rxo(&offered(), &requested()), vec![]);
        assert!(partitions_match(
            &offered().partition,
            &requested().partition
        ));
    }

    // --- 정책별 규칙: 호환 하나, 불호환 하나 ---

    #[test]
    fn reliability_best_effort_offered_vs_reliable_requested_is_incompatible() {
        let w = PublicationQos {
            reliability: ReliabilityKind::BestEffort,
            ..offered()
        };
        let r = SubscriptionQos {
            reliability: ReliabilityKind::Reliable,
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![QosPolicyId::Reliability]);
        // 반대 방향 (RELIABLE 제공, BEST_EFFORT 요구)은 호환
        let w = PublicationQos {
            reliability: ReliabilityKind::Reliable,
            ..offered()
        };
        let r = SubscriptionQos {
            reliability: ReliabilityKind::BestEffort,
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn durability_offered_lower_than_requested_is_incompatible() {
        let r = SubscriptionQos {
            durability: DurabilityKind::TransientLocal,
            ..requested()
        };
        assert_eq!(check_rxo(&offered(), &r), vec![QosPolicyId::Durability]);
        let w = PublicationQos {
            durability: DurabilityKind::TransientLocal,
            ..offered()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn deadline_offered_longer_than_requested_is_incompatible() {
        let w = PublicationQos {
            deadline: Duration::from_millis(200),
            ..offered()
        };
        let r = SubscriptionQos {
            deadline: Duration::from_millis(100),
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![QosPolicyId::Deadline]);
        // 같으면 호환 (<=)
        let w = PublicationQos {
            deadline: Duration::from_millis(100),
            ..offered()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
        // 기본값 (무한대)을 제공하고 유한한 값을 요구하면 불호환
        let r = SubscriptionQos {
            deadline: Duration::from_secs(1),
            ..requested()
        };
        assert_eq!(check_rxo(&offered(), &r), vec![QosPolicyId::Deadline]);
    }

    #[test]
    fn latency_budget_offered_longer_than_requested_is_incompatible() {
        let w = PublicationQos {
            latency_budget: Duration::from_millis(10),
            ..offered()
        };
        assert_eq!(
            check_rxo(&w, &requested()),
            vec![QosPolicyId::LatencyBudget]
        );
        let r = SubscriptionQos {
            latency_budget: Duration::from_millis(10),
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn ownership_must_be_equal() {
        let w = PublicationQos {
            ownership: OwnershipKind::Exclusive,
            ..offered()
        };
        assert_eq!(check_rxo(&w, &requested()), vec![QosPolicyId::Ownership]);
        let r = SubscriptionQos {
            ownership: OwnershipKind::Exclusive,
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn liveliness_kind_lower_or_lease_longer_is_incompatible() {
        let r = SubscriptionQos {
            liveliness: Liveliness {
                kind: LivelinessKind::ManualByTopic,
                lease_duration: Duration::Infinite,
            },
            ..requested()
        };
        assert_eq!(check_rxo(&offered(), &r), vec![QosPolicyId::Liveliness]);

        let r = SubscriptionQos {
            liveliness: Liveliness {
                kind: LivelinessKind::Automatic,
                lease_duration: Duration::from_secs(5),
            },
            ..requested()
        };
        assert_eq!(check_rxo(&offered(), &r), vec![QosPolicyId::Liveliness]);
        let w = PublicationQos {
            liveliness: Liveliness {
                kind: LivelinessKind::Automatic,
                lease_duration: Duration::from_secs(5),
            },
            ..offered()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn destination_order_offered_lower_than_requested_is_incompatible() {
        let r = SubscriptionQos {
            destination_order: DestinationOrderKind::BySourceTimestamp,
            ..requested()
        };
        assert_eq!(
            check_rxo(&offered(), &r),
            vec![QosPolicyId::DestinationOrder]
        );
        let w = PublicationQos {
            destination_order: DestinationOrderKind::BySourceTimestamp,
            ..offered()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn presentation_checks_scope_coherent_and_ordered() {
        let r = SubscriptionQos {
            presentation: Presentation {
                access_scope: PresentationAccessScope::Topic,
                ..Presentation::default()
            },
            ..requested()
        };
        assert_eq!(check_rxo(&offered(), &r), vec![QosPolicyId::Presentation]);

        let r = SubscriptionQos {
            presentation: Presentation {
                coherent_access: true,
                ..Presentation::default()
            },
            ..requested()
        };
        assert_eq!(check_rxo(&offered(), &r), vec![QosPolicyId::Presentation]);
        let w = PublicationQos {
            presentation: Presentation {
                coherent_access: true,
                ..Presentation::default()
            },
            ..offered()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
        // 제공만 TRUE이고 요구가 FALSE이면 호환
        assert_eq!(check_rxo(&w, &requested()), vec![]);
    }

    #[test]
    fn data_representation_offered_must_be_in_requested_list() {
        // 둘 다 빈 목록: XCDR1끼리
        assert_eq!(check_rxo(&offered(), &requested()), vec![]);
        // writer XCDR2, reader 기본값 (XCDR1만)
        let w = PublicationQos {
            data_representation: vec![DataRepresentationId::XCDR2],
            ..offered()
        };
        assert_eq!(
            check_rxo(&w, &requested()),
            vec![QosPolicyId::DataRepresentation]
        );
        // reader가 둘 다 받으면 호환
        let r = SubscriptionQos {
            data_representation: vec![DataRepresentationId::XCDR1, DataRepresentationId::XCDR2],
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn multiple_incompatible_policies_are_returned_in_id_order() {
        let w = PublicationQos {
            reliability: ReliabilityKind::BestEffort,
            ownership: OwnershipKind::Exclusive,
            ..offered()
        };
        let r = SubscriptionQos {
            reliability: ReliabilityKind::Reliable,
            durability: DurabilityKind::TransientLocal,
            ..requested()
        };
        assert_eq!(
            check_rxo(&w, &r),
            vec![
                QosPolicyId::Durability,
                QosPolicyId::Ownership,
                QosPolicyId::Reliability
            ]
        );
    }

    // --- 파티션 ---

    #[test]
    fn partition_empty_list_equals_empty_name() {
        assert!(partitions_match::<&str, &str>(&[], &[]));
        assert!(partitions_match(&[""], &names(&[])));
        assert!(!partitions_match(&names(&[]), &["A"]));
    }

    #[test]
    fn partition_literal_names() {
        assert!(partitions_match(&["A"], &["A"]));
        assert!(!partitions_match(&["A"], &["B"]));
        assert!(partitions_match(&["A", "B"], &["C", "B"]));
    }

    #[test]
    fn partition_wildcards_on_either_side() {
        assert!(partitions_match(&["*"], &["A"]));
        assert!(partitions_match(&["A"], &["*"]));
        assert!(partitions_match(&["*"], &names(&[]))); // `*`는 ""와도 맞는다
        assert!(partitions_match(&["A?"], &["AB"]));
        assert!(!partitions_match(&["A?"], &["A"]));
        assert!(partitions_match(&["sensor/*"], &["sensor/temp"]));
        assert!(!partitions_match(&["sensor/*"], &["actuator/x"]));
    }

    #[test]
    fn partition_two_wildcard_names_never_match() {
        assert!(!partitions_match(&["*"], &["*"]));
        assert!(!partitions_match(&["A*"], &["A*"]));
        // 목록 단위가 아니라 이름 쌍 단위다: 와일드카드가 없는 쌍이 하나 있으면 맞는다
        assert!(partitions_match(&["A*", "B"], &["C*", "B"]));
    }

    #[test]
    fn partition_star_backtracks() {
        assert!(partitions_match(&["a*b*c"], &["axxbxxc"]));
        assert!(partitions_match(&["a*b*c"], &["abc"]));
        assert!(!partitions_match(&["a*b*c"], &["axxbxx"]));
        assert!(partitions_match(&["*c"], &["abcc"]));
    }

    #[test]
    fn partition_bracket_is_literal_for_now() {
        assert!(partitions_match(&["[ab]"], &["[ab]"]));
        assert!(!partitions_match(&["[ab]"], &["a"]));
    }

    // --- 캡처에서 읽은 QoS 쌍 (captures/cyclone-fastdds/). 판정이 벤더 로그와 같아야 한다.
    // 값은 Wireshark에서 읽어 직접 적었다 (pcap을 읽는 골든 테스트는 파서가 생긴 뒤에).
    // SEDP에 없는 PID는 그 엔티티의 기본값이므로 `..Default::default()`로 채운다.

    #[test]
    fn capture_rel_cyclone_pub_is_compatible() {
        // 22: Cyclone writer 정보. PID_RELIABILITY RELIABLE. DURABILITY, DATA_REPRESENTATION 없음
        let w = PublicationQos {
            reliability: ReliabilityKind::Reliable,
            ..offered()
        };
        // 47: Fast DDS reader 정보. DURABILITY VOLATILE, RELIABILITY RELIABLE, DATA_REPRESENTATION [XCDR]
        let r = SubscriptionQos {
            durability: DurabilityKind::Volatile,
            reliability: ReliabilityKind::Reliable,
            data_representation: vec![DataRepresentationId::XCDR1],
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
        assert!(partitions_match(&w.partition, &r.partition));
        // 로그: 양쪽에 matched
    }

    #[test]
    fn capture_be_cyclone_pub_is_compatible() {
        // 27: Cyclone writer 정보. PID_RELIABILITY BEST_EFFORT
        let w = PublicationQos {
            reliability: ReliabilityKind::BestEffort,
            ..offered()
        };
        // 32: Fast DDS reader 정보. DURABILITY VOLATILE, DATA_REPRESENTATION [XCDR]. RELIABILITY 없음
        let r = SubscriptionQos {
            durability: DurabilityKind::Volatile,
            data_representation: vec![DataRepresentationId::XCDR1],
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn capture_rel_fastdds_pub_is_compatible() {
        // 17: Fast DDS writer 정보. DURABILITY VOLATILE, DATA_REPRESENTATION [XCDR]. RELIABILITY 없음
        let w = PublicationQos {
            durability: DurabilityKind::Volatile,
            data_representation: vec![DataRepresentationId::XCDR1],
            ..offered()
        };
        // 23: Cyclone reader 정보. PID_RELIABILITY RELIABLE
        let r = SubscriptionQos {
            reliability: ReliabilityKind::Reliable,
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn capture_xcdr2_is_compatible() {
        // 29: Cyclone writer 정보. RELIABILITY RELIABLE, DATA_REPRESENTATION [XCDR2]
        let w = PublicationQos {
            reliability: ReliabilityKind::Reliable,
            data_representation: vec![DataRepresentationId::XCDR2],
            ..offered()
        };
        // 38: Fast DDS reader 정보. DURABILITY VOLATILE, RELIABILITY RELIABLE, DATA_REPRESENTATION [XCDR2]
        let r = SubscriptionQos {
            durability: DurabilityKind::Volatile,
            reliability: ReliabilityKind::Reliable,
            data_representation: vec![DataRepresentationId::XCDR2],
            ..requested()
        };
        assert_eq!(check_rxo(&w, &r), vec![]);
    }

    #[test]
    fn capture_qos_mismatch_reports_reliability() {
        // 28: Cyclone writer 정보. PID_RELIABILITY BEST_EFFORT
        let w = PublicationQos {
            reliability: ReliabilityKind::BestEffort,
            ..offered()
        };
        // 35: Fast DDS reader 정보. DURABILITY VOLATILE, RELIABILITY RELIABLE, DATA_REPRESENTATION [XCDR]
        let r = SubscriptionQos {
            durability: DurabilityKind::Volatile,
            reliability: ReliabilityKind::Reliable,
            data_representation: vec![DataRepresentationId::XCDR1],
            ..requested()
        };
        // 로그: on_offered_incompatible_qos / on_requested_incompatible_qos ... 11 (RELIABILITY)
        assert_eq!(check_rxo(&w, &r), vec![QosPolicyId::Reliability]);
        assert_eq!(QosPolicyId::Reliability as u32, 11);
    }

    #[test]
    fn capture_partition_mismatch_is_rxo_compatible_but_partitions_differ() {
        // 23: Cyclone writer 정보. PID_RELIABILITY RELIABLE. PID_PARTITION 없음
        let w = PublicationQos {
            reliability: ReliabilityKind::Reliable,
            ..offered()
        };
        // 33: Fast DDS reader 정보. DURABILITY VOLATILE, RELIABILITY RELIABLE, PARTITION ["A"],
        //     DATA_REPRESENTATION [XCDR]
        let r = SubscriptionQos {
            durability: DurabilityKind::Volatile,
            reliability: ReliabilityKind::Reliable,
            partition: names(&["A"]),
            data_representation: vec![DataRepresentationId::XCDR1],
            ..requested()
        };
        // 로그: 양쪽 모두 아무 알림 없음. RxO는 호환이고 파티션만 맞지 않는다
        assert_eq!(check_rxo(&w, &r), vec![]);
        assert!(!partitions_match(&w.partition, &r.partition));
    }
}
