
use relm4::prelude::*;
use relm4::factory::{FactoryVecDeque, DynamicIndex, FactoryComponent, FactorySender};
use gtk4::prelude::*;
use crate::database::{AppSettings, SortType};
use crate::input_settings::{Action, InputMap, InputSpec};
use gtk4::gdk;
use gtk4::glib::translate::{IntoGlib, FromGlib};
use crate::i18n::{Language, localize};

#[derive(Debug)]
pub struct SettingsDialogModel {
    pub is_active: bool,
    pub dark_mode: bool,
    pub default_spread_view: bool,
    pub default_right_to_left: bool,
    pub default_dir_sort: SortType,
    pub default_image_sort: SortType,
    pub loop_images: bool,
    pub single_first_page: bool,
    pub input_map: InputMap,
    pub capturing_action: Option<Action>,
    pub keyboard_rows: FactoryVecDeque<KeyboardItem>,
    pub mouse_rows: FactoryVecDeque<MouseItem>,
    pub language: Language,
}

#[derive(Debug)]
pub enum SettingsDialogMsg {
    Open(AppSettings),
    Close,
    Save,
    UpdateDarkMode(bool),
    UpdateDefaultSpread(bool),
    UpdateDefaultRTL(bool),
    UpdateDefaultDirSort(SortType),
    UpdateDefaultImageSort(SortType),
    UpdateLoopImages(bool),
    UpdateSingleFirstPage(bool),
    StartCapture(Action),
    CancelCapture,
    CaptureInput(InputSpec),
    ResetInputs,
    UpdateMouseBinding(MouseInputType, Option<Action>),
    UpdateLanguage(Language),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseInputType {
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
    LeftDouble,
}

impl MouseInputType {
    fn label(&self, lang: Language) -> String {
        let key = match self {
            Self::RightClick => "Right Click",
            Self::MiddleClick => "Middle Click",
            Self::ScrollUp => "Scroll Up",
            Self::ScrollDown => "Scroll Down",
            Self::LeftDouble => "Left Double Click",
        };
        localize(key, lang)
    }

    fn variants() -> &'static [MouseInputType] {
        &[
            Self::RightClick,
            Self::MiddleClick,
            Self::ScrollUp,
            Self::ScrollDown,
            Self::LeftDouble,
        ]
    }

    fn to_input_spec(&self) -> InputSpec {
        match self {
            Self::RightClick => InputSpec::Mouse { button: 3, modifiers: 0, double_click: false },
            Self::MiddleClick => InputSpec::Mouse { button: 2, modifiers: 0, double_click: false },
            Self::ScrollUp => InputSpec::Scroll { direction: crate::input_settings::ScrollDirection::Up, modifiers: 0 },
            Self::ScrollDown => InputSpec::Scroll { direction: crate::input_settings::ScrollDirection::Down, modifiers: 0 },
            Self::LeftDouble => InputSpec::Mouse { button: 1, modifiers: 0, double_click: true },
        }
    }
}

