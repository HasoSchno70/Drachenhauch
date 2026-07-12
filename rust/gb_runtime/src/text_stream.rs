//! Streaming-UTF-8-Dekodierung fuer byteweise Reads (TCP/Serial/...).
//! Kein Feature-Gate -- wird von mehreren, unabhaengig zuschaltbaren I/O-
//! Modulen (net, serial) genutzt und muss deshalb immer verfuegbar sein.

/// Dekodiert `data` moeglichst als UTF-8: echte Fehlsequenzen werden (wie bei
/// `from_utf8_lossy`) durch U+FFFD ersetzt, aber Byte am ENDE, die noch der
/// unvollstaendige Anfang eines Mehrbyte-Zeichens sein koennten, werden NICHT
/// ersetzt, sondern als `leftover` zurueckgegeben (fuer den naechsten Read).
/// Ohne das wuerde ein Umlaut/Emoji, der genau an einer Lesegrenze
/// zerschnitten wird, als kaputtes Zeichen erscheinen.
pub fn decode_utf8_streaming(mut data: Vec<u8>) -> (String, Vec<u8>) {
    let mut out = String::new();
    let mut consumed = 0usize;
    loop {
        let rest = &data[consumed..];
        if rest.is_empty() {
            break;
        }
        match std::str::from_utf8(rest) {
            Ok(s) => {
                out.push_str(s);
                consumed = data.len();
                break;
            }
            Err(e) => {
                let valid = e.valid_up_to();
                out.push_str(std::str::from_utf8(&rest[..valid]).unwrap());
                match e.error_len() {
                    Some(bad_len) => {
                        // echte Fehlsequenz (kein Fragment) -> Ersatzzeichen, weiterscannen
                        out.push('\u{FFFD}');
                        consumed += valid + bad_len;
                    }
                    None => {
                        // Bytes am Ende koennten noch vervollstaendigt werden -> aufheben
                        consumed += valid;
                        break;
                    }
                }
            }
        }
    }
    let leftover = data.split_off(consumed);
    (out, leftover)
}

#[cfg(test)]
mod tests {
    use super::decode_utf8_streaming;

    #[test]
    fn ascii_passthrough() {
        let (text, leftover) = decode_utf8_streaming(b"hello".to_vec());
        assert_eq!(text, "hello");
        assert!(leftover.is_empty());
    }

    #[test]
    fn multibyte_char_split_across_two_reads_reassembles() {
        let full = "Grosse Hitze: \u{00e4}".as_bytes().to_vec();
        let split_at = full.len() - 1;
        let (first_chunk, second_chunk) = full.split_at(split_at);

        let (text1, leftover) = decode_utf8_streaming(first_chunk.to_vec());
        assert!(!text1.contains('\u{FFFD}'));
        assert!(!leftover.is_empty());

        let mut rest = leftover;
        rest.extend_from_slice(second_chunk);
        let (text2, leftover2) = decode_utf8_streaming(rest);
        assert!(leftover2.is_empty());
        assert_eq!(format!("{}{}", text1, text2), "Grosse Hitze: \u{00e4}");
    }

    #[test]
    fn genuinely_invalid_byte_becomes_replacement_char_not_dropped() {
        let mut data = b"vor".to_vec();
        data.push(0xFF);
        data.extend_from_slice(b"nach");
        let (text, leftover) = decode_utf8_streaming(data);
        assert!(leftover.is_empty());
        assert_eq!(text, "vor\u{FFFD}nach");
    }
}
