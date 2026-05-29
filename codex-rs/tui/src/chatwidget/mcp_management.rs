use std::collections::HashMap;

use codex_app_server_protocol::ConfigWriteResponse;
use codex_app_server_protocol::McpAuthStatus;
use codex_app_server_protocol::McpServerStatus;
use codex_app_server_protocol::WriteStatus;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Widget;

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::key_hint;
use crate::key_hint::KeyBindingListExt;
use crate::keymap::ListKeymap;
use crate::keymap::primary_binding;
use crate::render::Insets;
use crate::render::RectExt as _;
use crate::render::renderable::ColumnRenderable;
use crate::render::renderable::Renderable;
use crate::style::user_message_style;

pub(crate) const MCP_MANAGEMENT_VIEW_ID: &str = "mcp-management";

const SEARCH_PLACEHOLDER: &str = "Type to search MCP servers";
const SEARCH_PROMPT_PREFIX: &str = "> ";

#[derive(Clone, Debug, PartialEq, Eq)]
struct McpManagementItem {
    name: String,
    enabled: bool,
    tool_count: usize,
    auth_status: Option<McpAuthStatus>,
}

pub(crate) struct McpManagementView {
    items: Vec<McpManagementItem>,
    filtered_indices: Vec<usize>,
    selected_idx: Option<usize>,
    scroll_top: usize,
    complete: bool,
    search_query: String,
    app_event_tx: AppEventSender,
    keymap: ListKeymap,
    header: Box<dyn Renderable>,
    footer_hint: Line<'static>,
}

impl McpManagementView {
    fn new(
        items: Vec<McpManagementItem>,
        app_event_tx: AppEventSender,
        keymap: ListKeymap,
        initial_selected_idx: Option<usize>,
    ) -> Self {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("MCP Servers".bold()));
        header.push(Line::from(
            "Toggle servers for the next turn. Current connections stay untouched.".dim(),
        ));

        let mut view = Self {
            items,
            filtered_indices: Vec::new(),
            selected_idx: initial_selected_idx,
            scroll_top: 0,
            complete: false,
            search_query: String::new(),
            app_event_tx,
            footer_hint: mcp_management_hint_line(&keymap),
            keymap,
            header: Box::new(header),
        };
        view.apply_filter();
        view
    }

    fn visible_len(&self) -> usize {
        self.filtered_indices.len()
    }

    fn max_visible_rows(len: usize) -> usize {
        crate::bottom_pane::popup_consts::MAX_POPUP_ROWS.min(len.max(1))
    }

    fn selected_actual_idx(&self) -> Option<usize> {
        self.selected_idx
            .and_then(|idx| self.filtered_indices.get(idx).copied())
    }

    fn apply_filter(&mut self) {
        let previously_selected = self.selected_actual_idx();
        let filter = self.search_query.trim().to_lowercase();
        self.filtered_indices = if filter.is_empty() {
            (0..self.items.len()).collect()
        } else {
            self.items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| item.name.to_lowercase().contains(&filter).then_some(idx))
                .collect()
        };

        let len = self.visible_len();
        self.selected_idx = previously_selected
            .and_then(|actual_idx| {
                self.filtered_indices
                    .iter()
                    .position(|idx| *idx == actual_idx)
            })
            .or(self.selected_idx.filter(|idx| *idx < len))
            .or_else(|| (len > 0).then_some(0));
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        let Some(selected) = self.selected_idx else {
            self.scroll_top = 0;
            return;
        };
        let visible = Self::max_visible_rows(self.visible_len());
        if selected < self.scroll_top {
            self.scroll_top = selected;
        } else if selected >= self.scroll_top.saturating_add(visible) {
            self.scroll_top = selected.saturating_add(1).saturating_sub(visible);
        }
    }

    fn move_up(&mut self) {
        let len = self.visible_len();
        if len == 0 {
            self.selected_idx = None;
            return;
        }
        self.selected_idx = Some(match self.selected_idx {
            Some(0) | None => len - 1,
            Some(idx) => idx - 1,
        });
        self.ensure_visible();
    }

    fn move_down(&mut self) {
        let len = self.visible_len();
        if len == 0 {
            self.selected_idx = None;
            return;
        }
        self.selected_idx = Some(match self.selected_idx {
            Some(idx) if idx + 1 < len => idx + 1,
            _ => 0,
        });
        self.ensure_visible();
    }

    fn toggle_selected(&mut self) {
        let Some(actual_idx) = self.selected_actual_idx() else {
            return;
        };
        let Some(item) = self.items.get_mut(actual_idx) else {
            return;
        };
        item.enabled = !item.enabled;
        self.app_event_tx.send(AppEvent::SetMcpServerEnabled {
            server_name: item.name.clone(),
            enabled: item.enabled,
        });
    }

    fn close(&mut self) {
        self.complete = true;
    }

    fn build_rows(&self) -> Vec<Line<'static>> {
        let visible = Self::max_visible_rows(self.visible_len());
        self.filtered_indices
            .iter()
            .enumerate()
            .skip(self.scroll_top)
            .take(visible)
            .filter_map(|(visible_idx, actual_idx)| {
                self.items.get(*actual_idx).map(|item| {
                    let prefix = if self.selected_idx == Some(visible_idx) {
                        '›'
                    } else {
                        ' '
                    };
                    let enabled = if item.enabled { 'x' } else { ' ' };
                    let tools = match item.tool_count {
                        1 => "1 tool".to_string(),
                        count => format!("{count} tools"),
                    };
                    Line::from(format!("{prefix} [{enabled}] {:<24} {tools}", item.name))
                })
            })
            .collect()
    }

    fn selected_description(&self) -> Line<'static> {
        let Some(actual_idx) = self.selected_actual_idx() else {
            return Line::from("No MCP server selected.".dim());
        };
        let Some(item) = self.items.get(actual_idx) else {
            return Line::from("No MCP server selected.".dim());
        };
        let auth = item.auth_status.map(auth_status_label).unwrap_or("unknown");
        let action = if item.enabled { "disable" } else { "enable" };
        Line::from(format!(
            "Auth: {auth}. Space to {action}; changes take effect on the next turn."
        ))
        .dim()
    }
}

