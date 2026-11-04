//! Only an explicit Send submits the message. No saved servers, accounts,
//! search text or diagnostics are attached. The form key is public and grants
//! submission access only; never embed a mail password or management token.
use super::*;
use std::{io::Read, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};

const FORM_KEY: &str = "";
const ENDPOINT: &str = "https://api.staticforms.dev/submit";
static BUSY: AtomicBool = AtomicBool::new(false);
type Reply = Arc<Mutex<Option<Result<(), Status>>>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Status {
    #[default]
    Idle,
    Sending,
    Accepted,
    Failed,
    Limited,
    NotConfigured,
}
impl Status {
    pub fn key(self) -> &'static str {
        match self {
            Self::Idle => "feedback_privacy",
            Self::Sending => "feedback_sending",
            Self::Accepted => "feedback_accepted",
            Self::Failed => "feedback_failed",
            Self::Limited => "feedback_limited",
            Self::NotConfigured => "feedback_not_configured",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct Sender {
    pub status: Status,
    reply: Reply,
    draft: Option<(usize,String)>,
}
impl Sender {
    pub fn configured() -> bool { !FORM_KEY.is_empty() }
    pub fn edited(&mut self, category:usize, text:&str) {
        let text=text.trim();
        if self.status==Status::Sending {return;}
        if self.draft.as_ref().is_none_or(|(c,t)| *c!=category || t!=text) {
            self.draft=Some((category,text.into()));
            self.status=Status::Idle;
        }
    }
    pub fn poll(&mut self) {
        if let Some(result)=self.reply.lock().unwrap_or_else(|e| e.into_inner()).take() {
            self.status=result.err().unwrap_or(Status::Accepted);
        }
    }
    pub fn send(&mut self, home:&Path, category:usize, text:&str) {
        self.start(home, category, text, FORM_KEY, core::unix_seconds(), transmit);
    }
    fn start<F>(&mut self, home:&Path, category:usize, text:&str, key:&str, now:u64, transport:F)
    where F: FnOnce(Vec<u8>)->Result<(),Status> + Send + 'static {
        self.poll();
        if self.status==Status::Sending { return; }
        if key.is_empty() { self.status=Status::NotConfigured; return; }
        let Ok(body)=payload(key,category,text) else { self.status=Status::Failed; return; };
        let rate=home.join("feedback-last-attempt.json");
        if persistence::read_bounded(&rate).ok()
            .and_then(|bytes| serde_json::from_slice::<u64>(&bytes).ok())
            .is_some_and(|last| now < last.saturating_add(60)) {
            self.status=Status::Limited; return;
        }
        if BUSY.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err() {
            self.status=Status::Limited; return;
        }
        struct Job;
        impl Drop for Job { fn drop(&mut self) { BUSY.store(false,Ordering::Release); } }
        let job=Job;
        if persistence::atomic_write(&rate,now.to_string().as_bytes()).is_err() {
            self.status=Status::Failed; return;
        }
        self.status=Status::Sending;
        self.draft=Some((category,text.into()));
        let reply=self.reply.clone();
        if std::thread::Builder::new().name("browser-feedback".into()).spawn(move || {
            let _job=job;
            let result=transport(body);
            *reply.lock().unwrap_or_else(|e| e.into_inner())=Some(result);
        }).is_err() { self.status=Status::Failed; }
    }
}

fn payload(key:&str, category:usize, text:&str)->Result<Vec<u8>,Status> {
    let text=text.trim();
    if text.is_empty() || text.chars().count()>500 || category>3 { return Err(Status::Failed); }
    let category=["Idea","Bug","Public server suggestion","General"][category];
    serde_json::to_vec(&serde_json::json!({
        "apiKey":key,
        "name":"Server Atlas",
        "category":category,
        "mod_version":"2.2.0",
        "message":text,
        "honeypot":""
    })).map_err(|_|Status::Failed)
}
fn acknowledgement(bytes:&[u8])->Result<(),Status> {
    if bytes.len()>16_384 { return Err(Status::Failed); }
    let json:serde_json::Value=serde_json::from_slice(bytes).map_err(|_|Status::Failed)?;
    if json["success"].as_bool()==Some(true) { Ok(()) } else { Err(Status::Failed) }
}
fn transmit(body:Vec<u8>)->Result<(),Status> {
    let agent=ureq::AgentBuilder::new().redirects(0).timeout(Duration::from_secs(10)).build();
    let response=agent.post(ENDPOINT).set("Content-Type","application/json")
        .set("Accept","application/json").set("User-Agent","Server-Atlas/2.2.0")
        .send_bytes(&body).map_err(|e| if matches!(e,ureq::Error::Status(429,_)) {Status::Limited} else {Status::Failed})?;
    if response.status()!=200 { return Err(Status::Failed); }
    let mut bytes=Vec::new();
    response.into_reader().take(16_385).read_to_end(&mut bytes).map_err(|_|Status::Failed)?;
    // An API acknowledgement means received by the service, not proof of
    // delivery to Outlook. No automatic retries after an ambiguous timeout.
    acknowledgement(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn submission_contains_only_explicit_message_and_checks_real_acknowledgement() {
        let value:serde_json::Value=serde_json::from_slice(&payload("public-form-key",2,"  mc.example.org | Survival  ").unwrap()).unwrap();
        assert_eq!(value["message"],"mc.example.org | Survival");
        assert_eq!(value["category"],"Public server suggestion");
        assert_eq!(value.as_object().unwrap().len(),6);
        for field in ["email","recipient","uuid","servers","diagnostics","search"] { assert!(value.get(field).is_none()); }
        assert!(payload("key",0,"").is_err());
        assert!(payload("key",0,&"x".repeat(501)).is_err());
        assert!(acknowledgement(br#"{"success":true}"#).is_ok());
        for bytes in [br#"{"success":false}"#.as_slice(),br#"{"success":"true"}"#,b"<html>OK</html>"] {
            assert_eq!(acknowledgement(bytes),Err(Status::Failed));
        }
    }
    #[test]
    fn failure_keeps_a_persisted_cooldown_and_reopening_cannot_resend() {
        let home=std::env::temp_dir().join(format!("browser-mail-{}",std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        let mut sender=Sender::default();
        sender.start(&home,0,"Test draft","public-key",100,|_|Err(Status::Failed));
        let deadline=Instant::now()+Duration::from_secs(3);
        while sender.status==Status::Sending && Instant::now()<deadline {
            std::thread::sleep(Duration::from_millis(5)); sender.poll();
        }
        assert_eq!(sender.status,Status::Failed);
        let mut reopened=Sender::default();
        reopened.start(&home,0,"Test draft","public-key",101,|_|panic!("cooldown must prevent transport"));
        assert_eq!(reopened.status,Status::Limited);
        std::fs::remove_dir_all(home).unwrap();
    }
}
