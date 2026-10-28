mod batch;
mod bulk;
mod health;
mod sent;
mod single;

pub use batch::send_batch_handler;
pub use bulk::send_bulk_email_handler;
pub use health::health_check;
pub use sent::sent_handler;
pub use single::send_email_handler;
