//! УКЭП (GOST) CMS signing via the CryptoPro `cryptcp` CLI.
//! Port of docs/znack_api/znack/signature/CryptoProSignatureProvider.java.
//!
//! Two signature shapes are used by Честный ЗНАК:
//!   - attached  — authentication challenge (/auth/simpleSignIn `data` field)
//!   - detached  — СУЗ X-Signature header and True API document `signature`

use crate::error::{AppError, AppResult};
use crate::znack::models::ZnackSettings;
use base64::Engine;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignMode {
    Attached,
    Detached,
}

/// Sign `payload` and return the CMS (DER bytes). Runs cryptcp synchronously —
/// call through `tauri::async_runtime::spawn_blocking` from async contexts.
pub fn sign(settings: &ZnackSettings, payload: &[u8], mode: SignMode) -> AppResult<Vec<u8>> {
    let thumbprint = settings.cert_thumbprint.trim();
    if thumbprint.is_empty() {
        return Err(AppError::Msg("Chưa chọn chứng thư CryptoPro (thumbprint).".into()));
    }
    let cryptcp = resolve_cryptcp(&settings.cryptcp_path)?;
    let dir = tempfile_dir()?;
    let input = dir.join("payload.bin");
    let output = dir.join("signature.p7s");
    std::fs::write(&input, payload)?;

    let mode_flag = match mode {
        SignMode::Attached => "-attached",
        SignMode::Detached => "-detached",
    };
    let result = run_with_timeout(
        Command::new(&cryptcp)
            .arg("-sign")
            .arg("-uMy")
            .arg("-thumbprint")
            .arg(thumbprint)
            .arg("-der")
            .arg(mode_flag)
            .arg(&input)
            .arg(&output),
        Duration::from_secs(settings.resolved_timeout_secs()),
    );

    let cleanup = || {
        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
        let _ = std::fs::remove_dir(&dir);
    };

    let run = match result {
        Ok(r) => r,
        Err(e) => {
            cleanup();
            return Err(e);
        }
    };
    if !run.success {
        let diag = run.diagnostic();
        cleanup();
        return Err(AppError::Msg(classify_failure(&diag, run.exit_code)));
    }
    let raw = match std::fs::read(&output) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        _ => run.stdout.clone(),
    };
    cleanup();
    normalize_cms(&raw)
}

/// Quick availability probe used by the "test signing" button and pipeline
/// prerequisites: signs a small payload end-to-end.
pub fn test_sign(settings: &ZnackSettings) -> AppResult<String> {
    let cms = sign(settings, b"wcode-znack-test", SignMode::Attached)?;
    Ok(format!("OK — chữ ký CMS {} byte.", cms.len()))
}

pub fn to_base64(cms: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(cms)
}

// --- certificate discovery (certmgr -list) -----------------------------------
// Port of docs/znack_api/znack/signature/CryptoProCertificateDiscoveryService.java

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateInfo {
    pub thumbprint: String,
    pub subject: String,
    pub inn: String,
    pub not_after: String,
}

/// List usable (valid, with private key) certificates from the CryptoPro
/// "My" store via `certmgr -list -store uMy`.
pub fn list_certificates(timeout_secs: u64) -> AppResult<Vec<CertificateInfo>> {
    let certmgr = resolve_tool("certmgr")?;
    let run = run_with_timeout(
        Command::new(&certmgr).arg("-list").arg("-store").arg("uMy"),
        Duration::from_secs(timeout_secs.max(10)),
    )?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    if !run.success {
        let lower = text.to_lowercase();
        if lower.contains("empty certificate list") || lower.contains("список сертификатов пуст") {
            return Ok(vec![]);
        }
        return Err(AppError::Msg(format!(
            "Không đọc được danh sách chứng thư (certmgr exit {}): {}",
            run.exit_code,
            text.chars().take(400).collect::<String>()
        )));
    }
    let now = chrono::Local::now().naive_local();
    Ok(parse_certificates(&text)
        .into_iter()
        .filter(|c| c.usable_at(now))
        .map(|c| c.info)
        .collect())
}