#[derive(Debug)]
pub enum SettingsDialogOutput {
    SaveSettings(AppSettings),
    Close,
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsDialogModel {
    type Input = SettingsDialogMsg;
    type Output = SettingsDialogOutput;
    type Init = ();

    view! {
        settings_window = gtk4::Window {
            #[watch]
            set_title: Some(&localize("Settings", model.language)),
            set_default_width: 600,
            set_default_height: 700, 
            set_hide_on_close: true,
            set_modal: true,
            #[watch]
            set_visible: model.is_active,
            
            connect_close_request[sender] => move |_| {
                sender.input(SettingsDialogMsg::Close);
                gtk4::glib::Propagation::Stop
            },
            
            // Key Controller for Capturing
            add_controller = gtk4::EventControllerKey {
                connect_key_pressed[sender] => move |_, key, _, modifiers| {
                    let is_modifier = matches!(key,
                        gdk::Key::Shift_L | gdk::Key::Shift_R |
                        gdk::Key::Control_L | gdk::Key::Control_R |
                        gdk::Key::Alt_L | gdk::Key::Alt_R |
                        gdk::Key::Meta_L | gdk::Key::Meta_R |
                        gdk::Key::Super_L | gdk::Key::Super_R
                    );

                    if !is_modifier {
                        sender.input(SettingsDialogMsg::CaptureInput(InputSpec::Keyboard {
                             keyval: key.into_glib(),
                             modifiers: modifiers.bits(),
                        }));
                    }
                    gtk4::glib::Propagation::Proceed
                }
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                
                // Capture Overlay (Conditional)
                #[name(capture_overlay)]
                gtk4::Overlay {
                    set_vexpand: true,
                    set_hexpand: true,
                    
                    #[wrap(Some)]
                    set_child = &gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        
                        gtk4::ScrolledWindow {
                            set_vexpand: true,
                            set_hexpand: true,
                            
                            gtk4::Box {
                                set_orientation: gtk4::Orientation::Vertical,
                                set_spacing: 15,
                                set_margin_all: 20,
                                
                                // --- Directory Defaults ---
                                gtk4::Label {
                                    #[watch]
                                    set_label: &localize("Directory Defaults (Applied to new directories)", model.language),
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                },
                                
                                gtk4::CheckButton {
                                    #[watch]
                                    set_label: Some(&localize("Default Spread View", model.language)),
                                    #[watch]
                                    set_active: model.default_spread_view,
                                    connect_toggled[sender] => move |btn| {
                                        sender.input(SettingsDialogMsg::UpdateDefaultSpread(btn.is_active()));
                                    }
                                },
                                
                                gtk4::CheckButton {
                                    #[watch]
                                    set_label: Some(&localize("Default Right to Left", model.language)),
                                    #[watch]
                                    set_active: model.default_right_to_left,
                                    connect_toggled[sender] => move |btn| {
                                        sender.input(SettingsDialogMsg::UpdateDefaultRTL(btn.is_active()));
                                    }
                                },
                                
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Horizontal,
                                    set_spacing: 10,
                                    
                                    gtk4::Label {
                                        #[watch]
                                        set_label: &localize("Default Dir Sort:", model.language),
                                    },
                                    
                                    #[name(dir_sort_combo)]
                                    gtk4::ComboBoxText {
                                        append: (Some("NameAsc"), &localize("Name Asc", model.language)),
                                        append: (Some("NameDesc"), &localize("Name Desc", model.language)),
                                        append: (Some("DateAsc"), &localize("Date Asc", model.language)),
                                        append: (Some("DateDesc"), &localize("Date Desc", model.language)),
                                        append: (Some("SizeAsc"), &localize("Size Asc", model.language)),
                                        append: (Some("SizeDesc"), &localize("Size Desc", model.language)),
                                        #[watch]
                                        set_active_id: Some(match model.default_dir_sort {
                                            SortType::NameAsc => "NameAsc",
                                            SortType::NameDesc => "NameDesc",
                                            SortType::DateAsc => "DateAsc",
                                            SortType::DateDesc => "DateDesc",
                                            SortType::SizeAsc => "SizeAsc",
                                            SortType::SizeDesc => "SizeDesc",
                                        }),
                                        connect_changed[sender] => move |cb| {
                                            if let Some(id) = cb.active_id() {
                                                let sort = match id.as_str() {
                                                    "NameAsc" => SortType::NameAsc,
                                                    "NameDesc" => SortType::NameDesc,
                                                    "DateAsc" => SortType::DateAsc,
                                                    "DateDesc" => SortType::DateDesc,
                                                    "SizeAsc" => SortType::SizeAsc,
                                                    "SizeDesc" => SortType::SizeDesc,
                                                    _ => SortType::NameAsc,
                                                };
                                                sender.input(SettingsDialogMsg::UpdateDefaultDirSort(sort));
                                            }
                                        }
                                    },
                                },
        
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Horizontal,
                                    set_spacing: 10,
                                    
                                    gtk4::Label {
                                        #[watch]
                                        set_label: &localize("Default Image Sort:", model.language),
                                    },
                                     #[name(image_sort_combo)]
                                     gtk4::ComboBoxText {
                                        append: (Some("NameAsc"), &localize("Name Asc", model.language)),
                                        append: (Some("NameDesc"), &localize("Name Desc", model.language)),
                                        append: (Some("DateAsc"), &localize("Date Asc", model.language)),
                                        append: (Some("DateDesc"), &localize("Date Desc", model.language)),
                                        append: (Some("SizeAsc"), &localize("Size Asc", model.language)),
                                        append: (Some("SizeDesc"), &localize("Size Desc", model.language)),
                                        #[watch]
                                        set_active_id: Some(match model.default_image_sort {
                                            SortType::NameAsc => "NameAsc",
                                            SortType::NameDesc => "NameDesc",
                                            SortType::DateAsc => "DateAsc",
                                            SortType::DateDesc => "DateDesc",
                                            SortType::SizeAsc => "SizeAsc",
                                            SortType::SizeDesc => "SizeDesc",
                                        }),
                                        connect_changed[sender] => move |cb| {
                                            if let Some(id) = cb.active_id() {
                                                let sort = match id.as_str() {
                                                    "NameAsc" => SortType::NameAsc,
                                                    "NameDesc" => SortType::NameDesc,
                                                    "DateAsc" => SortType::DateAsc,
                                                    "DateDesc" => SortType::DateDesc,
                                                    "SizeAsc" => SortType::SizeAsc,
                                                    "SizeDesc" => SortType::SizeDesc,
                                                    _ => SortType::NameAsc,
                                                };
                                                sender.input(SettingsDialogMsg::UpdateDefaultImageSort(sort));
                                            }
                                        }
                                    },
                                },
                                
                                gtk4::Separator {},
        
                                // --- Application Settings ---
                                gtk4::Label {
                                    #[watch]
                                    set_label: &localize("Application Settings", model.language),
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                },
                                
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Horizontal,
                                    set_spacing: 10,
                                    
                                    gtk4::Label {
                                        #[watch]
                                        set_label: &localize("Language", model.language),
                                    },

                                    gtk4::ComboBoxText {
                                        append: (Some("English"), "English"),
                                        append: (Some("Japanese"), "日本語"),
                                        #[watch]
                                        set_active_id: Some(match model.language {
                                            Language::English => "English",
                                            Language::Japanese => "Japanese",
                                        }),
                                        connect_changed[sender] => move |cb| {
                                            if let Some(id) = cb.active_id() {
                                                let lang = match id.as_str() {
                                                    "English" => Language::English,
                                                    "Japanese" => Language::Japanese,
                                                    _ => Language::English,
                                                };
                                                sender.input(SettingsDialogMsg::UpdateLanguage(lang));
                                            }
                                        }
                                    },
                                },

                                gtk4::CheckButton {
                                    #[watch]
                                    set_label: Some(&localize("Dark Mode (Requires Restart)", model.language)),
                                    #[watch]
                                    set_active: model.dark_mode,
                                    connect_toggled[sender] => move |btn| {
                                        sender.input(SettingsDialogMsg::UpdateDarkMode(btn.is_active()));
                                    }
                                },
                                
                                gtk4::CheckButton {
                                     #[watch]
                                     set_label: Some(&localize("Loop Images (at end of list)", model.language)),
                                     #[watch]
                                     set_active: model.loop_images,
                                     connect_toggled[sender] => move |btn| {
                                         sender.input(SettingsDialogMsg::UpdateLoopImages(btn.is_active()));
                                     }
                                },
        
                                gtk4::CheckButton {
                                     #[watch]
                                     set_label: Some(&localize("Single Page for First Image (Spread View)", model.language)),
                                     #[watch]
                                     set_active: model.single_first_page,
                                     connect_toggled[sender] => move |btn| {
                                         sender.input(SettingsDialogMsg::UpdateSingleFirstPage(btn.is_active()));
                                     }
                                },
                                
                                gtk4::Separator {},
                                
                                // --- Input Settings ---
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Horizontal,
                                    gtk4::Label {
                                        #[watch]
                                        set_label: &localize("Input Configuration", model.language),
                                        set_xalign: 0.0,
                                        set_hexpand: true,
                                        add_css_class: "title-4",
                                    },
                                    gtk4::Button {
                                        #[watch]
                                        set_label: &localize("Reset to Defaults", model.language),
                                        connect_clicked => SettingsDialogMsg::ResetInputs,
                                    }
                                },
                                
                                // Keyboard Section
                                gtk4::Label {
                                    #[watch]
                                    set_label: &localize("Keyboard Shortcuts", model.language),
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                    set_margin_top: 10,
                                },

                                #[local_ref]
                                keyboard_list -> gtk4::ListBox,

                                // Mouse Section
                                gtk4::Label {
                                    #[watch]
                                    set_label: &localize("Mouse Configuration", model.language),
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                    set_margin_top: 10,
                                },

                                #[local_ref]
                                mouse_list -> gtk4::ListBox,
                            }
                        },
                        // Footer
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 10,
                            set_halign: gtk4::Align::End,
                            set_margin_all: 10,
                            
                            gtk4::Button {
                                #[watch]
                                set_label: &localize("Cancel", model.language),
                                connect_clicked => SettingsDialogMsg::Close,
                            },
                            
                            gtk4::Button {
                                #[watch]
                                set_label: &localize("Save", model.language),
                                add_css_class: "suggested-action",
                                connect_clicked => SettingsDialogMsg::Save,
                            }
                        }
                    }
                    }
                }
            }
        }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let keyboard_rows = FactoryVecDeque::builder()
            .launch(gtk4::ListBox::builder()
                .selection_mode(gtk4::SelectionMode::None)
                .css_classes(["boxed-list"])
                .vexpand(true)
                .hexpand(true)
                .build())
            .forward(sender.input_sender(), |msg| msg);
            
        let mouse_rows = FactoryVecDeque::builder()
            .launch(gtk4::ListBox::builder()
                .selection_mode(gtk4::SelectionMode::None)
                .css_classes(["boxed-list"])
                .vexpand(true)
                .hexpand(true)
                .build())
            .forward(sender.input_sender(), |msg| msg);

        let model = SettingsDialogModel {
            is_active: false,
            dark_mode: false,
            default_spread_view: false,
            default_right_to_left: true,
            default_dir_sort: SortType::NameAsc,
            default_image_sort: SortType::NameAsc,
            loop_images: false,
            single_first_page: false,
            input_map: InputMap::default(),
            capturing_action: None,
            keyboard_rows,
            mouse_rows,
            language: Language::default(),
        };
        
        let keyboard_list = model.keyboard_rows.widget().clone();
        let mouse_list = model.mouse_rows.widget().clone();
        let widgets = view_output!(keyboard_list, mouse_list);
        
        // Add overlay child for capturing manually
        let capture_label = gtk4::Label::builder()
            .label("Press a key (Esc to cancel)")
            .css_classes(["title-1", "dim-label"])
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();
        let capture_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .build();
        capture_box.append(&capture_label);
        
        let bg = gtk4::Box::builder()
            .css_classes(["view", "frame"])
            .hexpand(true)
            .vexpand(true)
            .build();
        bg.append(&capture_box);
        
        // Event controller for mouse capture on overlay
        let click_gesture = gtk4::GestureClick::new();
        let sender_clone = sender.clone();
        click_gesture.connect_pressed(move |gesture, n_press, _, _| {
            let button = gesture.current_button();
            let modifiers = gesture.current_event_state();
            if n_press == 2 {
                sender_clone.input(SettingsDialogMsg::CaptureInput(InputSpec::Mouse {
                    button,
                    modifiers: modifiers.bits(),
                    double_click: true,
                }));
            } else if n_press == 1 {
                 sender_clone.input(SettingsDialogMsg::CaptureInput(InputSpec::Mouse {
                    button,
                    modifiers: modifiers.bits(),
                    double_click: false,
                }));
            }
        });

        let scroll_controller = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE);
        let sender_clone_scroll = sender.clone();
        scroll_controller.connect_scroll(move |_, _dx, dy| {
            if dy < 0.0 {
                 sender_clone_scroll.input(SettingsDialogMsg::CaptureInput(InputSpec::Scroll {
                    direction: crate::input_settings::ScrollDirection::Up,
                    modifiers: gtk4::gdk::ModifierType::empty().bits(),
                 }));
            } else if dy > 0.0 {
                 sender_clone_scroll.input(SettingsDialogMsg::CaptureInput(InputSpec::Scroll {
                    direction: crate::input_settings::ScrollDirection::Down,
                    modifiers: gtk4::gdk::ModifierType::empty().bits(),
                 }));
            }
            gtk4::glib::Propagation::Stop
        });
        
        bg.add_controller(click_gesture);
        bg.add_controller(scroll_controller);
        
        widgets.capture_overlay.add_overlay(&bg);
        bg.set_visible(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SettingsDialogMsg::Open(settings) => {
                self.dark_mode = settings.dark_mode;
                self.default_spread_view = settings.default_spread_view;
                self.default_right_to_left = settings.default_right_to_left;
                self.default_dir_sort = settings.default_dir_sort;
                self.default_image_sort = settings.default_image_sort;
                self.loop_images = settings.loop_images;
                self.single_first_page = settings.single_first_page;
                self.input_map = settings.input_map.clone();
                self.language = settings.language;
                self.capturing_action = None;
                self.is_active = true;
                self.populate_factories();
            }
            SettingsDialogMsg::Close => {
                self.is_active = false;
                self.capturing_action = None;
                let _ = _sender.output(SettingsDialogOutput::Close);
            }
            SettingsDialogMsg::Save => {
                self.is_active = false;
                let new_settings = AppSettings {
                    key: "global".to_string(),
                    dark_mode: self.dark_mode,
                    default_spread_view: self.default_spread_view,
                    default_right_to_left: self.default_right_to_left,
                    default_dir_sort: self.default_dir_sort,
                    default_image_sort: self.default_image_sort,
                    loop_images: self.loop_images,
                    single_first_page: self.single_first_page,
                    input_map: self.input_map.clone(),
                    language: self.language,
                };
                let _ = _sender.output(SettingsDialogOutput::SaveSettings(new_settings));
            }
            SettingsDialogMsg::UpdateLanguage(lang) => {
                self.language = lang;
                self.populate_factories();
            }
            SettingsDialogMsg::UpdateDarkMode(val) => self.dark_mode = val,
            SettingsDialogMsg::UpdateDefaultSpread(val) => self.default_spread_view = val,
            SettingsDialogMsg::UpdateDefaultRTL(val) => self.default_right_to_left = val,
            SettingsDialogMsg::UpdateDefaultDirSort(val) => self.default_dir_sort = val,
            SettingsDialogMsg::UpdateDefaultImageSort(val) => self.default_image_sort = val,
            SettingsDialogMsg::UpdateLoopImages(val) => self.loop_images = val,
            SettingsDialogMsg::UpdateSingleFirstPage(val) => self.single_first_page = val,
            
            SettingsDialogMsg::StartCapture(action) => {
                self.capturing_action = Some(action);
            }
            SettingsDialogMsg::CancelCapture => {
                self.capturing_action = None;
            }
            SettingsDialogMsg::CaptureInput(spec) => {
                if let Some(action) = self.capturing_action {
                     if let InputSpec::Keyboard { keyval, .. } = spec {
                        if keyval == gtk4::gdk::Key::Escape.into_glib() {
                             self.capturing_action = None;
                             return;
                        }
                    }
                    
                    if matches!(spec, InputSpec::Keyboard { .. }) {
                        let mut new_specs = Vec::new();
                        if let Some(existing) = self.input_map.map.get(&action) {
                            for s in existing {
                                if !matches!(s, InputSpec::Keyboard { .. }) {
                                    new_specs.push(s.clone());
                                }
                            }
                        }
                        new_specs.push(spec);
                        self.input_map.map.insert(action, new_specs);
                        
                        // Update UI
                        if let Some(idx) = Action::variants().iter().position(|a| *a == action) {
                              let label = format_keyboard_specs(self.input_map.map.get(&action));
                              self.keyboard_rows.send(idx, KeyboardItemMsg::UpdateLabel(label));
                         }
                    }
                    self.capturing_action = None;
                }
            }
            SettingsDialogMsg::UpdateMouseBinding(input_type, action_opt) => {
                 for (_, specs) in self.input_map.map.iter_mut() {
                      specs.retain(|s| !matches_mouse_input(s, input_type));
                  }
                  
                  if let Some(action) = action_opt {
                      let mut specs = self.input_map.map.entry(action).or_insert(Vec::new()).clone();
                      specs.push(input_type.to_input_spec());
                      self.input_map.map.insert(action, specs);
                  }
                  
                  // Update UI
                  if let Some(idx) = MouseInputType::variants().iter().position(|t| *t == input_type) {
                      self.mouse_rows.send(idx, MouseItemMsg::UpdateSetting(action_opt));
                  }
            }
            SettingsDialogMsg::ResetInputs => {
                self.input_map = InputMap::default();
                self.populate_factories();
            }
        }
    }
}

