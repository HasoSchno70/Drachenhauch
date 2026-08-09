//! WiFi-Management (WIFI_*) -- nativer Port von `drachenhauch/modules/wifi.py`.
//! Windows via `netsh wlan`, Linux via `nmcli` (NetworkManager), macOS via
//! `networksetup`/`airport` (std::process, Text-Parsing). Feature `wifi`.
//!
//! **Cross-Platform-Status:** Windows-Zweig ist gegen echte Hardware
//! verifiziert. Linux-Zweig (nmcli) ist nach oeffentlicher, stabiler
//! nmcli-Doku geschrieben, aber NICHT auf echter Linux-Hardware getestet
//! (diese Entwicklungsumgebung ist Windows-only). macOS-Zweig ist der
//! unsicherste: `airport` ist ein privates, undokumentiertes Apple-Tool,
//! dessen Verhalten sich zwischen macOS-Versionen schon mehrfach geaendert
//! hat (Details bei scan()) -- ebenfalls ungetestet. Rueckmeldungen von
//! echten Linux-/macOS-Nutzern sind ausdruecklich erwuenscht.
#![cfg(feature = "wifi")]

#[cfg(windows)]
mod imp {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use std::time::Duration;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    /// Zeitlimit fuer `netsh`-Aufrufe -- ohne das kann ein haengender/
    /// ueberlasteter Netzwerk-Stack den kompletten Game-Loop-Thread
    /// unbegrenzt blockieren (`Command::output()` ist synchron/blockierend
    /// und laeuft auf demselben Thread wie Rendering/Input).
    const NETSH_TIMEOUT: Duration = Duration::from_secs(10);

    /// `netsh`-Konsolenausgabe ist auf Windows NICHT zuverlaessig eine feste
    /// Kodierung: je nach Windows-Version/Konfiguration liefert `netsh` bei
    /// umgeleitetem stdout (wie hier, `Command::output()`) mal die
    /// **OEM-Codepage** der Konsole (CP850/CP437/...), mal tatsaechlich UTF-8
    /// (empirisch auf diesem System bei `netsh wlan show profiles`
    /// verifiziert -- die vorherige Version, die blind ueber `GetOEMCP` +
    /// `MultiByteToWideChar` dekodierte, hat hier gueltiges UTF-8 kaputt
    /// gemacht statt es zu reparieren). Deshalb: erst UTF-8 versuchen (der
    /// haeufigere Fall bei umgeleitetem Output), nur bei ungueltigen Bytes auf
    /// die OEM-Codepage zurueckfallen (kein neues Crate, wie
    /// `local_datetime()` in builtins.rs -- rohe WinAPI-FFI).
    fn decode_console_output(bytes: &[u8]) -> String {
        if bytes.is_empty() {
            return String::new();
        }
        if let Ok(s) = std::str::from_utf8(bytes) {
            return s.to_string();
        }
        extern "system" {
            fn GetOEMCP() -> u32;
            fn MultiByteToWideChar(
                code_page: u32, dw_flags: u32,
                lp_multi_byte_str: *const u8, cb_multi_byte: i32,
                lp_wide_char_str: *mut u16, cch_wide_char: i32,
            ) -> i32;
        }
        unsafe {
            let cp = GetOEMCP();
            let needed = MultiByteToWideChar(cp, 0, bytes.as_ptr(), bytes.len() as i32, std::ptr::null_mut(), 0);
            if needed <= 0 {
                return String::from_utf8_lossy(bytes).into_owned();
            }
            let mut buf: Vec<u16> = vec![0u16; needed as usize];
            let written = MultiByteToWideChar(cp, 0, bytes.as_ptr(), bytes.len() as i32, buf.as_mut_ptr(), needed);
            if written <= 0 {
                return String::from_utf8_lossy(bytes).into_owned();
            }
            String::from_utf16_lossy(&buf[..written as usize])
        }
    }