struct ParsedCert {
    info: CertificateInfo,
    not_before: Option<chrono::NaiveDateTime>,
    not_after: Option<chrono::NaiveDateTime>,
    private_key: bool,
}

impl ParsedCert {
    fn usable_at(&self, now: chrono::NaiveDateTime) -> bool {
        if self.info.thumbprint.is_empty() || !self.private_key {
            return false;
        }
        if let Some(after) = self.not_after {
            if after < now {
                return false;
            }
        }
        if let Some(before) = self.not_before {
            if before > now {
                return false;
            }
        }
        true
    }
}

fn parse_certificates(output: &str) -> Vec<ParsedCert> {
    let mut certs = vec![];
    let mut fields: Vec<(String, String)> = vec![];
    let flush = |fields: &mut Vec<(String, String)>, certs: &mut Vec<ParsedCert>| {
        if fields.is_empty() {
            return;
        }
        let get = |names: &[&str]| -> String {
            for (k, v) in fields.iter() {
                if names.iter().any(|n| k == n) {
                    return v.clone();
                }
            }
            String::new()
        };
        let thumbprint = get(&[
            "sha1 thumbprint", "sha1 hash", "thumbprint", "отпечаток", "отпечаток sha1",
            "хэш sha1", "hash",
        ])
        .replace(' ', "")
        .trim_start_matches("0x")
        .to_string();
        let subject = get(&["subject", "субъект"]);
        let inn = extract_inn(&subject);
        let not_before = parse_cert_date(&get(&[
            "not valid before", "valid from", "действителен с", "выдан",
        ]));
        let not_after_raw = get(&["not valid after", "valid to", "действителен до", "истекает"]);
        let private_key_raw =
            get(&["privatekey link", "private key", "ссылка на закрытый ключ", "закрытый ключ"]);
        let pk_lower = private_key_raw.to_lowercase();
        // No "PrivateKey Link" line = no usable key (matches the Java reference).
        let private_key = !private_key_raw.is_empty()
            && !(pk_lower.contains("no")
                || pk_lower.contains("нет")
                || pk_lower.contains("absent")
                || pk_lower.contains("missing"));
        if !thumbprint.is_empty() {
            certs.push(ParsedCert {
                info: CertificateInfo {
                    thumbprint,
                    subject,
                    inn,
                    not_after: not_after_raw,
                },
                not_before,
                not_after: parse_cert_date(&get(&[
                    "not valid after", "valid to", "действителен до", "истекает",
                ])),
                private_key,
            });
        }
        fields.clear();
    };

    // Long Subject/Issuer DNs wrap onto indented continuation lines
    // ("  ИНН=770…"); glue them back so the INN/CN survive parsing.
    let is_multiline_field = |key: &str| matches!(key, "subject" | "субъект" | "issuer" | "издатель");
    let is_dn_attribute = |key: &str| {
        matches!(
            key,
            "cn" | "o" | "ou" | "sn" | "g" | "gn" | "inn" | "инн" | "e" | "email"
                | "emailaddress" | "c" | "s" | "st" | "l" | "street" | "t" | "ogrn"
                | "огрн" | "snils" | "снилс"
        )
    };
    let append_to = |fields: &mut Vec<(String, String)>, key: &str, extra: &str| {
        if let Some((_, v)) = fields.iter_mut().rev().find(|(k, _)| k == key) {
            v.push_str(", ");
            v.push_str(extra);
        }
    };

    let mut last_key: Option<String> = None;
    for raw_line in output.replace('\r', "").lines() {
        let trimmed = raw_line.trim();
        // "N-------" starts a new certificate block.
        let boundary = {
            let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
            !digits.is_empty() && trimmed[digits.len()..].chars().filter(|c| *c == '-').count() >= 3
                && trimmed[digits.len()..].chars().all(|c| c == '-' || c.is_whitespace())
        };
        if boundary {
            flush(&mut fields, &mut certs);
            last_key = None;
            continue;
        }
        let indented = raw_line.starts_with([' ', '\t']);
        if let Some(pos) = raw_line.find([':', '=']) {
            let key = raw_line[..pos].trim().to_lowercase();
            let key = key.split_whitespace().collect::<Vec<_>>().join(" ");
            let value = raw_line[pos + 1..].trim().to_string();
            if key.is_empty() {
                continue;
            }
            // A wrapped DN attribute belongs to the preceding Subject/Issuer.
            if indented
                && is_dn_attribute(&key)
                && last_key.as_deref().is_some_and(is_multiline_field)
            {
                append_to(&mut fields, last_key.as_deref().unwrap(), trimmed);
                continue;
            }
            // A second thumbprint-ish key without a boundary = next cert.
            let is_selector = matches!(
                key.as_str(),
                "sha1 thumbprint" | "sha1 hash" | "thumbprint" | "отпечаток"
                    | "отпечаток sha1" | "хэш sha1" | "hash"
            );
            if is_selector
                && fields.iter().any(|(k, _)| {
                    matches!(
                        k.as_str(),
                        "sha1 thumbprint" | "sha1 hash" | "thumbprint" | "отпечаток"
                            | "отпечаток sha1" | "хэш sha1" | "hash"
                    )
                })
            {
                flush(&mut fields, &mut certs);
            }
            fields.push((key.clone(), value));
            last_key = Some(key);
        } else if indented
            && !trimmed.is_empty()
            && last_key.as_deref().is_some_and(is_multiline_field)
        {
            append_to(&mut fields, last_key.as_deref().unwrap(), trimmed);
        }
    }
    flush(&mut fields, &mut certs);
    certs
}

