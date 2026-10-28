use crate::config::Config;
use crate::mailer;
use crate::models::{EmailResult, OutgoingMessage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

/// Scheduled jobs live here so they survive a restart
const JOBS_FILE: &str = "scheduled_jobs.json";

/// How often the background task looks for jobs that are due
const TICK: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Cancelled,
    /// The server stopped while this job was sending. Not retried, since some people may already have the email
    Interrupted,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Job {
    pub id: String,
    pub send_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub status: JobStatus,
    pub messages: Vec<OutgoingMessage>,
    pub results: Option<Vec<EmailResult>>,
}

#[derive(Serialize)]
pub struct JobSummary {
    pub id: String,
    pub send_at: DateTime<Utc>,
    pub status: JobStatus,
    pub total_emails: usize,
    pub successful: Option<usize>,
    pub failed: Option<usize>,
}

impl Job {
    pub fn summary(&self) -> JobSummary {
        let successful = self.results.as_ref().map(|r| r.iter().filter(|x| x.success).count());
        JobSummary {
            id: self.id.clone(),
            send_at: self.send_at,
            status: self.status,
            total_emails: self.messages.len(),
            successful,
            failed: successful.map(|s| self.messages.len() - s),
        }
    }
}

pub enum CancelError {
    NotFound,
    NotPending(JobStatus),
}

/// In-memory list of jobs, written to disk on every change
#[derive(Clone)]
pub struct Scheduler {
    jobs: Arc<Mutex<Vec<Job>>>,
    path: PathBuf,
}

impl Scheduler {
    pub fn load() -> Self {
        Self::load_from(JOBS_FILE.into())
    }

    fn load_from(path: PathBuf) -> Self {
        let mut jobs: Vec<Job> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();

        let mut changed = false;
        for job in jobs.iter_mut().filter(|j| j.status == JobStatus::Running) {
            println!("Job {} was interrupted by a restart, marking it interrupted", job.id);
            job.status = JobStatus::Interrupted;
            changed = true;
        }
        let missed = jobs.iter().filter(|j| j.status == JobStatus::Pending && j.send_at <= Utc::now()).count();
        if missed > 0 {
            println!("{} scheduled job(s) came due while the server was off, sending now", missed);
        }
        if changed {
            save(&path, &jobs);
        }
        Self { jobs: Arc::new(Mutex::new(jobs)), path }
    }

    pub fn add(&self, send_at: DateTime<Utc>, messages: Vec<OutgoingMessage>) -> Job {
        let job = Job {
            id: uuid::Uuid::new_v4().to_string(),
            send_at,
            created_at: Utc::now(),
            status: JobStatus::Pending,
            messages,
            results: None,
        };
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push(job.clone());
        save(&self.path, &jobs);
        job
    }

    pub fn list(&self) -> Vec<Job> {
        let mut jobs = self.jobs.lock().unwrap().clone();
        jobs.sort_by_key(|j| j.send_at);
        jobs
    }

    pub fn get(&self, id: &str) -> Option<Job> {
        self.jobs.lock().unwrap().iter().find(|j| j.id == id).cloned()
    }

    /// Only a job that has not started yet can be cancelled
    pub fn cancel(&self, id: &str) -> Result<Job, CancelError> {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs.iter_mut().find(|j| j.id == id).ok_or(CancelError::NotFound)?;
        if job.status != JobStatus::Pending {
            return Err(CancelError::NotPending(job.status));
        }
        job.status = JobStatus::Cancelled;
        let job = job.clone();
        save(&self.path, &jobs);
        Ok(job)
    }

    /// Earliest due job, marked running. One at a time, so a crash only interrupts the job that was actually sending
    fn take_next_due(&self, now: DateTime<Utc>) -> Option<Job> {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs
            .iter_mut()
            .filter(|j| j.status == JobStatus::Pending && j.send_at <= now)
            .min_by_key(|j| j.send_at)?;
        job.status = JobStatus::Running;
        let job = job.clone();
        save(&self.path, &jobs);
        Some(job)
    }

    fn finish(&self, id: &str, results: Vec<EmailResult>) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
            job.status = JobStatus::Done;
            job.results = Some(results);
        }
        save(&self.path, &jobs);
    }
}