impl BottomPaneView for McpManagementView {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let allow_plain_char_navigation = !is_plain_text_key_event(key_event);
        match key_event {
            _ if allow_plain_char_navigation && self.keymap.move_up.is_pressed(key_event) => {
                self.move_up()
            }
            _ if allow_plain_char_navigation && self.keymap.move_down.is_pressed(key_event) => {
                self.move_down()
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                self.search_query.pop();
                self.apply_filter();
            }
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => self.toggle_selected(),
            _ if self.keymap.accept.is_pressed(key_event) => self.toggle_selected(),
            _ if self.keymap.cancel.is_pressed(key_event) => {
                self.on_ctrl_c();
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers,
                ..
            } if !modifiers.contains(KeyModifiers::CONTROL)
                && !modifiers.contains(KeyModifiers::ALT) =>
            {
                self.search_query.push(c);
                self.apply_filter();
            }
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(MCP_MANAGEMENT_VIEW_ID)
    }

    fn selected_index(&self) -> Option<usize> {
        self.selected_actual_idx()
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.close();
        CancellationEvent::Handled
    }
}

impl Renderable for McpManagementView {
    fn desired_height(&self, width: u16) -> u16 {
        let header_height = self.header.desired_height(width.saturating_sub(4));
        header_height
            .saturating_add(1)
            .saturating_add(2)
            .saturating_add(Self::max_visible_rows(self.visible_len()) as u16)
            .saturating_add(2)
            .saturating_add(1)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let [content_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        Block::default()
            .style(user_message_style())
            .render(content_area, buf);

        let inner = content_area.inset(Insets::vh(/*v*/ 1, /*h*/ 2));
        let header_height = self.header.desired_height(inner.width);
        let rows_height = Self::max_visible_rows(self.visible_len()) as u16;
        let [header_area, _, search_area, list_area, description_area] = Layout::vertical([
            Constraint::Max(header_height),
            Constraint::Max(1),
            Constraint::Length(2),
            Constraint::Length(rows_height),
            Constraint::Length(1),
        ])
        .areas(inner);

        self.header.render(header_area, buf);

        if search_area.height >= 2 {
            let [placeholder_area, input_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(search_area);
            Line::from(SEARCH_PLACEHOLDER.dim()).render(placeholder_area, buf);
            let line = if self.search_query.is_empty() {
                Line::from(vec![SEARCH_PROMPT_PREFIX.dim()])
            } else {
                Line::from(vec![
                    SEARCH_PROMPT_PREFIX.dim(),
                    self.search_query.clone().into(),
                ])
            };
            line.render(input_area, buf);
        }

        let rows = self.build_rows();
        if rows.is_empty() {
            Line::from("no matches".dim()).render(list_area, buf);
        } else {
            for (offset, row) in rows.into_iter().enumerate() {
                let y = list_area.y.saturating_add(offset as u16);
                if y >= list_area.y.saturating_add(list_area.height) {
                    break;
                }
                row.render(
                    Rect {
                        x: list_area.x,
                        y,
                        width: list_area.width,
                        height: 1,
                    },
                    buf,
                );
            }
        }

        self.selected_description().render(description_area, buf);

        let hint_area = Rect {
            x: footer_area.x + 2,
            y: footer_area.y,
            width: footer_area.width.saturating_sub(2),
            height: footer_area.height,
        };
        self.footer_hint.clone().dim().render(hint_area, buf);
    }
}

impl ChatWidget {
    pub(crate) fn open_mcp_management_popup(&mut self) {
        let items = self.mcp_management_items();
        let view = McpManagementView::new(
            items,
            self.app_event_tx.clone(),
            self.bottom_pane.list_keymap(),
            None,
        );
        self.bottom_pane.show_view(Box::new(view));
        self.request_redraw();
        self.app_event_tx.send(AppEvent::FetchMcpManagementStatus {
            thread_id: self.thread_id(),
        });
    }

    pub(crate) fn on_mcp_management_status_loaded(
        &mut self,
        result: Result<Vec<McpServerStatus>, String>,
    ) {
        match result {
            Ok(statuses) => {
                self.mcp_management_statuses = statuses;
                self.refresh_mcp_management_popup();
            }
            Err(err) => {
                self.add_error_message(format!("Failed to load MCP servers: {err}"));
            }
        }
    }

    pub(crate) fn on_mcp_server_enabled_pending(&mut self, server_name: String, enabled: bool) {
        self.mcp_management_enabled_overrides
            .insert(server_name, enabled);
        self.refresh_mcp_management_popup();
    }

    pub(crate) fn on_mcp_server_enabled_set(
        &mut self,
        server_name: String,
        enabled: bool,
        result: Result<ConfigWriteResponse, String>,
    ) {
        self.mcp_management_enabled_overrides.remove(&server_name);
        match result {
            Ok(response) => self.apply_mcp_server_enabled_write(server_name, enabled, response),
            Err(err) => {
                self.add_error_message(err);
                self.refresh_mcp_management_popup();
            }
        }
    }

    fn apply_mcp_server_enabled_write(
        &mut self,
        server_name: String,
        enabled: bool,
        response: ConfigWriteResponse,
    ) {
        let effective_enabled = if response.status == WriteStatus::OkOverridden {
            let message = response
                .overridden_metadata
                .as_ref()
                .map(|metadata| metadata.message.as_str())
                .unwrap_or("the effective config is overridden by a higher-priority layer");
            self.add_error_message(format!(
                "MCP server change was saved but not applied: {message}"
            ));
            response
                .overridden_metadata
                .as_ref()
                .and_then(|metadata| metadata.effective_value.as_bool())
                .unwrap_or(enabled)
        } else {
            enabled
        };

        let mut servers = self.config.mcp_servers.get().clone();
        if let Some(server) = servers.get_mut(&server_name) {
            server.enabled = effective_enabled;
            if let Err(err) = self.config.mcp_servers.set(servers) {
                self.add_error_message(format!(
                    "Updated MCP config on disk, but failed to refresh UI state: {err}"
                ));
            }
        }
        self.refresh_mcp_management_popup();
    }

    fn refresh_mcp_management_popup(&mut self) {
        let selected_idx = self
            .bottom_pane
            .selected_index_for_active_view(MCP_MANAGEMENT_VIEW_ID);
        let view = McpManagementView::new(
            self.mcp_management_items(),
            self.app_event_tx.clone(),
            self.bottom_pane.list_keymap(),
            selected_idx,
        );
        if self
            .bottom_pane
            .replace_active_view_with_view(MCP_MANAGEMENT_VIEW_ID, Box::new(view))
        {
            self.request_redraw();
        }
    }

    fn mcp_management_items(&self) -> Vec<McpManagementItem> {
        let status_by_name: HashMap<&str, &McpServerStatus> = self
            .mcp_management_statuses
            .iter()
            .map(|status| (status.name.as_str(), status))
            .collect();
        let mut items: Vec<McpManagementItem> = self
            .config
            .mcp_servers
            .get()
            .iter()
            .map(|(name, server)| {
                let status = status_by_name.get(name.as_str()).copied();
                McpManagementItem {
                    name: name.clone(),
                    enabled: self
                        .mcp_management_enabled_overrides
                        .get(name)
                        .copied()
                        .unwrap_or(server.enabled),
                    tool_count: status.map(|status| status.tools.len()).unwrap_or(0),
                    auth_status: status.map(|status| status.auth_status),
                }
            })
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }
}

fn auth_status_label(status: McpAuthStatus) -> &'static str {
    match status {
        McpAuthStatus::Unsupported => "unsupported",
        McpAuthStatus::NotLoggedIn => "not logged in",
        McpAuthStatus::BearerToken => "bearer token",
        McpAuthStatus::OAuth => "oauth",
    }
}

fn is_plain_text_key_event(key_event: KeyEvent) -> bool {
    matches!(
        key_event,
        KeyEvent {
            code: KeyCode::Char(_),
            modifiers,
            ..
        } if !modifiers.contains(KeyModifiers::CONTROL)
            && !modifiers.contains(KeyModifiers::ALT)
    )
}

fn mcp_management_hint_line(keymap: &ListKeymap) -> Line<'static> {
    let space = key_hint::plain(KeyCode::Char(' '));
    let accept = primary_binding(&keymap.accept).filter(|binding| *binding != space);
    let cancel = primary_binding(&keymap.cancel);