fn extract_inn(subject: &str) -> String {
    // Search the ORIGINAL string (indexing an uppercased copy can misalign
    // byte offsets); cover the usual case variants of the marker directly.
    for marker in ["ИНН", "инн", "Инн", "INN", "inn"] {
        if let Some(pos) = subject.find(marker) {
            let tail = &subject[pos + marker.len()..];
            let digits: String = tail
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if digits.len() == 10 || digits.len() == 12 {
                return digits;
            }
        }
    }
    String::new()
}

fn parse_cert_date(value: &str) -> Option<chrono::NaiveDateTime> {
    let v = value
        .trim()
        .trim_end_matches(" UTC")
        .trim_end_matches(" GMT")
        .trim_end_matches(" MSK")
        .trim();
    if v.is_empty() {
        return None;
    }
    for fmt in [
        "%d/%m/%Y %H:%M:%S",
        "%d.%m.%Y %H:%M:%S",
        "%m/%d/%Y %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(v, fmt) {
            return Some(dt);
        }
    }
    for fmt in ["%d.%m.%Y", "%d/%m/%Y", "%m/%d/%Y", "%Y-%m-%d"] {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(v, fmt) {
            return Some(d.and_hms_opt(0, 0, 0).unwrap());
        }
    }
    None
}

/// Resolve a CryptoPro tool (cryptcp/certmgr) from common install dirs or PATH.
fn resolve_tool(name: &str) -> AppResult<PathBuf> {
    for dir in ["/opt/cprocsp/bin", "/opt/cprocsp/bin/amd64", "/opt/cprocsp/bin/aarch64"] {
        let path = PathBuf::from(dir).join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    if let Ok(output) = Command::new("which").arg(name).output() {
        let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if output.status.success() && !found.is_empty() {
            return Ok(PathBuf::from(found));
        }
    }
    Err(AppError::Msg(format!(
        "Không tìm thấy CryptoPro {name}. Hãy cài CryptoPro CSP trên máy này."
    )))
}

fn resolve_cryptcp(override_path: &str) -> AppResult<PathBuf> {
    let explicit = override_path.trim();
    if !explicit.is_empty() {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Ok(path);
        }
        return Err(AppError::Msg(format!("Không tìm thấy cryptcp tại: {explicit}")));
    }
    resolve_tool("cryptcp")
}

