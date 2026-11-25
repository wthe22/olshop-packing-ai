//! The label fields a page is read for: the Order ID, the tracking ID, the deadline and the
//! sender, each in the module beside this file.

mod fields;
mod sender;

pub(crate) use self::fields::whitespace_re;
pub use self::fields::{Form, find_order_id, find_ship_by, find_tracking_id, is_instant};
pub use self::sender::{Sender, SenderText, find_sender};
