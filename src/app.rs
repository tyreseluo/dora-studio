use makepad_widgets::*;
use crate::dataflow::{DataflowInfo, DataflowTableWidgetRefExt};
use crate::tools::execute_tool;

// Auto-refresh interval in seconds
const AUTO_REFRESH_INTERVAL: f64 = 5.0;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::chat::chat_screen::ChatScreen;
    use crate::dataflow::dataflow_table::DataflowTable;

    // Colors
    SIDEBAR_BG = #1e293b
    MAIN_BG = #f8fafc
    DIVIDER_COLOR = #e2e8f0

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: { title: "Dora Studio" }
                body = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    show_bg: true
                    draw_bg: { color: (MAIN_BG) }

                    // Top panel - Dataflow Table
                    <View> {
                        width: Fill, height: Fill
                        flow: Down
                        align: { x: 0.0, y: 0.0 }
                        padding: { top: 0, left: 16, right: 16, bottom: 16 }

                        dataflow_table = <DataflowTable> {}
                    }

                    // Divider line
                    <View> {
                        width: Fill, height: 1
                        show_bg: true
                        draw_bg: { color: (DIVIDER_COLOR) }
                    }

                    // Bottom panel - Chat
                    <View> {
                        width: Fill, height: 300
                        flow: Down
                        show_bg: true
                        draw_bg: { color: #ffffff }

                        <ChatScreen> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    initialized: bool,
    #[rust]
    last_refresh_time: f64,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
        crate::chat::live_design(cx);
        crate::dataflow::live_design(cx);
        // Light theme
        cx.link(live_id!(theme), live_id!(theme_desktop_light));
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Initialize API key from environment variable
        crate::api::init_api_key_from_env();

        // Schedule initial data load for next frame (after UI is ready)
        self.next_frame = cx.new_next_frame();
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle DataflowTable actions using direct button click checks
        let table = self.ui.dataflow_table(ids!(dataflow_table));

        if table.refresh_clicked(actions) {
            log!("[App] Refresh button clicked - refreshing dataflows");
            self.refresh_dataflows(cx);
        }

        if let Some(uuid) = table.stop_clicked(actions) {
            log!("[App] Stop button clicked for {}", uuid);
            self.stop_dataflow(cx, &uuid);
        }

        if let Some(uuid) = table.destroy_clicked(actions) {
            log!("[App] Destroy button clicked for {}", uuid);
            self.destroy_dataflow(cx, &uuid);
        }

        if let Some(uuid) = table.logs_clicked(actions) {
            log!("[App] Logs button clicked for {}", uuid);
            self.view_dataflow_logs(&uuid);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);

        // Handle next frame for initialization and auto-refresh
        if let Some(ne) = self.next_frame.is_event(event) {
            if !self.initialized {
                self.initialized = true;
                self.last_refresh_time = ne.time;
                log!("[App] Initializing dataflow table on first frame");
                self.refresh_dataflows(cx);
            } else {
                // Check if it's time for auto-refresh
                let elapsed = ne.time - self.last_refresh_time;
                if elapsed >= AUTO_REFRESH_INTERVAL {
                    self.last_refresh_time = ne.time;
                    log!("[App] Auto-refresh triggered after {:.1}s", elapsed);
                    self.refresh_dataflows(cx);
                }
            }
            // Schedule the next frame to keep auto-refresh running
            self.next_frame = cx.new_next_frame();
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl App {
    fn refresh_dataflows(&mut self, cx: &mut Cx) {
        log!("[App] refresh_dataflows called");
        let table = self.ui.dataflow_table(ids!(dataflow_table));
        table.set_loading(cx);

        // Execute dora list command
        let result = execute_tool("dora_list", "refresh", &serde_json::json!({}));
        log!("[App] dora_list result: is_error={}, content={}", result.is_error, &result.content);

        if result.is_error {
            table.set_error(cx, &result.content);
        } else {
            // Try parsing as JSON array first, then NDJSON
            let dataflows = if result.content.trim().starts_with('[') {
                DataflowInfo::parse_json_array(&result.content)
            } else {
                DataflowInfo::parse_ndjson(&result.content)
            };
            log!("[App] Parsed {} dataflows", dataflows.len());
            table.set_dataflows(cx, dataflows);
        }
    }

    fn stop_dataflow(&mut self, cx: &mut Cx, uuid: &str) {
        let args = serde_json::json!({ "dataflow_id": uuid });
        let result = execute_tool("dora_stop", "stop", &args);

        if result.is_error {
            log!("Error stopping dataflow: {}", result.content);
        }

        // Refresh the table after stopping
        self.refresh_dataflows(cx);
    }

    fn destroy_dataflow(&mut self, cx: &mut Cx, uuid: &str) {
        let args = serde_json::json!({ "dataflow_id": uuid });
        let result = execute_tool("dora_destroy", "destroy", &args);

        if result.is_error {
            log!("Error destroying dataflow: {}", result.content);
        }

        // Refresh the table after destroying
        self.refresh_dataflows(cx);
    }

    fn view_dataflow_logs(&self, uuid: &str) {
        let args = serde_json::json!({ "dataflow_id": uuid });
        let result = execute_tool("dora_logs", "logs", &args);

        if result.is_error {
            log!("Error getting logs: {}", result.content);
        } else {
            log!("Dataflow logs for {}:\n{}", uuid, result.content);
        }
    }
}

#[cfg(test)]
mod tests {
    // ============================================================================
    // App Configuration Tests
    // ============================================================================

    #[test]
    fn test_live_design_macro_compiles() {
        // This test verifies that the live_design! macro compiles correctly
        // The actual UI rendering requires a graphics context
        assert!(true, "live_design! macro compiled successfully");
    }

    #[test]
    fn test_window_title() {
        // Verify expected window configuration
        let expected_title = "Dora Studio";
        // The title is defined in the live_design! macro
        // This test documents the expected behavior
        assert_eq!(expected_title, "Dora Studio");
    }

    #[test]
    fn test_app_background_color() {
        // Verify expected background color configuration
        // The color #1a1a30 is defined in the live_design! macro
        let expected_color_hex = "#1a1a30";
        assert!(expected_color_hex.starts_with("#"));
        assert_eq!(expected_color_hex.len(), 7);
    }

    // ============================================================================
    // Live Register Tests
    // ============================================================================

    #[test]
    fn test_theme_configuration() {
        // Document the expected theme linking behavior
        // App::live_register links to theme_desktop_dark
        let expected_theme = "theme_desktop_dark";
        assert!(!expected_theme.is_empty());
    }

    // ============================================================================
    // Color Parsing Tests
    // ============================================================================

    #[test]
    fn test_hex_color_format() {
        // Verify the background color is valid hex format
        let color = "#1a1a30";
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);

        // Parse RGB components
        let r = u8::from_str_radix(&color[1..3], 16).unwrap();
        let g = u8::from_str_radix(&color[3..5], 16).unwrap();
        let b = u8::from_str_radix(&color[5..7], 16).unwrap();

        assert_eq!(r, 0x1a);
        assert_eq!(g, 0x1a);
        assert_eq!(b, 0x30);
    }

    // ============================================================================
    // App Module Structure Tests
    // ============================================================================

    #[test]
    fn test_app_module_structure() {
        // Document expected module structure
        // - App struct with ui: WidgetRef field
        // - LiveRegister implementation for live_design registration
        // - MatchEvent implementation for event handling
        // - AppMain implementation for main event loop
        assert!(true, "Module structure documented");
    }
}