struct RunResult {
    success: bool,
    exit_code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl RunResult {
    fn diagnostic(&self) -> String {
        let mut text = String::from_utf8_lossy(&self.stderr).trim().to_string();
        if text.is_empty() {
            text = String::from_utf8_lossy(&self.stdout).trim().to_string();
        }
        text.chars().take(800).collect()
    }
}

fn run_with_timeout(command: &mut Command, timeout: Duration) -> AppResult<RunResult> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::Instant;

    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Msg(format!("Không chạy được cryptcp: {e}")))?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    let _ = s.read_to_end(&mut stdout);
                }
                if let Some(mut s) = child.stderr.take() {
                    let _ = s.read_to_end(&mut stderr);
                }
                return Ok(RunResult {
                    success: status.success(),
                    exit_code: status.code().unwrap_or(-1),
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(AppError::Msg(format!(
                        "cryptcp không phản hồi sau {}s (có thể đang chờ mã PIN của token).",
                        timeout.as_secs()
                    )));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(AppError::Msg(format!("Lỗi chờ cryptcp: {e}"))),
        }
    }
}

/// cryptcp may emit DER or base64 text; return validated DER CMS bytes.
fn normalize_cms(raw: &[u8]) -> AppResult<Vec<u8>> {
    if raw.is_empty() {
        return Err(AppError::Msg("CryptoPro trả về chữ ký rỗng.".into()));
    }
    let mut value = raw.to_vec();
    if let Ok(text) = std::str::from_utf8(raw) {
        let compact: String = text.split_whitespace().collect();
        if !compact.is_empty()
            && compact.len() % 4 == 0
            && compact
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=')
        {
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&compact) {
                value = decoded;
            }
        }
    }
    // A CMS/PKCS#7 structure is a DER SEQUENCE.
    if value.first() != Some(&0x30) {
        return Err(AppError::Msg("CryptoPro trả về dữ liệu chữ ký không hợp lệ (không phải CMS DER).".into()));
    }
    Ok(value)
}

fn classify_failure(diagnostic: &str, exit_code: i32) -> String {
    let lower = diagnostic.to_lowercase();
    let reason = if lower.contains("license") || lower.contains("licence") || lower.contains("лиценз")
        || lower.contains("0x0000065b") || lower.contains("0x65b") || lower.contains("0x20000324")
    {
        "Giấy phép CryptoPro không hợp lệ hoặc đã hết hạn"
    } else if lower.contains("cancel") || lower.contains("отмен") {
        "Thao tác ký bị huỷ"
    } else if lower.contains("expired") || lower.contains("истек") {
        "Chứng thư đã hết hạn"
    } else if lower.contains("private key") || lower.contains("закрыт") {
        "Không truy cập được khoá riêng (token chưa cắm hoặc PIN sai)"
    } else if lower.contains("certificate") || lower.contains("сертифик") {
        "Không tìm thấy chứng thư theo thumbprint"
    } else {
        "Ký CryptoPro thất bại"
    };
    format!("{reason} (cryptcp exit {exit_code}): {diagnostic}")
}

fn tempfile_dir() -> AppResult<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "wcode-znack-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::parse_certificates;

    #[test]
    fn multiline_subject_keeps_inn_and_missing_private_key_line_is_unusable() {
        let output = "\
1-------
Issuer              : CN=Test CA
Subject             : CN=ООО Ромашка, O=Ромашка,
                      ИНН=7701234567, OGRN=1027700000000
Not valid before    : 01/01/2026 00:00:00 UTC
Not valid after     : 01/01/2030 00:00:00 UTC
SHA1 Thumbprint     : aa bb cc dd ee
PrivateKey Link     : Yes
2-------
Subject             : CN=Без ключа, ИНН=7709876543
Not valid after     : 01/01/2030 00:00:00 UTC
SHA1 Thumbprint     : 11 22 33 44 55
";
        let certs = parse_certificates(output);
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0].info.thumbprint, "aabbccddee");
        assert!(certs[0].info.subject.contains("ИНН=7701234567"));
        assert_eq!(certs[0].info.inn, "7701234567");
        assert!(certs[0].private_key);
        // No "PrivateKey Link" line → not usable for signing.
        assert!(!certs[1].private_key);
        let now = chrono::Local::now().naive_local();
        assert!(certs[0].usable_at(now));
        assert!(!certs[1].usable_at(now));
    }
}
