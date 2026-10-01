//! Local single-writer coordination for the KGW migration task.
//!
//! The claim is a small text blob stored under `refs/kgw/writer-claim`.
//! Every transition is a compare-and-swap through `git update-ref <ref> <new>
//! <expected-old>`, so two sessions can never both win the same transition.
//! The claim carries a monotonic `epoch` that is the fencing token: protected
//! mutations must call `writer-claim verify` with the session id and epoch they
//! were granted, and that check fails as soon as another session takes over.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

const CLAIM_REF: &str = "refs/kgw/writer-claim";
const DEFAULT_LEASE_SECS: u64 = 1800;
const STATE_ACTIVE: &str = "active";
const STATE_RELEASED: &str = "released";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Claim {
    writer_id: String,
    session_id: String,
    host_id: String,
    state: String,
    acquired_at: u64,
    renewed_at: u64,
    lease_secs: u64,
    epoch: u64,
    base_head: String,
    worktree_fingerprint: String,
    transition_count: u64,
}

impl Claim {
    fn lease_until(&self) -> u64 {
        self.renewed_at.saturating_add(self.lease_secs)
    }

    fn is_expired(&self, now: u64) -> bool {
        self.state != STATE_ACTIVE || now >= self.lease_until()
    }

    fn encode(&self) -> String {
        format!(
            "writer_id={}\nsession_id={}\nhost_id={}\nstate={}\nacquired_at={}\nrenewed_at={}\nlease_secs={}\nepoch={}\nbase_head={}\nworktree_fingerprint={}\ntransition_count={}\n",
            self.writer_id,
            self.session_id,
            self.host_id,
            self.state,
            self.acquired_at,
            self.renewed_at,
            self.lease_secs,
            self.epoch,
            self.base_head,
            self.worktree_fingerprint,
            self.transition_count
        )
    }

    fn decode(text: &str) -> Result<Self, String> {
        let mut fields = std::collections::BTreeMap::new();
        for line in text.lines() {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("malformed writer claim line: {line}"))?;
            fields.insert(key.to_owned(), value.to_owned());
        }
        let text_field = |name: &str| {
            fields
                .get(name)
                .cloned()
                .ok_or_else(|| format!("writer claim is missing {name}"))
        };
        let number_field = |name: &str| -> Result<u64, String> {
            text_field(name)?
                .parse::<u64>()
                .map_err(|error| format!("writer claim {name} is not a number: {error}"))
        };
        Ok(Self {
            writer_id: text_field("writer_id")?,
            session_id: text_field("session_id")?,
            host_id: text_field("host_id")?,
            state: text_field("state")?,
            acquired_at: number_field("acquired_at")?,
            renewed_at: number_field("renewed_at")?,
            lease_secs: number_field("lease_secs")?,
            epoch: number_field("epoch")?,
            base_head: text_field("base_head")?,
            worktree_fingerprint: text_field("worktree_fingerprint")?,
            transition_count: number_field("transition_count")?,
        })
    }
}

struct AcquireRequest<'a> {
    writer_id: &'a str,
    session_id: &'a str,
    host_id: &'a str,
    lease_secs: u64,
    base_head: &'a str,
    worktree_fingerprint: &'a str,
}

fn validate_token(name: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err(format!(
            "{name} must be 1-128 characters of letters, digits, '-', '_', '.' or ':'"
        ));
    }
    Ok(())
}

fn decide_acquire(
    existing: Option<&Claim>,
    request: &AcquireRequest<'_>,
    now: u64,
) -> Result<Claim, String> {
    validate_token("writer-id", request.writer_id)?;
    validate_token("session-id", request.session_id)?;
    validate_token("host-id", request.host_id)?;
    if request.lease_secs == 0 {
        return Err("lease-secs must be greater than zero".to_owned());
    }
    let (epoch, acquired_at, transition_count) = match existing {
        None => (1, now, 1),
        Some(current) => {
            if !current.is_expired(now) && current.session_id != request.session_id {
                return Err(format!(
                    "writer claim is held by session {} (writer {}, epoch {}) until {}",
                    current.session_id,
                    current.writer_id,
                    current.epoch,
                    current.lease_until()
                ));
            }
            if !current.is_expired(now) {
                (
                    current.epoch,
                    current.acquired_at,
                    current.transition_count + 1,
                )
            } else {
                (current.epoch + 1, now, current.transition_count + 1)
            }
        }
    };
    Ok(Claim {
        writer_id: request.writer_id.to_owned(),
        session_id: request.session_id.to_owned(),
        host_id: request.host_id.to_owned(),
        state: STATE_ACTIVE.to_owned(),
        acquired_at,
        renewed_at: now,
        lease_secs: request.lease_secs,
        epoch,
        base_head: request.base_head.to_owned(),
        worktree_fingerprint: request.worktree_fingerprint.to_owned(),
        transition_count,
    })
}