/// Written to a temp file and renamed, so a crash can't leave a half-written file
fn save(path: &Path, jobs: &[Job]) {
    let tmp = path.with_extension("json.tmp");
    let written = serde_json::to_string_pretty(jobs)
        .map_err(std::io::Error::from)
        .and_then(|text| std::fs::write(&tmp, text))
        .and_then(|_| std::fs::rename(&tmp, path));
    if let Err(e) = written {
        eprintln!("Could not save {}: {}", path.display(), e);
    }
}

/// Background task that sends each job when its time comes
pub fn spawn(scheduler: Scheduler, cfg: Arc<Config>) {
    tokio::spawn(async move {
        loop {
            while let Some(job) = scheduler.take_next_due(Utc::now()) {
                println!("Sending scheduled job {} ({} emails)", job.id, job.messages.len());
                let results = mailer::send_batch(&cfg, &job.messages).await;
                scheduler.finish(&job.id, results);
            }
            tokio::time::sleep(TICK).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as Span;

    fn temp_path() -> PathBuf {
        std::env::temp_dir().join(format!("jobs-{}.json", uuid::Uuid::new_v4()))
    }

    fn msg(to: &str) -> OutgoingMessage {
        OutgoingMessage { to: to.into(), subject: "s".into(), body: "b".into(), html: None }
    }

    #[test]
    fn takes_only_due_jobs_earliest_first() {
        let s = Scheduler::load_from(temp_path());
        let now = Utc::now();
        let later = s.add(now + Span::hours(1), vec![msg("a@x.com")]);
        let second = s.add(now - Span::minutes(1), vec![msg("b@x.com")]);
        let first = s.add(now - Span::minutes(5), vec![msg("c@x.com")]);

        assert_eq!(s.take_next_due(now).unwrap().id, first.id);
        assert_eq!(s.take_next_due(now).unwrap().id, second.id);
        assert!(s.take_next_due(now).is_none());
        assert_eq!(s.get(&later.id).unwrap().status, JobStatus::Pending);
    }

    #[test]
    fn only_pending_jobs_can_be_cancelled() {
        let s = Scheduler::load_from(temp_path());
        let now = Utc::now();
        let job = s.add(now - Span::minutes(1), vec![msg("a@x.com")]);
        let other = s.add(now + Span::hours(1), vec![msg("b@x.com")]);

        assert!(matches!(s.cancel("nope"), Err(CancelError::NotFound)));
        assert_eq!(s.cancel(&other.id).ok().unwrap().status, JobStatus::Cancelled);
        assert!(matches!(s.cancel(&other.id), Err(CancelError::NotPending(JobStatus::Cancelled))));

        s.take_next_due(now);
        assert!(matches!(s.cancel(&job.id), Err(CancelError::NotPending(JobStatus::Running))));
    }

    #[test]
    fn jobs_survive_a_restart_and_running_ones_become_interrupted() {
        let path = temp_path();
        let s = Scheduler::load_from(path.clone());
        let now = Utc::now();
        let pending = s.add(now + Span::hours(1), vec![msg("a@x.com")]);
        let running = s.add(now - Span::minutes(1), vec![msg("b@x.com")]);
        let done = s.add(now - Span::minutes(2), vec![msg("c@x.com")]);
        s.take_next_due(now);
        s.finish(&done.id, vec![]);
        s.take_next_due(now);

        let reloaded = Scheduler::load_from(path.clone());
        assert_eq!(reloaded.get(&pending.id).unwrap().status, JobStatus::Pending);
        assert_eq!(reloaded.get(&running.id).unwrap().status, JobStatus::Interrupted);
        assert_eq!(reloaded.get(&done.id).unwrap().status, JobStatus::Done);
        let _ = std::fs::remove_file(path);
    }
}
