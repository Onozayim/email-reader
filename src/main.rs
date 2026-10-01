extern crate imap;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    env,
    fs::OpenOptions,
    io::{Read, Seek, Write},
    println,
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Email {
    subject: String,
    body: String,
    from_email: String,
    received_at: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("HELLO!!");
    dotenv::dotenv().ok();

    let e = fetch_inbox_top().await;

    let e = match e {
        Ok(message) => message,
        Err(e) => {
            storing_error(e.to_string());
            return;
        }
    };

    match e {
        Some(_e) => println!("*****"),
        None => {
            println!("No hay correos nuevos");
            return;
        }
    };
}

fn storing_error(message: String) {
    //Sotring error in file later later
    let mut err_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("error.txt")
        .unwrap();

    writeln!(err_file, "{}", message).unwrap();
}

fn get_old_number(num: u32) -> anyhow::Result<Option<u32>> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("index.txt")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let old_number = if contents.is_empty() {
        0
    } else {
        contents.trim().parse::<u32>()?
    };

    if old_number == num {
        return Ok(None);
    }

    file.set_len(0)?;
    file.seek(std::io::SeekFrom::Start(0))?;

    write!(file, "{}", num)?;

    return Ok(Some(old_number));
}

async fn fetch_inbox_top() -> anyhow::Result<Option<String>> {
    let domain_string = env::var("SMTP_SERVER")?;
    let domain = domain_string.as_str();
    let tls = native_tls::TlsConnector::builder().build()?;
    let client = imap::connect((domain, 993), domain, &tls)?;

    let mut imap_session = client
        .login(&env::var("EMAIL")?, &env::var("PASS")?)
        .map_err(|e| e.0)?;

    let select = imap_session.select("INBOX")?;

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("index.txt")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let num = select.exists;
    let old_number = get_old_number(num)?;

    let mut old_number = match old_number {
        Some(e) => e,
        None => return Ok(None),
    };

    let mut total = 0;

    while old_number < num {
        let temp: u32 = old_number + 1;
        old_number = if old_number + 10 <= num {
            old_number + 10
        } else {
            num
        };

        let messages =
            imap_session.fetch(format!("{}:{}", temp, old_number), "(INTERNALDATE RFC822)")?;

        if messages.is_empty() {
            return Ok(None);
        }

        let mut messages_to_send: Vec<Email> = Vec::new();

        let mut iterator = 0;
        for m in &messages {
            let body = match m.body() {
                Some(bytes) => bytes,
                None => {
                    storing_error("Se encontro un correo sin cuerpo".to_string());
                    continue;
                }
            };

            let body = match std::str::from_utf8(body) {
                Ok(s) => s,
                Err(e) => {
                    storing_error(e.to_string());
                    continue;
                }
            };

            let subject_re = Regex::new(r"(?m)^Subject:\s*(.+)$")?;
            let subject: &str = subject_re
                .captures(&body)
                .and_then(|caps| caps.get(1))
                .map(|m: regex::Match<'_>| m.as_str())
                .unwrap_or("Subject not found");

            let from_re = Regex::new(r"(?m)^From:\s*(.+)$")?;
            let from: &str = from_re
                .captures(&body)
                .and_then(|caps| caps.get(1))
                .map(|m: regex::Match<'_>| m.as_str())
                .unwrap_or("From not found");

            // println!("From");
            // println!("{}", from);

            let boundary_re = Regex::new(r#"boundary="([^"]+)""#)?;
            let boundary = boundary_re
                .captures(&body)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str())
                .unwrap_or("Boundary not found");

            // println!("---- boundary ----");
            // println!("{}", boundary);
            // println!("---- body -----");
            // println!("{}", body);
            // // Split body parts
            let parts: Vec<&str> = body.split(&format!("--{}", boundary)).collect();

            let mut plain_text_body: Option<&str> = None;
            let mut html_body: Option<&str> = None;
            for part in &parts {
                let part_lower = part.to_lowercase();
                // println!("----------------part-----------------");
                // println!("{}", part);

                if let Some(body_start) = part.find("\r\n\r\n").or_else(|| part.find("\n\n")) {
                    let body = part[body_start..].trim();

                    if part_lower.contains("content-type: text/plain") {
                        plain_text_body = Some(body);
                    } else if part_lower.contains("content-type: text/html") {
                        html_body = Some(body);
                    }
                }
            }

            let body = plain_text_body.or(html_body).unwrap_or("Body not found");

            // Output
            // Change for post request

            //THIS IS ONLY FOR TESTING PURPOSES
            // let multiple_senders = [
            //     "asmith@example.net",
            //     "smithsteven@example.org",
            //     "cramirez@example.org",
            //     "ksmith@example.com",
            //     "hgarcia@example.com",
            //     "adamanderson@example.org",
            //     "ujones@example.org",
            //     "adamsrobert@example.net",
            //     "hbrown@example.org",
            //     "reyesrobert@example.com",
            //     "amatthews@example.org",
            //     "tiffany23@example.org",
            //     "jjohnson@example.org",
            //     "xfox@example.org",
            //     "xharris@example.net",
            //     "william74@example.net",
            //     "bjohnson@example.org",
            //     "paulmartinez@example.org",
            //     "ojones@example.net",
            //     "brian49@example.com",
            //     "nphillips@example.com",
            //     "davidwilliams@example.org",
            //     "qsmith@example.org",
            //     "tjohnson@example.net",
            //     "thompsonmichael@example.net",
            //     "allison30@example.org",
            //     "martinjulia@example.com",
            //     "prhodes@example.net",
            //     "ramirezpatrick@example.net",
            //     "ewashington@example.net",
            //     "jonesjessica@example.com",
            //     "bjones@example.net",
            //     "john23@example.net",
            //     "rosalesmichael@example.org",
            //     "kjones@example.net",
            //     "psanchez@example.org",
            //     "ewilkins@example.com",
            //     "iclark@example.com",
            //     "ksmith@example.org",
            //     "smithkenneth@example.net",
            //     "patriciamartin@example.org",
            //     "susan74@example.org",
            //     "amber05@example.com",
            //     "smitheric@example.net",
            //     "ralexander@example.org",
            //     "hmartinez@example.org",
            //     "hallmichael@example.org",
            //     "tonya59@example.org",
            //     "ymartinez@example.com",
            //     "wjohnson@example.com",
            //     "matthew09@example.org",
            //     "ltaylor@example.com",
            //     "michelle58@example.com",
            //     "kolsen@example.net",
            //     "robert60@example.com",
            //     "william48@example.com",
            //     "williamhernandez@example.org",
            //     "hjohnson@example.com",
            //     "xgomez@example.net",
            //     "ulong@example.net",
            //     "sjones@example.com",
            //     "jbrown@example.com",
            //     "lwilliams@example.org",
            //     "steven91@example.net",
            //     "david63@example.org",
            //     "twilliams@example.org",
            //     "sandra63@example.net",
            //     "robertgarcia@example.net",
            //     "ismith@example.net",
            //     "danielsmith@example.net",
            //     "kelly63@example.net",
            //     "ynguyen@example.com",
            //     "eshaw@example.net",
            //     "cfox@example.org",
            //     "zporter@example.org",
            //     "rebecca74@example.net",
            //     "tracy42@example.net",
            //     "trogers@example.com",
            //     "michael21@example.org",
            //     "johnsonamy@example.net",
            //     "mjones@example.com",
            //     "justin72@example.org",
            //     "jsanchez@example.com",
            //     "wmartin@example.com",
            //     "fmoore@example.net",
            //     "staylor@example.net",
            //     "amywilliams@example.net",
            //     "brownchristopher@example.net",
            //     "eric47@example.net",
            //     "brandonwilliams@example.org",
            //     "hmoore@example.org",
            //     "matthew93@example.com",
            //     "fvance@example.net",
            //     "vjones@example.com",
            //     "yortiz@example.org",
            //     "youngnicole@example.net",
            //     "bvasquez@example.org",
            //     "ywilliams@example.com",
            //     "thoward@example.net",
            //     "vgilbert@example.com",
            //     "obrown@example.org",
            //     "linda09@example.net",
            //     "james71@example.org",
            //     "juliehernandez@example.com",
            //     "jsmith@example.net",
            //     "bhunter@example.net",
            //     "oellis@example.net",
            //     "kristinsellers@example.net",
            //     "brenda25@example.org",
            //     "joseph76@example.net",
            //     "wanderson@example.net",
            //     "aporter@example.com",
            //     "fbrewer@example.com",
            //     "donald39@example.net",
            //     "arobinson@example.org",
            //     "tsmith@example.com",
            //     "zsmith@example.com",
            //     "brownjoshua@example.com",
            //     "vgonzalez@example.org",
            //     "jeffrey71@example.com",
            //     "david38@example.org",
            //     "susan86@example.org",
            //     "michael18@example.org",
            //     "iwilson@example.net",
            //     "jeremy10@example.net",
            //     "enunez@example.org",
            //     "john38@example.org",
            //     "michaelwilliams@example.org",
            //     "oedwards@example.net",
            //     "rsmith@example.org",
            //     "heather64@example.com",
            //     "williamsjames@example.net",
            //     "swilliams@example.com",
            //     "michaelhill@example.net",
            //     "ubrooks@example.org",
            //     "patriciasmith@example.org",
            //     "christopher78@example.com",
            //     "michael04@example.org",
            //     "thomaskimberly@example.org",
            //     "jessica82@example.net",
            //     "icruz@example.org",
            //     "rgarcia@example.com",
            //     "bmoore@example.org",
            //     "ajohnson@example.com",
            //     "odavis@example.com",
            //     "andrea10@example.org",
            //     "sandrajones@example.net",
            //     "xmeyer@example.com",
            //     "denise74@example.org",
            //     "imartinez@example.com",
            //     "zlewis@example.org",
            //     "mharrison@example.com",
            //     "abryant@example.org",
            //     "wking@example.com",
            //     "zanderson@example.net",
            //     "lisa57@example.org",
            //     "caindenise@example.com",
            //     "lisa12@example.net",
            //     "psmith@example.org",
            //     "mwilliams@example.com",
            //     "fwright@example.org",
            //     "sanchezmichael@example.org",
            //     "johnsonwilliam@example.com",
            //     "djohnson@example.net",
            //     "elizabeth57@example.com",
            //     "xkim@example.com",
            //     "earmstrong@example.org",
            //     "eric65@example.com",
            //     "qjohnson@example.net",
            //     "wsmith@example.org",
            //     "knunez@example.com",
            //     "justin48@example.net",
            //     "williamsadam@example.com",
            //     "xflores@example.com",
            //     "sbrown@example.net",
            //     "dcarroll@example.com",
            //     "elizabethbell@example.com",
            //     "janet33@example.com",
            //     "jameslawrence@example.org",
            //     "mtaylor@example.org",
            //     "jeffrey23@example.com",
            //     "njohnson@example.net",
            //     "turnerchristopher@example.net",
            //     "monica63@example.net",
            //     "matthewking@example.com",
            // ];


            // println!("|{}|", &from.to_string().as_str().trim());

            // if !multiple_senders.contains(&from.to_string().as_str().trim()) {
            //     continue;
            // }

            // println!("EMAIL SENT");

            total = total + 1;

            messages_to_send.push(Email {
                subject: subject.to_string(),
                body: body.to_string(),
                from_email: from.trim().to_string(),
                received_at: m.internal_date().unwrap().to_rfc3339(),
            });

            iterator = iterator + 1;

            println!("------MESSAGE SENT------");
        }

        //Send messages to post request
        // println!("{}",messages_to_send);
        let client = Client::new();
        let api_url = env::var("API_URL")?;

        let _ = client.post(api_url).json(&messages_to_send).send().await?;
    }

    imap_session.logout()?;

    println!("emails totales {}", total);

    return Ok(Some("Emails leídos".to_string()));
}
