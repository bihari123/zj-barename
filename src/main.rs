mod line;
mod tab;

use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::convert::TryInto;

use tab::get_tab_to_focus;
use zellij_tile::prelude::*;

use crate::line::tab_line;
use crate::tab::tab_style;

#[derive(Debug, Default)]
pub struct LinePart {
    part: String,
    len: usize,
    tab_index: Option<usize>,
}

impl LinePart {
    pub fn append(&mut self, to_append: &LinePart) {
        self.part.push_str(&to_append.part);
        self.len += to_append.len;
    }
}

#[derive(Default, Debug)]
struct State {
    tabs: Vec<TabInfo>,
    active_tab_idx: usize,
    mode_info: ModeInfo,
    tab_line: Vec<LinePart>,
    hide_swap_layout_indication: bool,
    cached_keybinds: KeybindsVec,
    // zj-barename: right-aligned HH:MM clock. `utc_offset_secs` is derived from the
    // `utc_offset` plugin config (e.g. "+05:30"); `clock_cache` holds the last
    // rendered "HH:MM" so the timer only forces a redraw when the minute changes.
    utc_offset_secs: i64,
    clock_cache: String,
    show_clock: bool,
}

static ARROW_SEPARATOR: &str = "";

register_plugin!(State);

// zj-barename: parse a "+HH:MM" / "-HH:MM" (or "+HH") UTC offset into seconds.
fn parse_utc_offset(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (sign, rest) = match s.as_bytes()[0] {
        b'+' => (1i64, &s[1..]),
        b'-' => (-1i64, &s[1..]),
        _ => (1i64, s),
    };
    let mut it = rest.split(':');
    let hours: i64 = it.next()?.trim().parse().ok()?;
    let minutes: i64 = match it.next() {
        Some(m) => m.trim().parse().ok()?,
        None => 0,
    };
    Some(sign * (hours * 3600 + minutes * 60))
}