    match (accept, cancel) {
        (Some(accept), Some(cancel)) => Line::from(vec![
            "Press ".into(),
            space.into(),
            " or ".into(),
            accept.into(),
            " to toggle; ".into(),
            cancel.into(),
            " to close".into(),
        ]),
        (Some(accept), None) => Line::from(vec![
            "Press ".into(),
            space.into(),
            " or ".into(),
            accept.into(),
            " to toggle".into(),
        ]),
        (None, Some(cancel)) => Line::from(vec![
            "Press ".into(),
            space.into(),
            " to toggle; ".into(),
            cancel.into(),
            " to close".into(),
        ]),
        (None, None) => Line::from(vec!["Press ".into(), space.into(), " to toggle".into()]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_event::AppEvent;
    use tokio::sync::mpsc::unbounded_channel;

    #[test]
    fn mcp_management_view_space_toggles_while_search_is_active() {
        let (tx_raw, mut rx) = unbounded_channel::<AppEvent>();
        let tx = AppEventSender::new(tx_raw);
        let mut view = McpManagementView::new(
            vec![McpManagementItem {
                name: "docs".to_string(),
                enabled: true,
                tool_count: 2,
                auth_status: Some(McpAuthStatus::OAuth),
            }],
            tx,
            crate::keymap::RuntimeKeymap::defaults().list,
            None,
        );

        view.handle_key_event(KeyEvent::from(KeyCode::Char('d')));
        view.handle_key_event(KeyEvent::from(KeyCode::Char(' ')));

        assert!(!view.items[0].enabled);
        match rx.try_recv().expect("toggle event") {
            AppEvent::SetMcpServerEnabled {
                server_name,
                enabled,
            } => {
                assert_eq!(server_name, "docs");
                assert!(!enabled);
            }
            other => panic!("expected MCP toggle event, got {other:?}"),
        }
    }
}