fn verify_holder(current: &Claim, session_id: &str, epoch: u64, now: u64) -> Result<(), String> {
    if current.state != STATE_ACTIVE {
        return Err("writer claim is released".to_owned());
    }
    if current.session_id != session_id || current.epoch != epoch {
        return Err(format!(
            "stale writer: current claim is epoch {} held by another session",
            current.epoch
        ));
    }
    if now >= current.lease_until() {
        return Err("writer lease expired; re-acquire before mutating".to_owned());
    }
    Ok(())
}

fn decide_renew(
    current: &Claim,
    session_id: &str,
    epoch: u64,
    now: u64,
    worktree_fingerprint: &str,
) -> Result<Claim, String> {
    verify_holder(current, session_id, epoch, now)?;
    let mut next = current.clone();
    next.renewed_at = now;
    next.worktree_fingerprint = worktree_fingerprint.to_owned();
    next.transition_count += 1;
    Ok(next)
}

fn decide_release(
    current: &Claim,
    session_id: &str,
    epoch: u64,
    now: u64,
) -> Result<Claim, String> {
    verify_holder(current, session_id, epoch, now)?;
    let mut next = current.clone();
    next.state = STATE_RELEASED.to_owned();
    next.renewed_at = now;
    next.transition_count += 1;
    Ok(next)
}

