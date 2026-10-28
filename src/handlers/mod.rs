mod batch;
mod bulk;
mod health;
mod schedule;
mod sent;
mod single;
mod validate;

pub use batch::send_batch_handler;
pub use bulk::send_bulk_email_handler;
pub use health::health_check;
pub use schedule::{cancel_scheduled_handler, get_scheduled_handler, list_scheduled_handler, schedule_batch_handler};
pub use sent::sent_handler;
pub use single::send_email_handler;
pub use validate::{validate_email_handler, validate_emails_handler};
