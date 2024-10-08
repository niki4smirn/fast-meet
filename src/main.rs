use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use clap::Parser;

const CALENDAR_API_URL: &str = "https://www.googleapis.com/calendar/v3/calendars/primary/events/";

#[derive(Debug, Serialize, Deserialize)]
struct ConferenceSolutionKey {
    #[serde(rename = "type")]
    ty: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRequest {
    request_id: String,
    conference_solution_key: ConferenceSolutionKey,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConferenceData {
    create_request: CreateRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleCalendarEvent {
    start: EventDateTime,
    end: EventDateTime,
    conference_data: ConferenceData,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventDateTime {
    #[serde(rename = "dateTime")]
    date_time: String,
}

fn set_data_path(new_path: PathBuf) {
    let mut data_path = DATA_PATH.lock().unwrap();
    data_path.clear();
    data_path.push(new_path);
}

fn get_data_path() -> PathBuf {
    DATA_PATH.lock().unwrap().clone()
}

use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

async fn auth() -> Result<AccessToken, yup_oauth2::Error> {
    let secret =
        yup_oauth2::read_application_secret(get_data_path().join("credentials.json")).await?;

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(get_data_path().join("tokencache.json"))
        .build()
        .await?;

    let scopes = &["https://www.googleapis.com/auth/calendar"];

    auth.token(scopes).await
}

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "PATH", help = "~/.meet_data/ by default")]
    data: Option<PathBuf>,
}

lazy_static! {
    static ref DATA_PATH: Mutex<PathBuf> = Mutex::new(PathBuf::default());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.data {
        Some(path) => set_data_path(path),
        None => {
            let home_dir = home::home_dir().unwrap();
            set_data_path(home_dir.join(".meet_data/"));
        }
    };

    let start_time = chrono::Utc::now();
    let end_time = start_time + chrono::Duration::hours(1);

    let request_id_cache_path = get_data_path().join("request_id_cache");
    let request_id = std::fs::read_to_string(&request_id_cache_path)?;

    let event = GoogleCalendarEvent {
        start: EventDateTime {
            date_time: start_time.to_rfc3339(),
        },
        end: EventDateTime {
            date_time: end_time.to_rfc3339(),
        },
        conference_data: ConferenceData {
            create_request: CreateRequest {
                request_id: request_id.clone(),
                conference_solution_key: ConferenceSolutionKey {
                    ty: String::from("hangoutsMeet"),
                },
            },
        },
    };

    let auth_token: String = auth().await?.token().unwrap().into();

    let create_resp = create_google_calendar_event(&event, &auth_token).await?;
    println!("Google Meet Link: {}", create_resp.link);

    copy_to_clipboard(&create_resp.link)?;
    println!("Link is copied to the clipboard!");

    let request_id = request_id.trim().parse::<u64>()?;
    fs::write(&request_id_cache_path, (request_id + 1).to_string())?;

    delete_google_calendar_event(&create_resp.id, &auth_token).await?;

    open::that(create_resp.link)?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateEventResponse {
    id: String,
    #[serde(rename = "hangoutLink")]
    link: String,
}

async fn create_google_calendar_event(
    event: &GoogleCalendarEvent,
    access_token: &String,
) -> Result<CreateEventResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let url = Url::parse_with_params(CALENDAR_API_URL, &[("conferenceDataVersion", "1")])?;

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .json(event)
        .send()
        .await?;

    if !response.status().is_success() {
        println!("{}", response.status());
        panic!()
    }

    Ok(response.json().await?)
}

async fn delete_google_calendar_event(
    id: &String,
    access_token: &String,
) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    let base = Url::parse(CALENDAR_API_URL)?;
    let url = base.join(id)?;

    client
        .delete(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    Ok(())
}

fn copy_to_clipboard(s: &String) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(s.as_bytes())?;
    }

    child.wait()?;
    Ok(())
}