    fn run_netsh(args: &[&str]) -> Result<(i32, String), String> {
        // `Command::output()` selbst hat kein Timeout -- auf einem Hilfs-Thread
        // laufen lassen und mit `recv_timeout` begrenzen (gleiches Muster wie
        // die DNS-Timeout-Aufloesung in net.rs).
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = Command::new("netsh")
                .args(&args_owned)
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            let _ = tx.send(result);
        });
        match rx.recv_timeout(NETSH_TIMEOUT) {
            Ok(Ok(out)) => {
                let mut text = decode_console_output(&out.stdout);
                text.push_str(&decode_console_output(&out.stderr));
                Ok((out.status.code().unwrap_or(-1), text))
            }
            Ok(Err(_)) => Err("WIFI: 'netsh' nicht gefunden".to_string()),
            Err(_) => Err(format!("WIFI: 'netsh' hat nach {}s nicht geantwortet", NETSH_TIMEOUT.as_secs())),
        }
    }

    fn re(pat: &str) -> regex::Regex {
        regex::Regex::new(pat).unwrap()
    }

    pub fn available() -> bool {
        // rc==0 heisst "netsh wlan show interfaces" konnte eine Schnittstelle
        // auflisten -- sprachunabhaengig und damit robuster als die vorherige
        // feste Woerterliste ("WLAN"/"Wireless"/"Drahtlos"), die auf jedem
        // Windows jenseits dieser drei Sprachen faelschlich FALSE lieferte
        // (enger als current()'s 5-Sprachen-Erkennung).
        matches!(run_netsh(&["wlan", "show", "interfaces"]), Ok((0, out)) if !out.trim().is_empty())
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

#[cfg(target_os = "linux")]
mod imp {
    use std::process::Command;
    use std::time::Duration;

    const NMCLI_TIMEOUT: Duration = Duration::from_secs(10);

    fn run_nmcli(args: &[&str]) -> Result<(i32, String), String> {
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = Command::new("nmcli").args(&args_owned).output();
            let _ = tx.send(result);
        });
        match rx.recv_timeout(NMCLI_TIMEOUT) {
            Ok(Ok(out)) => {
                let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&out.stderr));
                Ok((out.status.code().unwrap_or(-1), text))
            }
            Ok(Err(_)) => Err("WIFI: 'nmcli' nicht gefunden (NetworkManager installiert?)".to_string()),
            Err(_) => Err(format!("WIFI: 'nmcli' hat nach {}s nicht geantwortet", NMCLI_TIMEOUT.as_secs())),
        }
    }

    /// nmcli-Terse-Output (`-t`) trennt Felder mit `:`; ein `:` INNERHALB
    /// eines Feldwerts (z.B. in einer SSID) kommt als `\:` zurueck. Ein
    /// simples `split(':')` wuerde solche SSIDs falsch in zu viele Felder
    /// zerlegen.
    fn split_terse(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut cur = String::new();
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    cur.push(next);
                    continue;
                }
            }
            if c == ':' {
                fields.push(std::mem::take(&mut cur));
            } else {
                cur.push(c);
            }
        }
        fields.push(cur);
        fields
    }

    pub fn available() -> bool {
        matches!(run_nmcli(&["-t", "-f", "TYPE", "dev"]), Ok((0, out)) if out.lines().any(|l| l.trim() == "wifi"))
    }

    pub fn current() -> Result<String, String> {
        let (_, out) = run_nmcli(&["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])?;
        for line in out.lines() {
            let f = split_terse(line);
            if f.len() >= 2 && f[0] == "yes" {
                return Ok(f[1].clone());
            }
        }
        Ok(String::new())
    }

    pub fn signal() -> Result<i64, String> {
        let (_, out) = run_nmcli(&["-t", "-f", "ACTIVE,SIGNAL", "dev", "wifi"])?;
        for line in out.lines() {
            let f = split_terse(line);
            if f.len() >= 2 && f[0] == "yes" {
                return Ok(f[1].parse::<i64>().unwrap_or(-1));
            }
        }
        Ok(-1)
    }

    pub fn scan() -> Result<String, String> {
        let (_, out) = run_nmcli(&["-t", "-f", "SSID,SIGNAL", "dev", "wifi", "list", "--rescan", "yes"])?;
        let mut rows: Vec<(String, i64)> = Vec::new();
        for line in out.lines() {
            let f = split_terse(line);
            if f.len() < 2 {
                continue;
            }
            let ssid = if f[0].is_empty() { "(versteckt)".to_string() } else { f[0].clone() };
            let sig = f[1].parse::<i64>().unwrap_or(0);
            rows.push((ssid, sig));
        }
        rows.sort_by_key(|(_, s)| -s);
        Ok(rows.iter().map(|(s, sig)| format!("{}|{}", s, sig)).collect::<Vec<_>>().join("\n"))
    }

    pub fn connect(ssid: &str, password: &str) -> Result<bool, String> {
        if ssid.is_empty() {
            return Err("WIFI_CONNECT: SSID darf nicht leer sein".into());
        }
        let (rc, out) = if password.is_empty() {
            run_nmcli(&["dev", "wifi", "connect", ssid])?
        } else {
            run_nmcli(&["dev", "wifi", "connect", ssid, "password", password])?
        };
        if rc != 0 {
            return Err(format!("WIFI_CONNECT: {}", out.trim()));
        }
        Ok(true)
    }

    pub fn disconnect() -> Result<bool, String> {
        // "con down" auf jede aktive WLAN-Connection-ID statt "dev disconnect
        // <iface>" -- so muss der WLAN-Interface-Name (wlan0/wlp3s0/...) nicht
        // selbst erraten werden.
        let (_, out) = run_nmcli(&["-t", "-f", "NAME,TYPE", "con", "show", "--active"])?;
        let mut any = false;
        for line in out.lines() {
            let f = split_terse(line);
            if f.len() >= 2 && f[1] == "802-11-wireless" {
                let _ = run_nmcli(&["con", "down", "id", &f[0]]);
                any = true;
            }
        }
        Ok(any)
    }

    pub fn profiles() -> Result<String, String> {
        let (_, out) = run_nmcli(&["-t", "-f", "NAME,TYPE", "con", "show"])?;
        let names: Vec<String> = out.lines()
            .filter_map(|line| {
                let f = split_terse(line);
                if f.len() >= 2 && f[1] == "802-11-wireless" { Some(f[0].clone()) } else { None }
            })
            .collect();
        Ok(names.join("\n"))
    }

    pub fn delete_profile(name: &str) -> Result<bool, String> {
        if name.is_empty() {
            return Err("WIFI_DELETE_PROFILE: Name darf nicht leer sein".into());
        }
        Ok(run_nmcli(&["con", "delete", "id", name])?.0 == 0)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::process::Command;
    use std::sync::OnceLock;
    use std::time::Duration;

    const CMD_TIMEOUT: Duration = Duration::from_secs(10);
    /// Privates Apple-Framework-Tool, keine oeffentlich dokumentierte/stabile
    /// API -- siehe scan()-Kommentar.
    const AIRPORT_BIN: &str = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";

    fn run_cmd(bin: &str, args: &[&str]) -> Result<(i32, String), String> {
        let bin_owned = bin.to_string();
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = Command::new(&bin_owned).args(&args_owned).output();
            let _ = tx.send(result);
        });
        match rx.recv_timeout(CMD_TIMEOUT) {
            Ok(Ok(out)) => {
                let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&out.stderr));
                Ok((out.status.code().unwrap_or(-1), text))
            }
            Ok(Err(e)) => Err(format!("WIFI: '{}' nicht ausfuehrbar: {}", bin, e)),
            Err(_) => Err(format!("WIFI: '{}' hat nach {}s nicht geantwortet", bin, CMD_TIMEOUT.as_secs())),
        }
    }

    fn run_networksetup(args: &[&str]) -> Result<(i32, String), String> {
        run_cmd("networksetup", args)
    }

    /// Geraetename des Wi-Fi-Hardware-Ports (z.B. "en0") -- einmalig ermittelt
    /// und gecacht, aendert sich waehrend eines Programmlaufs praktisch nie.
    fn wifi_device() -> Option<String> {
        static DEV: OnceLock<Option<String>> = OnceLock::new();
        DEV.get_or_init(|| {
            let (_, out) = run_networksetup(&["-listallhardwareports"]).ok()?;
            let lines: Vec<&str> = out.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let l = line.trim();
                if l == "Hardware Port: Wi-Fi" || l == "Hardware Port: AirPort" {
                    if let Some(next) = lines.get(i + 1) {
                        if let Some(dev) = next.trim().strip_prefix("Device: ") {
                            return Some(dev.to_string());
                        }
                    }
                }
            }
            None
        }).clone()
    }

    fn device_or_err() -> Result<String, String> {
        wifi_device().ok_or_else(|| "WIFI: kein Wi-Fi-Hardware-Port gefunden".to_string())
    }

    pub fn available() -> bool {
        wifi_device().is_some()
    }

    pub fn current() -> Result<String, String> {
        let dev = device_or_err()?;
        let (_, out) = run_networksetup(&["-getairportnetwork", &dev])?;
        let out = out.trim();
        if let Some(ssid) = out.strip_prefix("Current Wi-Fi Network: ") {
            return Ok(ssid.trim().to_string());
        }
        Ok(String::new())
    }

    /// RSSI (dBm) -> ungefaehre Prozent-Qualitaet -- dieselbe grobe Formel wie
    /// NetworkManager (-50dBm~=100%, -100dBm~=0%). macOS liefert Signalstaerke
    /// nur als dBm, nicht als Prozent wie Windows' netsh.
    fn rssi_to_percent(rssi: i64) -> i64 {
        (2 * (rssi + 100)).clamp(0, 100)
    }

    pub fn signal() -> Result<i64, String> {
        let (_, out) = run_cmd(AIRPORT_BIN, &["-I"])?;
        for line in out.lines() {
            if let Some(v) = line.trim().strip_prefix("agrCtlRSSI:") {
                if let Ok(rssi) = v.trim().parse::<i64>() {
                    return Ok(rssi_to_percent(rssi));
                }
            }
        }
        Ok(-1)
    }

    /// Findet die Startposition eines BSSID-Musters (`xx:xx:xx:xx:xx:xx`) in
    /// einer `airport -s`-Zeile -- trennt die SSID-Spalte (kann selbst
    /// Leerzeichen enthalten) vom Rest der whitespace-getrennten Spalten.
    fn find_bssid_start(line: &str) -> Option<usize> {
        let bytes = line.as_bytes();
        for i in 0..bytes.len() {
            if i + 17 > bytes.len() {
                break;
            }
            let cand = &line[i..i + 17];
            let looks_like_bssid = cand.as_bytes().iter().enumerate()
                .all(|(j, &c)| if j % 3 == 2 { c == b':' } else { c.is_ascii_hexdigit() });
            if looks_like_bssid && (i == 0 || bytes[i - 1] == b' ') {
                return Some(i);
            }
        }
        None
    }

    pub fn scan() -> Result<String, String> {
        // UNGETESTET + am unsichersten von allen drei Plattformen: `airport`
        // liegt in einem privaten Apple-Framework (keine oeffentliche/stabile
        // API), und Apple hat WLAN-Scan per CLI in neueren macOS-Versionen
        // zunehmend eingeschraenkt (teils Standortdienste-Berechtigung fuer
        // den aufrufenden Prozess noetig). Schlaegt es fehl, kommt eine klare
        // Fehlermeldung statt eines stillen leeren Ergebnisses oder Crashs.
        let (rc, out) = run_cmd(AIRPORT_BIN, &["-s"])?;
        if rc != 0 {
            return Err(format!(
                "WIFI_SCAN: 'airport -s' fehlgeschlagen (rc={}) -- auf neueren \
                 macOS-Versionen evtl. Standortdienste-Berechtigung fuer diesen \
                 Prozess noetig, oder das Tool wurde von Apple geaendert/entfernt:\n{}",
                rc, out.trim()));
        }
        let mut rows: Vec<(String, i64)> = Vec::new();
        for line in out.lines().skip(1) {
            // Spaltenformat: "SSID BSSID RSSI CHANNEL HT CC SECURITY"
            // (whitespace-getrennt; SSID kann selbst Leerzeichen enthalten,
            // BSSID ist immer genau 17 Zeichen "xx:xx:xx:xx:xx:xx").
            if let Some(bssid_pos) = find_bssid_start(line) {
                let ssid = line[..bssid_pos].trim().to_string();
                let rest = line[bssid_pos..].trim();
                let rssi = rest.split_whitespace().nth(1).and_then(|s| s.parse::<i64>().ok()).unwrap_or(-100);
                if !ssid.is_empty() {
                    rows.push((ssid, rssi_to_percent(rssi)));
                }
            }
        }
        rows.sort_by_key(|(_, s)| -s);
        Ok(rows.iter().map(|(s, sig)| format!("{}|{}", s, sig)).collect::<Vec<_>>().join("\n"))
    }

    pub fn connect(ssid: &str, password: &str) -> Result<bool, String> {
        if ssid.is_empty() {
            return Err("WIFI_CONNECT: SSID darf nicht leer sein".into());
        }
        let dev = device_or_err()?;
        let (rc, out) = if password.is_empty() {
            run_networksetup(&["-setairportnetwork", &dev, ssid])?
        } else {
            run_networksetup(&["-setairportnetwork", &dev, ssid, password])?
        };
        if rc != 0 {
            return Err(format!("WIFI_CONNECT: {}", out.trim()));
        }
        Ok(true)
    }

    pub fn disconnect() -> Result<bool, String> {
        // networksetup hat keinen echten "trenne WLAN"-Befehl -- etablierter
        // Workaround: Funk kurz aus- und wieder einschalten (deassoziiert von
        // jedem Netz, WLAN bleibt danach an).
        let dev = device_or_err()?;
        run_networksetup(&["-setairportpower", &dev, "off"])?;
        run_networksetup(&["-setairportpower", &dev, "on"])?;
        Ok(true)
    }

    pub fn profiles() -> Result<String, String> {
        let dev = device_or_err()?;
        let (_, out) = run_networksetup(&["-listpreferredwirelessnetworks", &dev])?;
        // Erste Zeile ist ein Header ("Preferred networks on en0:"), danach
        // ein eingerueckter Name pro Zeile.
        Ok(out.lines().skip(1).map(|l| l.trim().to_string()).filter(|l| !l.is_empty())
            .collect::<Vec<_>>().join("\n"))
    }

    pub fn delete_profile(name: &str) -> Result<bool, String> {
        if name.is_empty() {
            return Err("WIFI_DELETE_PROFILE: Name darf nicht leer sein".into());
        }
        let dev = device_or_err()?;
        Ok(run_networksetup(&["-removepreferredwirelessnetwork", &dev, name])?.0 == 0)
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod imp {
    fn unsupported<T>() -> Result<T, String> {
        Err("Modul WIFI: nur unter Windows/Linux/macOS unterstuetzt.".into())
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
