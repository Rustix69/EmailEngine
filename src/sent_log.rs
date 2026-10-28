use std::collections::HashSet;
use std::io::Write;

/// One line per successful send, from any endpoint: `address<TAB>unix time`. Old lines with just an address still work. Survives restarts and cleared browsers.
const SENT_LOG: &str = "sent_log.txt";

pub fn load_sent() -> HashSet<String> {
    // First word of each line is the address; anything after it (timestamp, note) is ignored
    std::fs::read_to_string(SENT_LOG)
        .map(|t| t.lines().filter_map(|l| l.split_whitespace().next()).map(|a| a.to_lowercase()).collect())
        .unwrap_or_default()
}

pub fn log_sent(address: &str) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // One write per line, so concurrent sends (/send-bulk-email) can't interleave
    let line = format!("{}\t{}\n", address.to_lowercase(), now);
    let written = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(SENT_LOG)
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = written {
        eprintln!("Could not record {} in {}: {}", address, SENT_LOG, e);
    }
}