fn git(root: &Path, args: &[&str], stdin: Option<&str>) -> Result<String, String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if let Some(input) = stdin {
        let mut pipe = child
            .stdin
            .take()
            .ok_or_else(|| "git stdin was not available".to_owned())?;
        pipe.write_all(input.as_bytes())
            .map_err(|error| format!("failed to write git stdin: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_current(root: &Path) -> Result<Option<(String, Claim)>, String> {
    let resolved = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", "--quiet", CLAIM_REF])
        .output()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !resolved.status.success() {
        return Ok(None);
    }
    let oid = String::from_utf8_lossy(&resolved.stdout).trim().to_owned();
    let text = git(root, &["cat-file", "blob", &oid], None)?;
    Ok(Some((oid, Claim::decode(&text)?)))
}

/// Atomically replaces the claim. `expected_old` is the object id the caller read
/// (or `None` when the claim must not exist yet); a concurrent change makes git
/// refuse the update, which is the compare-and-swap that prevents two winners.
fn cas_write(root: &Path, next: &Claim, expected_old: Option<&str>) -> Result<(), String> {
    let new_oid = git(
        root,
        &["hash-object", "-w", "--stdin"],
        Some(&next.encode()),
    )?
    .trim()
    .to_owned();
    let zero = "0".repeat(new_oid.len());
    let old = expected_old.unwrap_or(&zero);
    git(root, &["update-ref", CLAIM_REF, &new_oid, old], None)
        .map(|_| ())
        .map_err(|error| format!("writer claim changed concurrently: {error}"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn host_id() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-host".to_owned())
}

fn worktree_fingerprint(root: &Path) -> Result<String, String> {
    let status = git(root, &["status", "--porcelain=v1", "-uall"], None)?;
    let diff = git(root, &["diff", "--binary", "HEAD"], None)?;
    git(
        root,
        &["hash-object", "--stdin"],
        Some(&format!("{status}\n--\n{diff}")),
    )
    .map(|oid| oid.trim().to_owned())
}

fn generated_session_id(writer_id: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let digest = Sha256::digest(format!("{writer_id}|{nanos}|{}", std::process::id()));
    digest
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct Flags {
    writer_id: Option<String>,
    session_id: Option<String>,
    epoch: Option<u64>,
    lease_secs: Option<u64>,
}

fn parse_flags(args: &[String]) -> Result<Flags, String> {
    let mut flags = Flags {
        writer_id: None,
        session_id: None,
        epoch: None,
        lease_secs: None,
    };
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--writer-id" => flags.writer_id = Some(value.clone()),
            "--session-id" => flags.session_id = Some(value.clone()),
            "--epoch" => {
                flags.epoch = Some(
                    value
                        .parse()
                        .map_err(|error| format!("--epoch is not a number: {error}"))?,
                );
            }
            "--lease-secs" => {
                flags.lease_secs = Some(
                    value
                        .parse()
                        .map_err(|error| format!("--lease-secs is not a number: {error}"))?,
                );
            }
            other => return Err(format!("unknown writer-claim flag: {other}")),
        }
    }
    Ok(flags)
}

fn describe(claim: &Claim, now: u64) -> String {
    format!(
        "state={} epoch={} writer={} session={} host={} lease_remaining_secs={} transitions={} base_head={}",
        claim.state,
        claim.epoch,
        claim.writer_id,
        claim.session_id,
        claim.host_id,
        claim.lease_until().saturating_sub(now),
        claim.transition_count,
        claim.base_head
    )
}

pub fn run(root: &Path, args: &[String]) -> Result<String, String> {
    let (action, rest) = args
        .split_first()
        .ok_or_else(|| "writer-claim requires acquire|renew|verify|release|status".to_owned())?;
    let flags = parse_flags(rest)?;
    let now = now_secs();
    let current = read_current(root)?;
    let require_holder = |flags: &Flags| -> Result<(String, String, u64), String> {
        let session = flags
            .session_id
            .clone()
            .ok_or_else(|| "--session-id is required".to_owned())?;
        let epoch = flags
            .epoch
            .ok_or_else(|| "--epoch is required".to_owned())?;
        let (oid, _) = current
            .as_ref()
            .ok_or_else(|| "no writer claim exists".to_owned())?;
        Ok((oid.clone(), session, epoch))
    };
    match action.as_str() {
        "status" => Ok(match &current {
            None => "WRITER_CLAIM none".to_owned(),
            Some((_, claim)) => format!("WRITER_CLAIM {}", describe(claim, now)),
        }),
        "acquire" => {
            let writer_id = flags
                .writer_id
                .clone()
                .ok_or_else(|| "--writer-id is required".to_owned())?;
            let session_id = flags
                .session_id
                .clone()
                .unwrap_or_else(|| generated_session_id(&writer_id));
            let base_head = git(root, &["rev-parse", "HEAD"], None)?.trim().to_owned();
            let fingerprint = worktree_fingerprint(root)?;
            let host = host_id();
            let request = AcquireRequest {
                writer_id: &writer_id,
                session_id: &session_id,
                host_id: &host,
                lease_secs: flags.lease_secs.unwrap_or(DEFAULT_LEASE_SECS),
                base_head: &base_head,
                worktree_fingerprint: &fingerprint,
            };
            let next = decide_acquire(current.as_ref().map(|(_, claim)| claim), &request, now)?;
            cas_write(root, &next, current.as_ref().map(|(oid, _)| oid.as_str()))?;
            Ok(format!(
                "WRITER_CLAIM ACQUIRED epoch={} session={} lease_secs={}",
                next.epoch, next.session_id, next.lease_secs
            ))
        }
        "renew" => {
            let (oid, session, epoch) = require_holder(&flags)?;
            let claim = current.as_ref().map(|(_, claim)| claim).ok_or("no claim")?;
            let fingerprint = worktree_fingerprint(root)?;
            let next = decide_renew(claim, &session, epoch, now, &fingerprint)?;
            cas_write(root, &next, Some(&oid))?;
            Ok(format!("WRITER_CLAIM RENEWED epoch={}", next.epoch))
        }
        "verify" => {
            let (_, session, epoch) = require_holder(&flags)?;
            let claim = current.as_ref().map(|(_, claim)| claim).ok_or("no claim")?;
            verify_holder(claim, &session, epoch, now)?;
            Ok(format!("WRITER_CLAIM VERIFIED epoch={epoch}"))
        }
        "release" => {
            let (oid, session, epoch) = require_holder(&flags)?;
            let claim = current.as_ref().map(|(_, claim)| claim).ok_or("no claim")?;
            let next = decide_release(claim, &session, epoch, now)?;
            cas_write(root, &next, Some(&oid))?;
            Ok(format!("WRITER_CLAIM RELEASED epoch={}", next.epoch))
        }
        other => Err(format!("unknown writer-claim action: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(session: &'a str) -> AcquireRequest<'a> {
        AcquireRequest {
            writer_id: "writer-a",
            session_id: session,
            host_id: "host-1",
            lease_secs: 100,
            base_head: "abc",
            worktree_fingerprint: "def",
        }
    }

    #[test]
    fn first_acquire_starts_at_epoch_one() {
        let claim = decide_acquire(None, &request("s1"), 1_000).unwrap();
        assert_eq!((claim.epoch, claim.transition_count), (1, 1));
        assert_eq!(Claim::decode(&claim.encode()).unwrap(), claim);
    }

    #[test]
    fn live_claim_blocks_other_session_but_not_same_session() {
        let held = decide_acquire(None, &request("s1"), 1_000).unwrap();
        assert!(decide_acquire(Some(&held), &request("s2"), 1_050).is_err());
        let again = decide_acquire(Some(&held), &request("s1"), 1_050).unwrap();
        assert_eq!(again.epoch, held.epoch);
        assert_eq!(again.acquired_at, held.acquired_at);
    }

    #[test]
    fn expired_takeover_increments_epoch_and_fences_old_writer() {
        let held = decide_acquire(None, &request("s1"), 1_000).unwrap();
        let taken = decide_acquire(Some(&held), &request("s2"), 1_100).unwrap();
        assert_eq!(taken.epoch, 2);
        assert!(verify_holder(&taken, "s2", 2, 1_101).is_ok());
        assert!(verify_holder(&taken, "s1", 1, 1_101).is_err());
        assert!(verify_holder(&taken, "s1", 2, 1_101).is_err());
    }

    #[test]
    fn expired_lease_fails_verification_and_renewal() {
        let held = decide_acquire(None, &request("s1"), 1_000).unwrap();
        assert!(verify_holder(&held, "s1", 1, 1_099).is_ok());
        assert!(verify_holder(&held, "s1", 1, 1_100).is_err());
        assert!(decide_renew(&held, "s1", 1, 1_100, "x").is_err());
        let renewed = decide_renew(&held, "s1", 1, 1_050, "x").unwrap();
        assert_eq!((renewed.renewed_at, renewed.transition_count), (1_050, 2));
        assert_eq!(renewed.epoch, held.epoch);
    }

    #[test]
    fn release_keeps_epoch_and_next_acquire_advances_it() {
        let held = decide_acquire(None, &request("s1"), 1_000).unwrap();
        let released = decide_release(&held, "s1", 1, 1_010).unwrap();
        assert!(verify_holder(&released, "s1", 1, 1_011).is_err());
        let next = decide_acquire(Some(&released), &request("s2"), 1_011).unwrap();
        assert_eq!(next.epoch, 2);
    }

    #[test]
    fn identifiers_reject_injection_and_empty_values() {
        for bad in ["", "a b", "a\nb", "a=b", "a/b"] {
            assert!(validate_token("session-id", bad).is_err(), "{bad:?}");
        }
        assert!(Claim::decode("writer_id").is_err());
    }

    fn temp_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for args in [
            vec!["init", "-q"],
            vec![
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@example.invalid",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "x",
            ],
        ] {
            git(dir.path(), &args, None).unwrap();
        }
        dir
    }

    fn cli(root: &Path, args: &[&str]) -> Result<String, String> {
        let owned: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        run(root, &owned)
    }

    #[test]
    fn git_cas_rejects_a_stale_writer_and_enforces_fencing() {
        let repo = temp_repo();
        let root = repo.path();
        let first = cli(
            root,
            &["acquire", "--writer-id", "w1", "--session-id", "s1"],
        )
        .unwrap();
        assert!(first.contains("epoch=1"), "{first}");
        assert!(
            cli(
                root,
                &["acquire", "--writer-id", "w2", "--session-id", "s2"]
            )
            .is_err()
        );
        assert!(cli(root, &["verify", "--session-id", "s1", "--epoch", "1"]).is_ok());
        assert!(cli(root, &["verify", "--session-id", "s2", "--epoch", "1"]).is_err());

        let (stale_oid, claim) = read_current(root).unwrap().unwrap();
        cli(root, &["renew", "--session-id", "s1", "--epoch", "1"]).unwrap();
        let stale_attempt = cas_write(root, &claim, Some(&stale_oid));
        assert!(stale_attempt.is_err(), "stale compare-and-swap must fail");

        cli(root, &["release", "--session-id", "s1", "--epoch", "1"]).unwrap();
        let second = cli(
            root,
            &["acquire", "--writer-id", "w2", "--session-id", "s2"],
        )
        .unwrap();
        assert!(second.contains("epoch=2"), "{second}");
        assert!(cli(root, &["verify", "--session-id", "s1", "--epoch", "1"]).is_err());
    }
}
