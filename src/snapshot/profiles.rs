use crate::model::ProfileMatch;

pub struct Profile {
    pub id: String,
    pub version: Option<&'static str>,
    pub match_kind: ProfileMatch,
}

const KNOWN: [(&str, &str); 10] = [
    ("d20a1be77c3d3c41b2a5accaee1ce549", "3.4.3"),
    ("80a49c7111088100a233b2ae788e1f48", "3.5.0"),
    ("cda356e9bae476c70de33809fd92e009", "3.5.1"),
    ("f956f595844a2f845a55707faaaa51e4", "3.6.2"),
    ("d91c0e6f35f0eb2e44124e8f42aa44a7", "3.7.0"),
    ("830f4f59e7969c70b595182826435c19", "3.8.1"),
    ("97ff04a728735e6b6b098bdf983faaba", "3.9.2"),
    ("1ce86630892e2dca9a8543fdb8ed8e22", "3.10.7"),
    ("78da37fed6bf1489361a312568249f3f", "3.11.0"),
    ("bf2a89a0870c9457c268c1bc89403fe1", "3.12.0-dev"),
];

pub fn detect(hash: &str, runtime_version: Option<&str>) -> Profile {
    if let Some((_, version)) = KNOWN.iter().find(|(candidate, _)| *candidate == hash) {
        return Profile {
            id: match version.split_once('.').map(|(major, rest)| {
                let minor = rest.split('.').next().unwrap_or("unknown");
                format!("dart-{major}.{minor}-object-header")
            }) {
                Some(value) => value,
                None => "dart-unknown".to_owned(),
            },
            version: Some(version),
            match_kind: ProfileMatch::Exact,
        };
    }

    let supported_runtime = runtime_version
        .and_then(|version| {
            let mut parts = version.split('.');
            let major = parts.next()?.parse::<u32>().ok()?;
            let minor = parts.next()?.parse::<u32>().ok()?;
            Some((major, minor))
        })
        .is_some_and(|(major, minor)| major == 3 && (4..=11).contains(&minor));

    if supported_runtime {
        Profile {
            id: "dart-3.4-3.11-compatible-object-header".to_owned(),
            version: None,
            match_kind: ProfileMatch::Compatible,
        }
    } else {
        Profile {
            id: "unknown".to_owned(),
            version: None,
            match_kind: ProfileMatch::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::detect;
    use crate::model::ProfileMatch;

    #[test]
    fn identifies_dart_311_snapshot() {
        let profile = detect("78da37fed6bf1489361a312568249f3f", Some("3.11.4"));
        assert!(matches!(profile.match_kind, ProfileMatch::Exact));
        assert_eq!(profile.version, Some("3.11.0"));
    }
}
