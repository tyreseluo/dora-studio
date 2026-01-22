pub mod dataflow_table;

pub use dataflow_table::*;

use makepad_widgets::*;

pub fn live_design(cx: &mut Cx) {
    dataflow_table::live_design(cx);
}
