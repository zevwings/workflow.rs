//! SSH 实体类型

/// SSH 密钥信息
#[derive(Debug, Clone)]
pub struct SshKeyInfo {
    /// 密钥指纹
    pub fingerprint: String,
    /// 密钥类型/算法
    pub algorithm: String,
    /// 注释（通常是路径或邮箱）
    pub comment: String,
}

impl SshKeyInfo {
    /// 解析 `ssh-add -l` 单行输出
    ///
    /// 格式：`<bits> <fingerprint> <comment> (<algorithm>)`
    /// 示例：`256 SHA256:abc123... user@host (ED25519)`
    pub fn parse_ssh_add_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            return None;
        }

        let fingerprint = parts[1].to_string();
        let rest = parts[2];

        let (comment, algorithm) = if let Some(paren_start) = rest.rfind('(') {
            let algo = rest[paren_start + 1..].trim_end_matches(')').to_string();
            let comment = rest[..paren_start].trim().to_string();
            (comment, algo)
        } else {
            (rest.to_string(), String::new())
        };

        Some(Self {
            fingerprint,
            algorithm,
            comment,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_add_line_ed25519() {
        let line = "256 SHA256:abc123def456 user@host (ED25519)";
        let info = SshKeyInfo::parse_ssh_add_line(line).unwrap();
        assert_eq!(info.fingerprint, "SHA256:abc123def456");
        assert_eq!(info.algorithm, "ED25519");
        assert_eq!(info.comment, "user@host");
    }

    #[test]
    fn test_parse_ssh_add_line_rsa() {
        let line = "4096 SHA256:def456ghi789 /home/user/.ssh/id_rsa (RSA)";
        let info = SshKeyInfo::parse_ssh_add_line(line).unwrap();
        assert_eq!(info.fingerprint, "SHA256:def456ghi789");
        assert_eq!(info.algorithm, "RSA");
        assert_eq!(info.comment, "/home/user/.ssh/id_rsa");
    }

    #[test]
    fn test_parse_ssh_add_line_no_comment() {
        let line = "256 SHA256:abc123 (ED25519)";
        let info = SshKeyInfo::parse_ssh_add_line(line).unwrap();
        assert_eq!(info.fingerprint, "SHA256:abc123");
        assert_eq!(info.algorithm, "ED25519");
    }

    #[test]
    fn test_parse_ssh_add_line_invalid() {
        assert!(SshKeyInfo::parse_ssh_add_line("invalid line").is_none());
    }
}
