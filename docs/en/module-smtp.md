# Module `smtp`

Sending e-mail.

```basic
IMPORT "smtp"
```

The counterpart to [`xlsx`](module-xlsx.md) and [`pdf`](module-pdf.md): there
the report is produced, here it goes out. Together they are the two halves of
the chain most often asked for in office programs — *build a report and send
it out*.

## A complete example

```basic
IMPORT "smtp"

DIM m AS SMTP
m = SMTP_NEW()

SMTP_SERVER(m, "smtp.beispiel.de", 587)          ' 587 -> STARTTLS
SMTP_LOGIN(m, "hans@beispiel.de", "geheim")

SMTP_FROM(m, "hans@beispiel.de", "Abteilung Zahlen")
SMTP_TO(m, "chefin@beispiel.de")
SMTP_SUBJECT(m, "Auswertung August")
SMTP_TEXT(m, "Guten Tag," + CHR$(10) + CHR$(10) + "anbei die Zahlen.")
SMTP_ATTACH(m, "auswertung.xlsx")

SMTP_SEND(m)
SMTP_CLOSE(m)
```

## Overview

| Function | Purpose |
|---|---|
| `SMTP_NEW()` → SMTP | new message |
| `SMTP_SERVER(m, host$, port [, sicherheit$])` | where it is submitted |
| `SMTP_LOGIN(m, benutzer$, kennwort$)` | login (omit = without) |
| `SMTP_FROM(m, adresse$ [, name$])` | sender |
| `SMTP_TO(m, adresse$ [, name$])` | recipient — can be called several times |
| `SMTP_CC(m, adresse$ [, name$])` | copy recipient (CC) |
| `SMTP_BCC(m, adresse$)` | blind copy |
| `SMTP_SUBJECT(m, betreff$)` | set the subject line |
| `SMTP_TEXT(m, text$)` | message as plain text |
| `SMTP_HTML(m, html$)` | message as HTML |
| `SMTP_ATTACH(m, pfad$ [, name$])` | attach a file — can be called several times |
| `SMTP_TIMEOUT(m, ms)` | time limit per step (default 30 000) |
| `SMTP_MESSAGE$(m)` → STRING | the finished message, **without** sending |
| `SMTP_SEND(m)` | send it |
| `SMTP_CLOSE(m)` | free the memory |

**`SMTP_TEXT` and `SMTP_HTML` do not exclude each other.** If you set both,
both versions are sent (`multipart/alternative`); the reader shows the one it
can. If you set only one, you get a single-part message — nothing is nested
artificially.

## Encryption

The third argument of `SMTP_SERVER` is the security:

| Value | Meaning | usual port |
|---|---|---|
| `"starttls"` | connect in plain text first, then upgrade | 587 |
| `"tls"` | encrypted from the very first second | 465 |
| `"keine"` | not at all (none) | 25 |

**Without it, the port decides**: 465 → `tls`, 25 → `keine`, everything else
→ `starttls`. That covers the normal case (submission via 587) and can be
overridden explicitly at any time.

**A password never goes onto the network unencrypted.** `SMTP_LOGIN` together
with `"keine"` is an error — unless the server is `localhost`, because a relay
on the same machine without TLS is the normal case. This rule comes from the
module itself, not from the server: otherwise a mistyped port would silently
give the password away.

Login uses `AUTH PLAIN`, otherwise `AUTH LOGIN` — the two methods every server
supports. OAuth2 (`XOAUTH2`) is **not** available; providers that require it
need an *app password* instead.

## What happens behind the scenes

* **Umlauts in the subject and in the name** are encoded per RFC 2047
  (`=?UTF-8?B?…?=`), otherwise only ASCII would be allowed there. Long runs of
  umlauts are split along the **characters**, not the bytes.
* **The body is base64**, even plain text. SMTP has two traps — lines longer
  than 998 characters and lines that start with a dot — and base64 closes both
  at once.
* **A line break in the subject, name or address is an error.** It would be an
  additional header line; `"Rechnung\r\nBcc: fremd@example.com"` would
  otherwise send a silent copy. Removing it silently would be the worse
  answer: then a piece of the subject disappears without anyone noticing.
* **The blind copy is only in the envelope**, not in the headers — otherwise
  it would not be one.
* **The type of an attachment** comes from the file extension, from the same
  table that the module [`httpd`](module-httpd.md) uses.

`SMTP_MESSAGE$` returns exactly the characters that would otherwise go over
the wire. That is the way to check a message without having a mail server —
and the way to write it to a file instead.

## Limits

* **Sending only.** Reading mailboxes (POP3/IMAP) is not part of it.
* **No OAuth2**, see above.
* **No time zone**: the `Date` header is in UTC (`+0000`). Without a time zone
  database the machine's offset could not be named, and a wrong statement
  would be worse than an honest one.
* **An attachment is held completely in memory** (and base64-encoded another
  third larger). For reports and analyses that is no issue, for a video
  archive it is.
* **`smtp` needs the feature `smtp`** (included in the standard build). A
  build without it says so on the call.

## In the native runtime (dhrt)

`rust/drachenhauch_runtime/src/smtp.rs`, feature `smtp`. TLS via `rustls`
with `ring` — both were already in the dependency tree through `ureq` (module
`html`), so the module costs nothing new.

Building the message (`nachricht`) is separate from transmitting it
(`senden`); that is why there is `SMTP_MESSAGE$`, and why everything except
the network path can be checked without a mail server. Replies are read
**unbuffered**: a buffer could carry data from the unencrypted period across a
`STARTTLS`, and exactly that is what a known vulnerability consists of.

Example: [examples/178_bericht_verschicken.dh](../../examples/178_bericht_verschicken.dh)
— builds the report with `xlsx` and sends it off.