fn format_keyboard_specs(specs: Option<&Vec<InputSpec>>) -> String {
    if let Some(specs) = specs {
        let s = specs.iter()
            .filter(|s| matches!(s, InputSpec::Keyboard { .. }))
            .map(|s| format_spec(s))
            .collect::<Vec<_>>()
            .join(", ");
        if s.is_empty() { "None".to_string() } else { s }
    } else {
        "None".to_string()
    }
}

// Helper for factories
impl SettingsDialogModel {
    fn populate_factories(&mut self) {
        self.keyboard_rows.guard().clear();
        for (idx, action) in Action::variants().iter().enumerate() {
            let label = format_keyboard_specs(self.input_map.map.get(action));
            let desc = action.description(self.language);
            self.keyboard_rows.guard().push_back((idx, *action, label, desc));
        }
        
        self.mouse_rows.guard().clear();
        for (idx, input_type) in MouseInputType::variants().iter().enumerate() {
            let current_action = get_action_for_mouse_lookup(&self.input_map, *input_type);
            let type_label = input_type.label(self.language);
            self.mouse_rows.guard().push_back((idx, *input_type, current_action, type_label, self.language));
        }
    }
}

fn format_spec(spec: &InputSpec) -> String {
    match spec {
        InputSpec::Keyboard { keyval, modifiers } => {
            let key_name = unsafe { gdk::Key::from_glib(*keyval) }.name().unwrap_or_else(|| "Unknown".into());
            let mods = gdk::ModifierType::from_bits_truncate(*modifiers);
            let mut s = String::new();
            if mods.contains(gdk::ModifierType::SHIFT_MASK) { s.push_str("Shift+"); }
            if mods.contains(gdk::ModifierType::CONTROL_MASK) { s.push_str("Ctrl+"); }
            if mods.contains(gdk::ModifierType::ALT_MASK) { s.push_str("Alt+"); }
            if mods.contains(gdk::ModifierType::SUPER_MASK) { s.push_str("Super+"); }
            s.push_str(&key_name);
            s
        }
        _ => "".to_string(), // Should not be called for mouse/scroll in this UI context
    }
}

