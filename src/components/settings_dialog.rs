
use relm4::prelude::*;
use gtk4::prelude::*;
use crate::database::{AppSettings, SortType};
use crate::input_settings::{Action, InputMap, InputSpec};
use gtk4::gdk;
use gtk4::glib::translate::{IntoGlib, FromGlib};

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
    fn label(&self) -> &'static str {
        match self {
            Self::RightClick => "Right Click",
            Self::MiddleClick => "Middle Click",
            Self::ScrollUp => "Scroll Up",
            Self::ScrollDown => "Scroll Down",
            Self::LeftDouble => "Left Double Click",
        }
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
            set_title: Some("Settings"),
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
                                    set_label: "Directory Defaults (Applied to new directories)",
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                },
                                
                                gtk4::CheckButton {
                                    set_label: Some("Default Spread View"),
                                    #[watch]
                                    set_active: model.default_spread_view,
                                    connect_toggled[sender] => move |btn| {
                                        sender.input(SettingsDialogMsg::UpdateDefaultSpread(btn.is_active()));
                                    }
                                },
                                
                                gtk4::CheckButton {
                                    set_label: Some("Default Right to Left"),
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
                                        set_label: "Default Dir Sort:",
                                    },
                                    
                                    gtk4::ComboBoxText {
                                        append: (Some("NameAsc"), "Name Asc"),
                                        append: (Some("NameDesc"), "Name Desc"),
                                        append: (Some("DateAsc"), "Date Asc"),
                                        append: (Some("DateDesc"), "Date Desc"),
                                        append: (Some("SizeAsc"), "Size Asc"),
                                        append: (Some("SizeDesc"), "Size Desc"),
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
                                        set_label: "Default Image Sort:",
                                    },
                                     gtk4::ComboBoxText {
                                        append: (Some("NameAsc"), "Name Asc"),
                                        append: (Some("NameDesc"), "Name Desc"),
                                        append: (Some("DateAsc"), "Date Asc"),
                                        append: (Some("DateDesc"), "Date Desc"),
                                        append: (Some("SizeAsc"), "Size Asc"),
                                        append: (Some("SizeDesc"), "Size Desc"),
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
                                    set_label: "Application Settings",
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                },
                                
                                gtk4::CheckButton {
                                    set_label: Some("Dark Mode (Requires Restart)"),
                                    #[watch]
                                    set_active: model.dark_mode,
                                    connect_toggled[sender] => move |btn| {
                                        sender.input(SettingsDialogMsg::UpdateDarkMode(btn.is_active()));
                                    }
                                },
                                
                                gtk4::CheckButton {
                                     set_label: Some("Loop Images (at end of list)"),
                                     #[watch]
                                     set_active: model.loop_images,
                                     connect_toggled[sender] => move |btn| {
                                         sender.input(SettingsDialogMsg::UpdateLoopImages(btn.is_active()));
                                     }
                                },
        
                                gtk4::CheckButton {
                                     set_label: Some("Single Page for First Image (Spread View)"),
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
                                        set_label: "Input Configuration",
                                        set_xalign: 0.0,
                                        set_hexpand: true,
                                        add_css_class: "title-4",
                                    },
                                    gtk4::Button {
                                        set_label: "Reset to Defaults",
                                        connect_clicked => SettingsDialogMsg::ResetInputs,
                                    }
                                },
                                
                                // Keyboard Section
                                gtk4::Label {
                                    set_label: "Keyboard Shortcuts",
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                    set_margin_top: 10,
                                },

                                gtk4::ListBox {
                                    set_selection_mode: gtk4::SelectionMode::None,
                                    add_css_class: "boxed-list",
                                    
                                    // PrevDir
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::PrevDir.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::PrevDir)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::PrevDir)); }
                                            }
                                        }
                                    },
                                    // NextDir
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::NextDir.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::NextDir)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::NextDir)); }
                                            }
                                        }
                                    },
                                    // PrevPage
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::PrevPage.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::PrevPage)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::PrevPage)); }
                                            }
                                        }
                                    },
                                    // NextPage
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::NextPage.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::NextPage)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::NextPage)); }
                                            }
                                        }
                                    },
                                    // ToggleFullscreen
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::ToggleFullscreen.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::ToggleFullscreen)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::ToggleFullscreen)); }
                                            }
                                        }
                                    },
                                    // ZoomIn
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::ZoomIn.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::ZoomIn)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::ZoomIn)); }
                                            }
                                        }
                                    },
                                    // ZoomOut
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::ZoomOut.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::ZoomOut)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::ZoomOut)); }
                                            }
                                        }
                                    },
                                    // ResetZoom
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::ResetZoom.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::ResetZoom)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::ResetZoom)); }
                                            }
                                        }
                                    },
                                    // ToggleSpread
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::ToggleSpread.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::ToggleSpread)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::ToggleSpread)); }
                                            }
                                        }
                                    },
                                    // ToggleRTL
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::ToggleRTL.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::ToggleRTL)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::ToggleRTL)); }
                                            }
                                        }
                                    },
                                    // PrevPageSingle
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::PrevPageSingle.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::PrevPageSingle)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::PrevPageSingle)); }
                                            }
                                        }
                                    },
                                    // NextPageSingle
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: Action::NextPageSingle.description(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::Button {
                                                #[watch]
                                                set_label: &format_keyboard_specs(model.input_map.map.get(&Action::NextPageSingle)),
                                                connect_clicked[sender] => move |_| { sender.input(SettingsDialogMsg::StartCapture(Action::NextPageSingle)); }
                                            }
                                        }
                                    },
                                },

                                // Mouse Section
                                gtk4::Label {
                                    set_label: "Mouse Configuration",
                                    set_xalign: 0.0,
                                    add_css_class: "title-4",
                                    set_margin_top: 10,
                                },

                                gtk4::ListBox {
                                    set_selection_mode: gtk4::SelectionMode::None,
                                    add_css_class: "boxed-list",
                                    
                                    // RightClick
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: MouseInputType::RightClick.label(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::ComboBoxText {
                                                append: (Some("None"), "None"),
                                                append: (Some("PrevDir"), Action::PrevDir.description()),
                                                append: (Some("NextDir"), Action::NextDir.description()),
                                                append: (Some("PrevPage"), Action::PrevPage.description()),
                                                append: (Some("NextPage"), Action::NextPage.description()),
                                                append: (Some("ToggleFullscreen"), Action::ToggleFullscreen.description()),
                                                append: (Some("ZoomIn"), Action::ZoomIn.description()),
                                                append: (Some("ZoomOut"), Action::ZoomOut.description()),
                                                append: (Some("ResetZoom"), Action::ResetZoom.description()),
                                                append: (Some("ToggleSpread"), Action::ToggleSpread.description()),
                                                append: (Some("ToggleRTL"), Action::ToggleRTL.description()),
                                                append: (Some("PrevPageSingle"), Action::PrevPageSingle.description()),
                                                append: (Some("NextPageSingle"), Action::NextPageSingle.description()),
                                                
                                                #[watch]
                                                set_active_id: Some(get_action_id_for_mouse(&model.input_map, MouseInputType::RightClick).unwrap_or("None".to_string()).as_str()),
                                                
                                                connect_changed[sender] => move |cb| {
                                                    if let Some(id) = cb.active_id() {
                                                        let action = match id.as_str() {
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
                                                        };
                                                        sender.input(SettingsDialogMsg::UpdateMouseBinding(MouseInputType::RightClick, action));
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    // MiddleClick
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: MouseInputType::MiddleClick.label(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::ComboBoxText {
                                                append: (Some("None"), "None"),
                                                append: (Some("PrevDir"), Action::PrevDir.description()),
                                                append: (Some("NextDir"), Action::NextDir.description()),
                                                append: (Some("PrevPage"), Action::PrevPage.description()),
                                                append: (Some("NextPage"), Action::NextPage.description()),
                                                append: (Some("ToggleFullscreen"), Action::ToggleFullscreen.description()),
                                                append: (Some("ZoomIn"), Action::ZoomIn.description()),
                                                append: (Some("ZoomOut"), Action::ZoomOut.description()),
                                                append: (Some("ResetZoom"), Action::ResetZoom.description()),
                                                append: (Some("ToggleSpread"), Action::ToggleSpread.description()),
                                                append: (Some("ToggleRTL"), Action::ToggleRTL.description()),
                                                append: (Some("PrevPageSingle"), Action::PrevPageSingle.description()),
                                                append: (Some("NextPageSingle"), Action::NextPageSingle.description()),
                                                
                                                #[watch]
                                                set_active_id: Some(get_action_id_for_mouse(&model.input_map, MouseInputType::MiddleClick).unwrap_or("None".to_string()).as_str()),
                                                
                                                connect_changed[sender] => move |cb| {
                                                    if let Some(id) = cb.active_id() {
                                                        let action = match id.as_str() {
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
                                                        };
                                                        sender.input(SettingsDialogMsg::UpdateMouseBinding(MouseInputType::MiddleClick, action));
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    // ScrollUp
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: MouseInputType::ScrollUp.label(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::ComboBoxText {
                                                append: (Some("None"), "None"),
                                                append: (Some("PrevDir"), Action::PrevDir.description()),
                                                append: (Some("NextDir"), Action::NextDir.description()),
                                                append: (Some("PrevPage"), Action::PrevPage.description()),
                                                append: (Some("NextPage"), Action::NextPage.description()),
                                                append: (Some("ToggleFullscreen"), Action::ToggleFullscreen.description()),
                                                append: (Some("ZoomIn"), Action::ZoomIn.description()),
                                                append: (Some("ZoomOut"), Action::ZoomOut.description()),
                                                append: (Some("ResetZoom"), Action::ResetZoom.description()),
                                                append: (Some("ToggleSpread"), Action::ToggleSpread.description()),
                                                append: (Some("ToggleRTL"), Action::ToggleRTL.description()),
                                                append: (Some("PrevPageSingle"), Action::PrevPageSingle.description()),
                                                append: (Some("NextPageSingle"), Action::NextPageSingle.description()),
                                                
                                                #[watch]
                                                set_active_id: Some(get_action_id_for_mouse(&model.input_map, MouseInputType::ScrollUp).unwrap_or("None".to_string()).as_str()),
                                                
                                                connect_changed[sender] => move |cb| {
                                                    if let Some(id) = cb.active_id() {
                                                        let action = match id.as_str() {
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
                                                        };
                                                        sender.input(SettingsDialogMsg::UpdateMouseBinding(MouseInputType::ScrollUp, action));
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    // ScrollDown
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: MouseInputType::ScrollDown.label(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::ComboBoxText {
                                                append: (Some("None"), "None"),
                                                append: (Some("PrevDir"), Action::PrevDir.description()),
                                                append: (Some("NextDir"), Action::NextDir.description()),
                                                append: (Some("PrevPage"), Action::PrevPage.description()),
                                                append: (Some("NextPage"), Action::NextPage.description()),
                                                append: (Some("ToggleFullscreen"), Action::ToggleFullscreen.description()),
                                                append: (Some("ZoomIn"), Action::ZoomIn.description()),
                                                append: (Some("ZoomOut"), Action::ZoomOut.description()),
                                                append: (Some("ResetZoom"), Action::ResetZoom.description()),
                                                append: (Some("ToggleSpread"), Action::ToggleSpread.description()),
                                                append: (Some("ToggleRTL"), Action::ToggleRTL.description()),
                                                append: (Some("PrevPageSingle"), Action::PrevPageSingle.description()),
                                                append: (Some("NextPageSingle"), Action::NextPageSingle.description()),
                                                
                                                #[watch]
                                                set_active_id: Some(get_action_id_for_mouse(&model.input_map, MouseInputType::ScrollDown).unwrap_or("None".to_string()).as_str()),
                                                
                                                connect_changed[sender] => move |cb| {
                                                    if let Some(id) = cb.active_id() {
                                                        let action = match id.as_str() {
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
                                                        };
                                                        sender.input(SettingsDialogMsg::UpdateMouseBinding(MouseInputType::ScrollDown, action));
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    // LeftDouble
                                    gtk4::ListBoxRow {
                                        gtk4::Box {
                                            set_orientation: gtk4::Orientation::Horizontal,
                                            set_spacing: 10,
                                            set_margin_all: 5,
                                            gtk4::Label {
                                                set_label: MouseInputType::LeftDouble.label(),
                                                set_hexpand: true,
                                                set_xalign: 0.0,
                                            },
                                            gtk4::ComboBoxText {
                                                append: (Some("None"), "None"),
                                                append: (Some("PrevDir"), Action::PrevDir.description()),
                                                append: (Some("NextDir"), Action::NextDir.description()),
                                                append: (Some("PrevPage"), Action::PrevPage.description()),
                                                append: (Some("NextPage"), Action::NextPage.description()),
                                                append: (Some("ToggleFullscreen"), Action::ToggleFullscreen.description()),
                                                append: (Some("ZoomIn"), Action::ZoomIn.description()),
                                                append: (Some("ZoomOut"), Action::ZoomOut.description()),
                                                append: (Some("ResetZoom"), Action::ResetZoom.description()),
                                                append: (Some("ToggleSpread"), Action::ToggleSpread.description()),
                                                append: (Some("ToggleRTL"), Action::ToggleRTL.description()),
                                                append: (Some("PrevPageSingle"), Action::PrevPageSingle.description()),
                                                append: (Some("NextPageSingle"), Action::NextPageSingle.description()),
                                                
                                                #[watch]
                                                set_active_id: Some(get_action_id_for_mouse(&model.input_map, MouseInputType::LeftDouble).unwrap_or("None".to_string()).as_str()),
                                                
                                                connect_changed[sender] => move |cb| {
                                                    if let Some(id) = cb.active_id() {
                                                        let action = match id.as_str() {
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
                                                        };
                                                        sender.input(SettingsDialogMsg::UpdateMouseBinding(MouseInputType::LeftDouble, action));
                                                    }
                                                }
                                            }
                                        }
                                    },
                            }
                        },
                    },
                        
                        // Footer
                        gtk4::Box {
                            set_orientation: gtk4::Orientation::Horizontal,
                            set_spacing: 10,
                            set_halign: gtk4::Align::End,
                            set_margin_all: 10,
                            
                            gtk4::Button {
                                set_label: "Cancel",
                                connect_clicked => SettingsDialogMsg::Close,
                            },
                            
                            gtk4::Button {
                                set_label: "Save",
                                add_css_class: "suggested-action",
                                connect_clicked => SettingsDialogMsg::Save,
                            }
                        }
                    }
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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
        };
        let widgets = view_output!();
        
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
        
        // Background shim to block clicks?
        let bg = gtk4::Box::builder()
            .css_classes(["view", "frame"]) // Just some classes to make it opaque-ish? Overlay usually handles this manually if needed.
            // Actually, we want to capture mouse clicks too.
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
            // Check for Double Click
            // Since double click comes after single click, we might need to be smart.
            // However, InputSpec differentiates double click bool.
            if n_press == 2 {
                sender_clone.input(SettingsDialogMsg::CaptureInput(InputSpec::Mouse {
                    button,
                    modifiers: modifiers.bits(),
                    double_click: true,
                }));
            } else if n_press == 1 {
                 // Delay slightly? Or just register single click.
                 // For now, register single click immediately.
                 // A timer logic would be needed to distinguish single/double reliably if we want both on same button.
                 // Just send single click for now.
                 // If user double clicks, they trigger single capture first, then capture closes.
                 // Wait, if capture closes on first input, double click is hard to capture.
                 // Maybe we accept just single click for now, OR we need a "Wait" logic.
                 // Let's assume user intends single click unless they are fast?
                 // Actually Gtk GestureClick handles n_press.
                 sender_clone.input(SettingsDialogMsg::CaptureInput(InputSpec::Mouse {
                    button,
                    modifiers: modifiers.bits(),
                    double_click: false,
                }));
            }
        });
        // We need scroll capture too
        let scroll_controller = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE);
        let sender_clone_scroll = sender.clone();
        scroll_controller.connect_scroll(move |_, _dx, dy| {
            if dy < 0.0 {
                 sender_clone_scroll.input(SettingsDialogMsg::CaptureInput(InputSpec::Scroll {
                    direction: crate::input_settings::ScrollDirection::Up,
                    modifiers: gtk4::gdk::ModifierType::empty().bits(), // Modifiers hard to get here directly?
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
        // Initially hidden
        bg.set_visible(false);
        // Dirty hack: save widget to model to toggle visibility? 
        // No, we can use binding or just manual toggle in update if we store widget ref.
        // We can't store widget ref in model easily with Relm4 SimpleComponent (model is separated).
        // Use a "capturing" boolean in model and #[watch] set_visible on the overlay child.
        // But the child is manually added.
        // Let's rely on relm4 component update logic modifying the widget via 'widgets' struct if exposed?
        // SimpleComponent doesn't expose widgets in update. 
        // We have to move this overlay child into the view! macro.
        
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
                self.capturing_action = None;
                self.is_active = true;
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
                };
                let _ = _sender.output(SettingsDialogOutput::SaveSettings(new_settings));
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
                    // Check cancellation
                     if let InputSpec::Keyboard { keyval, .. } = spec {
                        if keyval == gtk4::gdk::Key::Escape.into_glib() {
                             self.capturing_action = None;
                             return;
                        }
                    }
                    
                    // We only accept Keyboard input for "CaptureInput" now
                    if matches!(spec, InputSpec::Keyboard { .. }) {
                        // preserve existing mouse bindings for this action?
                        // Actually, we want to allow multiple Keyboard bindings?
                        // Or just replace all for this action?
                        // The UI implies 1-to-1 or just "Set".
                        // To be safe and simple: 
                        // 1. Get existing specs for this action.
                        // 2. Filter out Keyboard specs.
                        // 3. Add new Keyboard spec.
                        // This prevents clearing mouse bindings when setting keyboard.
                        
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
                    }
                    self.capturing_action = None;
                }
            }
            SettingsDialogMsg::UpdateMouseBinding(input_type, action_opt) => {
                // Remove this mouse input from ALL actions
                 for (_, specs) in self.input_map.map.iter_mut() {
                     specs.retain(|s| !matches_mouse_input(s, input_type));
                 }
                 
                 // If action is selected, add it
                 if let Some(action) = action_opt {
                     let mut specs = self.input_map.map.entry(action).or_insert(Vec::new()).clone();
                     specs.push(input_type.to_input_spec());
                     self.input_map.map.insert(action, specs);
                 }
            }
            SettingsDialogMsg::ResetInputs => {
                self.input_map = InputMap::default();
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

fn get_action_id_for_mouse(input_map: &InputMap, input_type: MouseInputType) -> Option<String> {
    for (action, specs) in &input_map.map {
        for spec in specs {
            if matches_mouse_input(spec, input_type) {
                return Some(format!("{:?}", action));
            }
        }
    }
    None
}
