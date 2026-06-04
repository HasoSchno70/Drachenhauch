//! WiFi-Management (WIFI_*) -- nativer Port von `gamebasic/modules/wifi.py`.
//! Windows-only via `netsh wlan` (std::process), Regex-Parsing. Feature `wifi`.
#![cfg(feature = "wifi")]

#[cfg(windows)]
mod imp {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn run_netsh(args: &[&str]) -> Result<(i32, String), String> {
        let out = Command::new("netsh")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|_| "WIFI: 'netsh' nicht gefunden".to_string())?;
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&out.stderr));
        Ok((out.status.code().unwrap_or(-1), text))
    }

    fn re(pat: &str) -> regex::Regex {
        regex::Regex::new(pat).unwrap()
    }

    pub fn available() -> bool {
        match run_netsh(&["wlan", "show", "interfaces"]) {
            Ok((rc, out)) => rc == 0 && (out.contains("WLAN") || out.contains("Wireless") || out.contains("Drahtlos")),
            Err(_) => false,
        }
    }

    pub fn current() -> Result<String, String> {
        let (_, out) = run_netsh(&["wlan", "show", "interfaces"])?;
        let mut state = String::new();
        for line in out.lines() {
            let ls = line.trim().to_lowercase();
            if (ls.starts_with("state") || ls.starts_with("status") || ls.starts_with("zustand")) && ls.contains(':') {
                state = ls.split_once(':').unwrap().1.trim().to_string();
                break;
            }
        }
        let connected = matches!(state.as_str(), "connected" | "verbunden" | "connecté" | "connesso" | "conectado");
        if !connected {
            return Ok(String::new());
        }
        let rx = re(r"(?m)^\s*\bSSID\b\s*:\s*(.*?)\s*$");
        Ok(rx.captures(&out).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string()).unwrap_or_default())
    }

    pub fn signal() -> Result<i64, String> {
        let (_, out) = run_netsh(&["wlan", "show", "interfaces"])?;
        let rx = re(r"(?m)^\s*\S[^:\n]*?:\s*(\d+)%\s*$");
        Ok(rx.captures(&out).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<i64>().ok()).unwrap_or(-1))
    }

    pub fn scan() -> Result<String, String> {
        let (_, out) = run_netsh(&["wlan", "show", "networks", "mode=bssid"])?;
        let ssid_rx = re(r"(?m)^SSID\s+\d+\s*:\s*(.*?)\s*$");
        let sig_rx = re(r"(?m)^\s*\S[^:\n]*?:\s*(\d+)%\s*$");
        let mut blocks: Vec<(String, Option<i64>)> = Vec::new();
        let mut cur_ssid: Option<String> = None;
        let mut cur_sig: Option<i64> = None;
        for line in out.lines() {
            if let Some(c) = ssid_rx.captures(line) {
                if let Some(s) = cur_ssid.take() {
                    blocks.push((s, cur_sig.take()));
                }
                let name = c.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                cur_ssid = Some(if name.is_empty() { "(versteckt)".to_string() } else { name });
                cur_sig = None;
                continue;
            }
            if cur_ssid.is_some() && cur_sig.is_none() {
                if let Some(c) = sig_rx.captures(line) {
                    cur_sig = c.get(1).and_then(|m| m.as_str().parse::<i64>().ok());
                }
            }
        }
        if let Some(s) = cur_ssid.take() {
            blocks.push((s, cur_sig.take()));
        }
        blocks.sort_by_key(|b| (b.1.is_none(), -(b.1.unwrap_or(0))));
        Ok(blocks.iter().map(|(s, sig)| format!("{}|{}", s, sig.unwrap_or(0))).collect::<Vec<_>>().join("\n"))
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#x27;")
    }

    fn profile_xml(ssid: &str, password: &str) -> String {
        let ssid_x = xml_escape(ssid);
        if password.is_empty() {
            format!("<?xml version=\"1.0\"?>\n<WLANProfile xmlns=\"http://www.microsoft.com/networking/WLAN/profile/v1\">\n<name>{0}</name>\n<SSIDConfig><SSID><name>{0}</name></SSID></SSIDConfig>\n<connectionType>ESS</connectionType><connectionMode>auto</connectionMode>\n<MSM><security><authEncryption><authentication>open</authentication><encryption>none</encryption><useOneX>false</useOneX></authEncryption></security></MSM>\n</WLANProfile>\n", ssid_x)
        } else {
            format!("<?xml version=\"1.0\"?>\n<WLANProfile xmlns=\"http://www.microsoft.com/networking/WLAN/profile/v1\">\n<name>{0}</name>\n<SSIDConfig><SSID><name>{0}</name></SSID></SSIDConfig>\n<connectionType>ESS</connectionType><connectionMode>auto</connectionMode>\n<MSM><security><authEncryption><authentication>WPA2PSK</authentication><encryption>AES</encryption><useOneX>false</useOneX></authEncryption><sharedKey><keyType>passPhrase</keyType><protected>false</protected><keyMaterial>{1}</keyMaterial></sharedKey></security></MSM>\n</WLANProfile>\n", ssid_x, xml_escape(password))
        }
    }

    pub fn connect(ssid: &str, password: &str) -> Result<bool, String> {
        if ssid.is_empty() {
            return Err("WIFI_CONNECT: SSID darf nicht leer sein".into());
        }
        let xml = profile_xml(ssid, password);
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("_gb_wifi_{}.xml", ssid.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>()));
        std::fs::write(&tmp, xml).map_err(|e| format!("WIFI_CONNECT: temp-Datei: {}", e))?;
        let add = run_netsh(&["wlan", "add", "profile", &format!("filename={}", tmp.display()), "user=current"]);
        let _ = std::fs::remove_file(&tmp);
        let (rc, out) = add?;
        if rc != 0 {
            return Err(format!("WIFI_CONNECT: Profil konnte nicht angelegt werden\n{}", out.trim()));
        }
        let (rc2, _) = run_netsh(&["wlan", "connect", &format!("name={}", ssid)])?;
        Ok(rc2 == 0)
    }

    pub fn disconnect() -> Result<bool, String> {
        Ok(run_netsh(&["wlan", "disconnect"])?.0 == 0)
    }

    pub fn profiles() -> Result<String, String> {
        let (_, out) = run_netsh(&["wlan", "show", "profiles"])?;
        let rx = re(r"(?im)^[^:\n]*profil[^:\n]*?:[ \t]*(.+?)[ \t]*$");
        let mut seen = std::collections::HashSet::new();
        let mut names = Vec::new();
        for c in rx.captures_iter(&out) {
            let n = c.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
            if !n.is_empty() && !n.to_lowercase().starts_with("group") && seen.insert(n.clone()) {
                names.push(n);
            }
        }
        Ok(names.join("\n"))
    }

    pub fn delete_profile(name: &str) -> Result<bool, String> {
        if name.is_empty() {
            return Err("WIFI_DELETE_PROFILE: Name darf nicht leer sein".into());
        }
        Ok(run_netsh(&["wlan", "delete", "profile", &format!("name={}", name)])?.0 == 0)
    }
}

#[cfg(not(windows))]
mod imp {
    fn unsupported<T>() -> Result<T, String> {
        Err("Modul WIFI: nur unter Windows unterstuetzt (nutzt netsh wlan).".into())
    }
    pub fn available() -> bool { false }
    pub fn current() -> Result<String, String> { unsupported() }
    pub fn signal() -> Result<i64, String> { unsupported() }
    pub fn scan() -> Result<String, String> { unsupported() }
    pub fn connect(_s: &str, _p: &str) -> Result<bool, String> { unsupported() }
    pub fn disconnect() -> Result<bool, String> { unsupported() }
    pub fn profiles() -> Result<String, String> { unsupported() }
    pub fn delete_profile(_n: &str) -> Result<bool, String> { unsupported() }
}

pub use imp::*;