fn matches_mouse_input(spec: &InputSpec, input_type: MouseInputType) -> bool {
    let target = input_type.to_input_spec();
    match (spec, target) {
        (InputSpec::Mouse { button: b1, modifiers: m1, double_click: d1 }, 
         InputSpec::Mouse { button: b2, modifiers: m2, double_click: d2 }) => {
             *b1 == b2 && *m1 == m2 && *d1 == d2
         },
        (InputSpec::Scroll { direction: d1, modifiers: m1 },
         InputSpec::Scroll { direction: d2, modifiers: m2 }) => {
             *d1 == d2 && *m1 == m2
         },
        _ => false
    }
}

fn get_action_for_mouse_lookup(input_map: &InputMap, input_type: MouseInputType) -> Option<Action> {
    for (action, specs) in &input_map.map {
        for spec in specs {
            if matches_mouse_input(spec, input_type) {
                return Some(*action);
            }
        }
    }
    None
}

#[derive(Debug)]
pub struct KeyboardItem {
    pub step_id: usize, // e.g. index in variants()
    pub action: Action,
    pub label: String,
    pub description: String,
}

#[derive(Debug)]
pub enum KeyboardItemMsg {
    UpdateLabel(String),
    Interact,
}

#[relm4::factory(pub)]
impl FactoryComponent for KeyboardItem {
    type Init = (usize, Action, String, String);
    type Input = KeyboardItemMsg;
    type Output = SettingsDialogMsg;
    type CommandOutput = ();
    type ParentWidget = gtk4::ListBox;

