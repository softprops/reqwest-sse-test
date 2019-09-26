use reqwest::r#async::{Client, RequestBuilder};
use std::str::from_utf8;

#[derive(Default, Debug)]
struct Event<'a> {
    id: Option<&'a str>,
    data: &'a str,
}

#[derive(Debug)]
struct Next {
    last_event_id: Option<String>,
    err: reqwest::Error,
}

impl Next {
    fn new(
        last_event_id: Option<String>,
        err: reqwest::Error,
    ) -> Self {
        Self { last_event_id, err }
    }
}

async fn stream(req: RequestBuilder) -> Result<(), Next> {
    let mut last_id = None;
    let mut res = req
        .send()
        .await
        .map_err(|e| Next::new(last_id.clone(), e))?;
    while let Some(item) = res
        .chunk()
        .await
        .map_err(|e| Next::new(last_id.clone(), e))?
    {
        if let Ok(utf8) = from_utf8(&item) {
            let e = utf8.lines().fold(Event::default(), |mut e, line| {
                let mut split = line.splitn(2, ": ");
                match split.next() {
                    Some("id") => e.id = split.next(),
                    Some("data") => e.data = split.next().unwrap_or_default().trim(),
                    _ => (),
                }
                e
            });
            println!("{:#?}", e);
            last_id = e.id.map(|id| id.into());
        }
    }
    Ok(())
}

fn request(
    last_event_id: &Option<String>,
    client: Client,
) -> RequestBuilder {
    let builder = client
        .get("https://stream.wikimedia.org/v2/stream/recentchange")
        .header("Accept", "text/event-stream");
    if let Some(id) = last_event_id {
        builder.header("Last-Event-Id", id)
    } else {
        builder
    }
}

#[tokio::main]
async fn main() {
    let client = Client::new();
    let mut id = None;
    while let Err(Next { last_event_id, err }) = stream(request(&id, client.clone())).await {
        id = last_event_id;
        println!("retrying after error {}", err);
    }
}
