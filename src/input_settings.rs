use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gtk4::gdk;
use gtk4::glib::translate::{IntoGlib, FromGlib};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    PrevDir,
    NextDir,
    PrevPage,
    NextPage,
    ToggleFullscreen,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleSpread,
    ToggleRTL,
    PrevPageSingle,
    NextPageSingle,
}

impl Action {
    pub fn description(&self) -> &str {
        match self {
            Action::PrevDir => "Previous Directory / Archive",
            Action::NextDir => "Next Directory / Archive",
            Action::PrevPage => "Previous Image",
            Action::NextPage => "Next Image",
            Action::ToggleFullscreen => "Toggle Fullscreen",
            Action::ZoomIn => "Zoom In",
            Action::ZoomOut => "Zoom Out",
            Action::ResetZoom => "Reset Zoom",
            Action::ToggleSpread => "Toggle Spread View",
            Action::ToggleRTL => "Toggle Right-to-Left",
            Action::PrevPageSingle => "Previous Image (Single Step)",
            Action::NextPageSingle => "Next Image (Single Step)",
        }
    }
    pub fn variants() -> &'static [Action] {
        &[
            Action::PrevDir,
            Action::NextDir,
            Action::PrevPage,
            Action::NextPage,
            Action::ToggleFullscreen,
            Action::ZoomIn,
            Action::ZoomOut,
            Action::ResetZoom,
            Action::ToggleSpread,
            Action::ToggleRTL,
            Action::PrevPageSingle,
            Action::NextPageSingle,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InputSpec {
    Keyboard { keyval: u32, modifiers: u32 },
    Mouse { button: u32, modifiers: u32, double_click: bool },
    Scroll { direction: ScrollDirection, modifiers: u32 },
}

impl InputSpec {
    pub fn matches_key(&self, key: gdk::Key, mods: gdk::ModifierType) -> bool {
        if let InputSpec::Keyboard { keyval, modifiers } = self {
            // Mask out non-relevant modifiers (like NumLock, CapsLock if needed, but usually exact match is expected unless ignored)
            // For now exact match on significant modifiers
            let relevant_mods = mods & (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK);
            return key.to_lower() == unsafe { gdk::Key::from_glib(*keyval) }.to_lower() && relevant_mods.bits() == *modifiers;
        }
        false
    }

    pub fn matches_mouse(&self, btn: u32, mods: gdk::ModifierType, is_double: bool) -> bool {
        if let InputSpec::Mouse { button, modifiers, double_click } = self {
            let relevant_mods = mods & (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK);
            return *button == btn && relevant_mods.bits() == *modifiers && *double_click == is_double;
        }
        false
    }
    
    pub fn matches_scroll(&self, dir: ScrollDirection, mods: gdk::ModifierType) -> bool {
        if let InputSpec::Scroll { direction, modifiers } = self {
             let relevant_mods = mods & (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK);
             return *direction == dir && relevant_mods.bits() == *modifiers;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMap {
    pub map: HashMap<Action, Vec<InputSpec>>,
}

impl Default for InputMap {
    fn default() -> Self {
        let mut map = HashMap::new();
        
        // PrevDir: Up Arrow
        map.insert(Action::PrevDir, vec![
            InputSpec::Keyboard { keyval: gdk::Key::Up.into_glib(), modifiers: 0 }
        ]);
        
        // NextDir: Down Arrow
        map.insert(Action::NextDir, vec![
            InputSpec::Keyboard { keyval: gdk::Key::Down.into_glib(), modifiers: 0 }
        ]);
        
        // PrevPage: Left Arrow, Scroll Up
        map.insert(Action::PrevPage, vec![
            InputSpec::Keyboard { keyval: gdk::Key::Left.into_glib(), modifiers: 0 },
            InputSpec::Scroll { direction: ScrollDirection::Up, modifiers: 0 }
        ]);
        
        // NextPage: Right Arrow, Scroll Down
        map.insert(Action::NextPage, vec![
            InputSpec::Keyboard { keyval: gdk::Key::Right.into_glib(), modifiers: 0 },
            InputSpec::Scroll { direction: ScrollDirection::Down, modifiers: 0 }
        ]);
        
        // ToggleFullscreen: Enter, Double Click (Left Button)
        map.insert(Action::ToggleFullscreen, vec![
            InputSpec::Keyboard { keyval: gdk::Key::Return.into_glib(), modifiers: 0 },
            InputSpec::Mouse { button: gdk::BUTTON_PRIMARY, modifiers: 0, double_click: true }
        ]);
        
        // ZoomIn: + (plus)
        map.insert(Action::ZoomIn, vec![
            InputSpec::Keyboard { keyval: gdk::Key::plus.into_glib(), modifiers: 0 }
        ]);
        
        // ZoomOut: - (minus)
        map.insert(Action::ZoomOut, vec![
            InputSpec::Keyboard { keyval: gdk::Key::minus.into_glib(), modifiers: 0 }
        ]);
        
        // ResetZoom: Escape
        map.insert(Action::ResetZoom, vec![
            InputSpec::Keyboard { keyval: gdk::Key::Escape.into_glib(), modifiers: 0 }
        ]);
        
        // ToggleSpread: M
        map.insert(Action::ToggleSpread, vec![
            InputSpec::Keyboard { keyval: gdk::Key::m.into_glib(), modifiers: 0 }
        ]);
        
        // ToggleRTL: N (Requested Default)
        map.insert(Action::ToggleRTL, vec![
            InputSpec::Keyboard { keyval: gdk::Key::n.into_glib(), modifiers: 0 }
        ]);
        
        // PrevPageSingle: Shift + Left
        map.insert(Action::PrevPageSingle, vec![
             InputSpec::Keyboard { keyval: gdk::Key::Left.into_glib(), modifiers: gdk::ModifierType::SHIFT_MASK.bits() }
        ]);

        // NextPageSingle: Shift + Right
        map.insert(Action::NextPageSingle, vec![
             InputSpec::Keyboard { keyval: gdk::Key::Right.into_glib(), modifiers: gdk::ModifierType::SHIFT_MASK.bits() }
        ]);

        Self { map }
    }
}

impl InputMap {
    pub fn get_action_for_key(&self, key: gdk::Key, modifiers: gdk::ModifierType) -> Option<Action> {
        for (action, specs) in &self.map {
            for spec in specs {
                if spec.matches_key(key, modifiers) {
                    return Some(*action);
                }
            }
        }
        None
    }
    
    pub fn get_action_for_mouse(&self, button: u32, modifiers: gdk::ModifierType, is_double: bool) -> Option<Action> {
        for (action, specs) in &self.map {
            for spec in specs {
                if spec.matches_mouse(button, modifiers, is_double) {
                    return Some(*action);
                }
            }
        }
        None
    }
    
     pub fn get_action_for_scroll(&self, direction: ScrollDirection, modifiers: gdk::ModifierType) -> Option<Action> {
        for (action, specs) in &self.map {
            for spec in specs {
                if spec.matches_scroll(direction, modifiers) {
                    return Some(*action);
                }
            }
        }
        None
    }
}