    view! {
        gtk4::ListBoxRow {
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_all: 5,
                
                gtk4::Label {
                    set_label: &self.description,
                    set_hexpand: true,
                    set_xalign: 0.0,
                },
                
                gtk4::Button {
                    #[watch]
                    set_label: &self.label,
                    connect_clicked[sender] => move |_| {
                        sender.input(KeyboardItemMsg::Interact);
                    }
                }
            }
        }
    }

    fn init_model((step_id, action, label, description): Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { step_id, action, label, description }
    }

    fn update(&mut self, msg: KeyboardItemMsg, _sender: FactorySender<Self>) {
        match msg {
            KeyboardItemMsg::UpdateLabel(s) => self.label = s,
            KeyboardItemMsg::Interact => {
                let _ = _sender.output(SettingsDialogMsg::StartCapture(self.action));
            }
        }
    }
}

#[derive(Debug)]
pub struct MouseItem {
    pub step_id: usize,
    pub input_type: MouseInputType,
    pub current_setting: Option<Action>,
    pub label_text: String,
    pub language: Language,
}

#[derive(Debug)]
pub enum MouseItemMsg {
    UpdateSetting(Option<Action>),
    Change(Option<Action>),
}

#[relm4::factory(pub)]
impl FactoryComponent for MouseItem {
    type Init = (usize, MouseInputType, Option<Action>, String, Language);
    type Input = MouseItemMsg;
    type Output = SettingsDialogMsg;
    type CommandOutput = ();
    type ParentWidget = gtk4::ListBox;

