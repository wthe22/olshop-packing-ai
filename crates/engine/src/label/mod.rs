//! The label fields a page is read for: the Order ID, the tracking ID, the deadline, the sender
//! and the receiver's area, each in the module beside this file.

mod area;
mod fields;
mod sender;

pub use self::area::find_area;

pub(crate) use self::fields::whitespace_re;
pub use self::fields::{Form, find_order_id, find_ship_by, find_tracking_id, is_instant};
pub use self::sender::{Sender, SenderText, find_sender};
