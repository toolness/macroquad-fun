use std::{path::Path, borrow::Cow, fs, io::Write};

use rouille::{Response, try_or_400, post_input, input::post::BufferedFile, Request};
use semver::Version;
use uuid::Uuid;

const RECORDINGS_DIR: &'static str = "recordings";

fn nonempty_string(value: &Option<String>) -> &Option<String> {
    if let Some(value_string) = value {
        if value_string.len() > 0 {
            value
        } else {
            &None
        }
    } else {
        value
    }
}

fn parse_uuid_string(value: &Option<String>) -> Result<Option<Uuid>, uuid::Error> {
    if let Some(value_string) = nonempty_string(value) {
        let uuid = Uuid::parse_str(&value_string)?;
        Ok(Some(uuid))
    } else {
        Ok(None)
    }
}

fn is_version_valid(value: &String) -> bool {
    Version::parse(value).is_ok()
}

fn is_tracking_tag_valid(value: &String) -> bool {
    if value.len() > 10 {
        return false;
    }
    for ch in value.chars() {
        if !ch.is_alphanumeric() {
            return false;
        }
    }
    true
}

fn ensure_dir_exists(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        println!("Creating directory: {}", path.display());
        std::fs::create_dir(path)?;
    }
    Ok(())
}

fn process_request(request: &Request) -> Response {
    match request.url().as_str() {
        "/" => Response::text("hi!"),
        "/record" => {
            if request.method() == "POST" {
                // This is prone to abuse in all kinds of ways, but since we don't allow clients to
                // read data that they've submitted, the only thing they can really do is fill up
                // our disk storage.
                let input = try_or_400!(
                    // b = bytes to append to recording
                    // p = previous length of recording, in bytes (0 if it's a new recording)
                    // v = version of client, e.g. "1.0.1"
                    // t = tracking tag of client (optional)
                    // id = UUID of recording (only required if appending bytes to an existing recording)
                    post_input!(request, { b: BufferedFile, v: String, t: Option<String>, p: usize, id: Option<String> })
                );
                let bytes = &input.b.data;
                let prev_len = input.p;
                if prev_len > 0 && input.id.is_none() {
                    return Response::text("Missing ID").with_status_code(400);
                }
                let version = &input.v;
                if !is_version_valid(version) {
                    return Response::text("Invalid version").with_status_code(400);
                }
                let tracking_tag: Cow<str> = if let Some(ref t) = input.t {
                    if !is_tracking_tag_valid(t) {
                        return Response::text("Invalid tracking tag").with_status_code(400);
                    }
                    t.into()
                } else {
                    Cow::Borrowed("anonymous")
                };
                let uuid = try_or_400!(parse_uuid_string(&input.id)).unwrap_or(Uuid::new_v4());
                let filename = format!("{}-{}-{}.bin", tracking_tag, version, uuid);
                let path = Path::new(RECORDINGS_DIR).join(filename.clone());

                if !path.exists() {
                    let Ok(_) = fs::write(&path, vec![]) else {
                        println!("Error creating file: {}", path.display());
                        return Response::text("Internal Server Error").with_status_code(500);
                    };
                    println!("Created {}.", filename);
                }

                let Ok(metadata) = fs::metadata(&path) else {
                    println!("Error getting metadata for file: {}", path.display());
                    return Response::text("Internal Server Error").with_status_code(500);
                };

                if metadata.len() == prev_len as u64 {
                    let Ok(mut file) = fs::OpenOptions::new().append(true).open(&path) else {
                        println!("Error opening file for appending: {}", path.display());
                        return Response::text("Internal Server Error").with_status_code(500);
                    };
                    let Ok(_) = file.write_all(&bytes) else {
                        println!("Error writing to file: {}", path.display());
                        return Response::text("Internal Server Error").with_status_code(500);
                    };
                    println!("Wrote {} bytes to {}.", bytes.len(), filename);
                } else {
                    // Note that if the length of the recording doesn't match the previous length,
                    // we don't error. This is because we support at-least-once delivery, so
                    // multiple requests of the same recording may be sent, and all after the
                    // first will be no-ops.
                    println!("Ignoring write of {} bytes to {} (prev_len mismatch).", bytes.len(), filename);
                }

                Response::text(format!("{}", uuid))
            } else {
                Response::text("Method Not Allowed").with_status_code(405)
            }
        },
        _ => Response::empty_404(),
    }
}

fn main() {
    let addr: &'static str = "0.0.0.0:4001";

    ensure_dir_exists(Path::new(RECORDINGS_DIR)).unwrap();

    println!("Starting HTTP server on {}.", addr);

    rouille::start_server(addr, move |request| {
        rouille::log(request, std::io::stdout(), || {
            let response = process_request(request);
            response.with_additional_header("Access-Control-Allow-Origin", "*")
        })
    });
}