    view! {
        gtk4::ListBoxRow {
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 10,
                set_margin_all: 5,
                
                gtk4::Label {
                    set_label: &self.label_text,
                    set_hexpand: true,
                    set_xalign: 0.0,
                },
                
                gtk4::ComboBoxText {
                    append: (Some("None"), &localize("None", self.language)),
                    append: (Some("PrevDir"), &Action::PrevDir.description(self.language)),
                    append: (Some("NextDir"), &Action::NextDir.description(self.language)),
                    append: (Some("PrevPage"), &Action::PrevPage.description(self.language)),
                    append: (Some("NextPage"), &Action::NextPage.description(self.language)),
                    append: (Some("ToggleFullscreen"), &Action::ToggleFullscreen.description(self.language)),
                    append: (Some("ZoomIn"), &Action::ZoomIn.description(self.language)),
                    append: (Some("ZoomOut"), &Action::ZoomOut.description(self.language)),
                    append: (Some("ResetZoom"), &Action::ResetZoom.description(self.language)),
                    append: (Some("ToggleSpread"), &Action::ToggleSpread.description(self.language)),
                    append: (Some("ToggleRTL"), &Action::ToggleRTL.description(self.language)),
                    append: (Some("PrevPageSingle"), &Action::PrevPageSingle.description(self.language)),
                    append: (Some("NextPageSingle"), &Action::NextPageSingle.description(self.language)),
                    
                    #[watch]
                    set_active_id: Some(self.current_setting.map(|a| format!("{:?}", a)).unwrap_or("None".to_string()).as_str()),
                    
                    connect_changed[sender] => move |cb| {
                         if let Some(id) = cb.active_id() {
                             let action_opt = if id == "None" { None } else {
                                 match id.as_str() {
                                     "PrevDir" => Some(Action::PrevDir),
                                     "NextDir" => Some(Action::NextDir),
                                     "PrevPage" => Some(Action::PrevPage),
                                     "NextPage" => Some(Action::NextPage),
                                     "ToggleFullscreen" => Some(Action::ToggleFullscreen),
                                     "ZoomIn" => Some(Action::ZoomIn),
                                     "ZoomOut" => Some(Action::ZoomOut),
                                     "ResetZoom" => Some(Action::ResetZoom),
                                     "ToggleSpread" => Some(Action::ToggleSpread),
                                     "ToggleRTL" => Some(Action::ToggleRTL),
                                     "PrevPageSingle" => Some(Action::PrevPageSingle),
                                     "NextPageSingle" => Some(Action::NextPageSingle),
                                     _ => None,
                                 }
                             };
                             sender.input(MouseItemMsg::Change(action_opt));
                         }
                    }
                }
            }
        }
    }

    fn init_model((step_id, input_type, current, label_text, language): Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { step_id, input_type, current_setting: current, label_text, language }
    }

    fn update(&mut self, msg: MouseItemMsg, _sender: FactorySender<Self>) {
        match msg {
            MouseItemMsg::UpdateSetting(a) => self.current_setting = a,
            MouseItemMsg::Change(a) => {
                 let _ = _sender.output(SettingsDialogMsg::UpdateMouseBinding(self.input_type, a));
            }
        }
    }
}
