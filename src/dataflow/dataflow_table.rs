use makepad_widgets::*;
use std::cell::RefMut;
use serde::Deserialize;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // Colors
    HEADER_BG = #e5e7eb
    ROW_BG = #ffffff
    ROW_ALT_BG = #f9fafb
    STATUS_RUNNING = #22c55e
    STATUS_FINISHED = #3b82f6
    STATUS_FAILED = #ef4444

    // Table header row
    TableHeader = <View> {
        width: Fill, height: 36
        flow: Right
        show_bg: true
        draw_bg: { color: (HEADER_BG) }
        padding: { left: 8, right: 8 }
        align: { y: 0.5 }

        <Label> {
            width: 180, height: Fit
            draw_text: { color: #374151, text_style: { font_size: 12.0 } }
            text: "Name"
        }
        <Label> {
            width: 80, height: Fit
            draw_text: { color: #374151, text_style: { font_size: 12.0 } }
            text: "Status"
        }
        <Label> {
            width: 60, height: Fit
            draw_text: { color: #374151, text_style: { font_size: 12.0 } }
            text: "Nodes"
        }
        <Label> {
            width: 80, height: Fit
            draw_text: { color: #374151, text_style: { font_size: 12.0 } }
            text: "CPU"
        }
        <Label> {
            width: 100, height: Fit
            draw_text: { color: #374151, text_style: { font_size: 12.0 } }
            text: "Memory"
        }
    }

    // Table data row
    TableRow = <View> {
        width: Fill, height: 32
        flow: Right
        show_bg: true
        draw_bg: { color: (ROW_BG) }
        padding: { left: 8, right: 8 }
        align: { y: 0.5 }

        name_label = <Label> {
            width: 180, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        status_label = <Label> {
            width: 80, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        nodes_label = <Label> {
            width: 60, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        cpu_label = <Label> {
            width: 80, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        memory_label = <Label> {
            width: 100, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
    }

    // Alternate row with different background
    TableRowAlt = <View> {
        width: Fill, height: 32
        flow: Right
        show_bg: true
        draw_bg: { color: (ROW_ALT_BG) }
        padding: { left: 8, right: 8 }
        align: { y: 0.5 }

        name_label = <Label> {
            width: 180, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        status_label = <Label> {
            width: 80, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        nodes_label = <Label> {
            width: 60, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        cpu_label = <Label> {
            width: 80, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
        memory_label = <Label> {
            width: 100, height: Fit
            draw_text: { color: #1f2937, text_style: { font_size: 12.0 } }
        }
    }

    pub DataflowTable = {{DataflowTable}} {
        width: Fill, height: Fit
        flow: Down

        // Header
        <TableHeader> {}

        // Data rows via PortalList
        table_list = <PortalList> {
            width: Fill, height: 200
            flow: Down

            TableRow = <TableRow> {}
            TableRowAlt = <TableRowAlt> {}
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataflowInfo {
    pub uuid: String,
    pub name: String,
    pub status: String,
    pub nodes: u32,
    pub cpu: f64,
    pub memory: f64,
}

impl DataflowInfo {
    /// Parse NDJSON (newline-delimited JSON) into a vector of DataflowInfo
    pub fn parse_ndjson(input: &str) -> Vec<Self> {
        input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    /// Format memory in human-readable format
    pub fn memory_formatted(&self) -> String {
        if self.memory < 0.001 {
            "0 B".to_string()
        } else if self.memory < 1.0 {
            format!("{:.0} MB", self.memory * 1024.0)
        } else {
            format!("{:.2} GB", self.memory)
        }
    }

    /// Format CPU percentage
    pub fn cpu_formatted(&self) -> String {
        format!("{:.1}%", self.cpu)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DataflowTable {
    #[deref] view: View,
    #[rust] dataflows: Vec<DataflowInfo>,
}

impl Widget for DataflowTable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                self.draw_rows(cx, &mut list);
            }
        }
        DrawStep::done()
    }
}

impl DataflowTable {
    /// Set the dataflows to display
    pub fn set_dataflows(&mut self, cx: &mut Cx, dataflows: Vec<DataflowInfo>) {
        self.dataflows = dataflows;
        self.redraw(cx);
    }

    /// Parse and set dataflows from NDJSON string
    pub fn set_from_ndjson(&mut self, cx: &mut Cx, ndjson: &str) {
        self.dataflows = DataflowInfo::parse_ndjson(ndjson);
        self.redraw(cx);
    }

    fn draw_rows(&mut self, cx: &mut Cx2d, list: &mut RefMut<PortalList>) {
        list.set_item_range(cx, 0, self.dataflows.len());

        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < self.dataflows.len() {
                let df = &self.dataflows[item_id];

                // Alternate row colors
                let template = if item_id % 2 == 0 {
                    live_id!(TableRow)
                } else {
                    live_id!(TableRowAlt)
                };

                let item = list.item(cx, item_id, template);

                item.label(id!(name_label)).set_text(cx, &df.name);
                item.label(id!(status_label)).set_text(cx, &df.status);
                item.label(id!(nodes_label)).set_text(cx, &df.nodes.to_string());
                item.label(id!(cpu_label)).set_text(cx, &df.cpu_formatted());
                item.label(id!(memory_label)).set_text(cx, &df.memory_formatted());

                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ndjson() {
        let input = r#"{"uuid":"abc","name":"test","status":"Running","nodes":3,"cpu":0.5,"memory":0.036}
{"uuid":"def","name":"test2","status":"Failed","nodes":0,"cpu":0.0,"memory":0.0}"#;

        let dataflows = DataflowInfo::parse_ndjson(input);
        assert_eq!(dataflows.len(), 2);
        assert_eq!(dataflows[0].name, "test");
        assert_eq!(dataflows[0].status, "Running");
        assert_eq!(dataflows[1].status, "Failed");
    }

    #[test]
    fn test_memory_formatted() {
        let df = DataflowInfo {
            uuid: "test".to_string(),
            name: "test".to_string(),
            status: "Running".to_string(),
            nodes: 1,
            cpu: 0.0,
            memory: 0.036,
        };
        assert_eq!(df.memory_formatted(), "37 MB");

        let df2 = DataflowInfo {
            memory: 1.5,
            ..df.clone()
        };
        assert_eq!(df2.memory_formatted(), "1.50 GB");
    }
}
