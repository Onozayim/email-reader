extern crate imap;
use regex::Regex;
use std::{
    env,
    fs::OpenOptions,
    io::{Read, Seek, Write},
    process::Command,
};
// #[derive(Debug, PartialEq, Eq)]
// struct Email {
//     subject: String,
//     body: String,
// }
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

async fn connect_sql() -> anyhow::Result<sqlx::MySqlPool> {
    return Ok(sqlx::MySqlPool::connect(&env::var("DATABASE_URL")?).await?);
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
        contents.parse::<u32>()?
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
    let domain: &'static str = "montesdevoca.com";
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

    let pool = connect_sql().await?;

    sqlx::query("TRUNCATE emails").execute(&pool).await?;

    let mut joy_count = 0;
    let mut sadness_count = 0;
    let mut anger_count = 0;
    let mut fear_count = 0;
    let mut surprise_count = 0;
    let mut love_count = 0;

    while old_number < num {
        let temp: u32 = old_number + 1;
        old_number = if old_number + 10 <= num {
            old_number + 10
        } else {
            num
        };

        let messages = imap_session.fetch(format!("{}:{}", temp, old_number), "RFC822")?;

        if messages.is_empty() {
            return Ok(None);
        }

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

            println!("---- boundary ----");
            println!("{}", boundary);
            println!("---- body -----");
            println!("{}", body);
            // Split body parts
            let parts: Vec<&str> = body.split(&format!("--{}", boundary)).collect();

            let mut plain_text_body = "Body not found";
            // let mut html_text_body = "Body not found";
            for part in &parts {

                println!("----------------part-----------------");
                println!("{}", part);

                if part.contains("Content-Type: text/html") {
                    if let Some(body_start) = part.find("\r\n\r\n").or_else(|| part.find("\n\n")) {
                        plain_text_body = part[body_start..].trim();
                        break;
                    }
                }

                // if part.contains("Content-Type: text/html") {
                //     if let Some(body_start) = part.find("\r\n\r\n").or_else(|| part.find("\n\n")) {
                //         html_text_body = part[body_start..].trim();
                //         break;
                //     }
                // }
            }

            // Output

            let output = Command::new("python")
                .arg("main.py")
                // .arg("--name ")
                .arg(format!("'{plain_text_body}'"))
                .output()
                .expect("Command failed");

            let result = String::from_utf8_lossy(&output.stdout).to_string();
            let mut emotion = "";

            if let Some(index) = result.find("emotion_detected:") {
                emotion = &result[index + 17..];
            }

            if output.status.success() {
                println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            } else {
                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            }

            sqlx::query!(
                r#"
            INSERT INTO
                emails (body, subject, emotion, emails.from)
            VALUES
                (?, ?, ?, ?)
            "#,
                plain_text_body,
                subject,
                emotion, 
                from
            )
            .execute(&pool)
            .await?;

            if let Some(_) = emotion.find("joy") {
                joy_count = joy_count + 1;
            }

            if let Some(_) = emotion.find("sadness") {
                sadness_count = sadness_count + 1;
            }

            if let Some(_) = emotion.find("anger") {
                anger_count = anger_count + 1;
            }

            if let Some(_) = emotion.find("fear") {
                fear_count = fear_count + 1;
            }

            if let Some(_) = emotion.find("love") {
                love_count = love_count + 1;
            }

            if let Some(_) = emotion.find("surprise") {
                surprise_count = surprise_count + 1;
            }
            
            println!("------MENSAJE------");
        }
    }

    sqlx::query!(
        r#"
            INSERT INTO
                emails_count (joy, sadness, anger, fear, love, surprise)
            VALUES
                (?, ?, ?, ?, ?, ?)
            "#,
        joy_count,
        sadness_count,
        anger_count,
        fear_count,
        love_count,
        surprise_count
    )
    .execute(&pool)
    .await?;

    imap_session.logout()?;

    return Ok(Some("Emails leídos".to_string()));
}