// zj-barename: current wall-clock time as "HH:MM" (24h), shifted by `offset_secs`.
fn current_hhmm(offset_secs: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let sod = (secs + offset_secs).rem_euclid(86_400);
    format!("{:02}:{:02}", sod / 3600, (sod % 3600) / 60)
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.hide_swap_layout_indication = configuration
            .get("hide_swap_layout_indication")
            .map(|s| s == "true")
            .unwrap_or(false);
        // zj-barename: right-aligned clock. Enabled unless `clock "false"`.
        // `utc_offset` (e.g. "+05:30", "-08:00") shifts UTC to local time.
        self.show_clock = configuration
            .get("clock")
            .map(|s| s != "false")
            .unwrap_or(true);
        self.utc_offset_secs = configuration
            .get("utc_offset")
            .and_then(|s| parse_utc_offset(s))
            .unwrap_or(0);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        set_selectable(false);
        subscribe(&[
            EventType::TabUpdate,
            EventType::ModeUpdate,
            EventType::Mouse,
            EventType::InitialKeybinds,
            EventType::Timer,
        ]);
        if self.show_clock {
            set_timeout(1.0);
        }
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::InitialKeybinds(keybinds) => {
                self.cached_keybinds = keybinds;
                if !self.cached_keybinds.is_empty() {
                    self.mode_info.keybinds = self.cached_keybinds.clone();
                }
                should_render = true;
            },
            Event::ModeUpdate(mut mode_info) => {
                if mode_info.keybinds.is_empty() && !self.cached_keybinds.is_empty() {
                    mode_info.keybinds = self.cached_keybinds.clone();
                } else if !mode_info.keybinds.is_empty() {
                    self.cached_keybinds = mode_info.keybinds.clone();
                }
                if self.mode_info != mode_info {
                    should_render = true;
                }
                self.mode_info = mode_info;
            },
            Event::TabUpdate(tabs) => {
                if let Some(active_tab_index) = tabs.iter().position(|t| t.active) {
                    // tabs are indexed starting from 1 so we need to add 1
                    let active_tab_idx = active_tab_index + 1;

                    if self.active_tab_idx != active_tab_idx || self.tabs != tabs {
                        should_render = true;
                    }
                    self.active_tab_idx = active_tab_idx;
                    self.tabs = tabs;
                } else {
                    eprintln!("Could not find active tab.");
                }
            },
            Event::Mouse(me) => match me {
                Mouse::LeftClick(_, col) => {
                    let tab_to_focus = get_tab_to_focus(&self.tab_line, self.active_tab_idx, col);
                    if let Some(idx) = tab_to_focus {
                        switch_tab_to(idx.try_into().unwrap());
                    }
                },
                Mouse::ScrollUp(_) => {
                    switch_tab_to(min(self.active_tab_idx + 1, self.tabs.len()) as u32);
                },
                Mouse::ScrollDown(_) => {
                    switch_tab_to(max(self.active_tab_idx.saturating_sub(1), 1) as u32);
                },
                _ => {},
            },
            Event::Timer(_) => {
                // zj-barename: tick the clock. Re-arm the timer and only request a
                // redraw when the displayed minute actually changes.
                if self.show_clock {
                    set_timeout(1.0);
                    let now = current_hhmm(self.utc_offset_secs);
                    if now != self.clock_cache {
                        self.clock_cache = now;
                        should_render = true;
                    }
                }
            },
            _ => {
                eprintln!("Got unrecognized event: {:?}", event);
            },
        }
        if self.tabs.is_empty() {
            // no need to render if we have no tabs, this can sometimes happen on startup before we
            // get the tab update and then we definitely don't want to render
            should_render = false;
        }
        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        if self.tabs.is_empty() {
            return;
        }
        let mut all_tabs: Vec<LinePart> = vec![];
        let mut active_tab_index = 0;
        let mut is_alternate_tab = false;
        for t in &mut self.tabs {
            let mut tabname = t.name.clone();
            if t.active && self.mode_info.mode == InputMode::RenameTab {
                if tabname.is_empty() {
                    tabname = String::from("Enter name...");
                }
                active_tab_index = t.position;
            } else if t.active {
                active_tab_index = t.position;
            }
            let tab = tab_style(
                tabname,
                t,
                is_alternate_tab,
                self.mode_info.style.colors,
                self.mode_info.capabilities,
            );
            is_alternate_tab = !is_alternate_tab;
            all_tabs.push(tab);
        }

        let background = self.mode_info.style.colors.text_unselected.background;

        // zj-barename: build the clock and reserve its width so tabs don't overlap it.
        let clock = if self.show_clock {
            format!(" {} ", current_hhmm(self.utc_offset_secs)) // e.g. " 14:22 "
        } else {
            String::new()
        };
        let clock_w = clock.chars().count();

        self.tab_line = tab_line(
            self.mode_info.session_name.as_deref(),
            all_tabs,
            active_tab_index,
            cols.saturating_sub(1 + clock_w),
            self.mode_info.style.colors,
            self.mode_info.capabilities,
            self.mode_info.style.hide_session_name,
            self.tabs.iter().find(|t| t.active),
            &self.mode_info,
            self.hide_swap_layout_indication,
            &background,
        );

        let output = self
            .tab_line
            .iter()
            .fold(String::new(), |output, part| output + &part.part);

        let bg_fill = match background {
            PaletteColor::Rgb((r, g, b)) => format!("\u{1b}[48;2;{};{};{}m", r, g, b),
            PaletteColor::EightBit(color) => format!("\u{1b}[48;5;{}m", color),
        };

        if clock_w == 0 {
            print!("{}{}\u{1b}[0K", output, bg_fill);
            return;
        }

        // zj-barename: right-align the clock — fill the gap with the bar background,
        // then render the time in the normal (bold) tab-text colour.
        let used: usize = self.tab_line.iter().map(|p| p.len).sum();
        let pad = cols.saturating_sub(used + clock_w);
        let clock_fg = match self.mode_info.style.colors.text_unselected.base {
            PaletteColor::Rgb((r, g, b)) => format!("\u{1b}[38;2;{};{};{}m", r, g, b),
            PaletteColor::EightBit(color) => format!("\u{1b}[38;5;{}m", color),
        };
        print!(
            "{}{}{}{}\u{1b}[1m{}\u{1b}[m{}\u{1b}[0K",
            output,
            bg_fill,
            " ".repeat(pad),
            clock_fg,
            clock,
            bg_fill,
        );
    }
}
