use relm4::prelude::*;
use gtk4::prelude::*;
use std::path::PathBuf;
use crate::database::SortType;
use crate::input_settings::{InputMap, Action, ScrollDirection};
use crate::i18n::Language;

#[derive(Debug)]
pub enum LoadedImageSource {
    TextureBytes(Vec<u8>),
    AnimPath(PathBuf),
    AnimTemp(tempfile::NamedTempFile),
    Error,
}

#[derive(Debug)]
pub struct ImageViewModel {
    current_paths: Vec<PathBuf>,
    textures_even: Vec<gtk4::gdk::Paintable>,
    textures_odd: Vec<gtk4::gdk::Paintable>,
    zoom: f64,
    is_fit_to_window: bool,
    pub spread_mode: bool,
    pub right_to_left: bool,
    pub dir_sort: SortType,
    pub image_sort: SortType,
    pub is_fullscreen: bool,
    temp_files_even: Vec<tempfile::NamedTempFile>,
    temp_files_odd: Vec<tempfile::NamedTempFile>,
    pub input_map: InputMap,
    pub language: Language,
    generation: u32,
    visible_generation: u32,
    viewport_size: (f64, f64),
}

#[derive(Debug)]
pub enum ImageViewMsg {
    ShowPages(Vec<PathBuf>),
    ZoomIn,
    ZoomOut,
    ResetZoom,
    UpdateSettings { spread_mode: bool, right_to_left: bool, dir_sort: SortType, image_sort: SortType, input_map: InputMap, language: Language },
    ChangeDirSort(SortType),
    ChangeImageSort(SortType),
    ToggleSpread,
    ToggleDirection,
    UpdateFullscreen(bool),
    LoadFallback(usize, PathBuf, Option<PathBuf>, u32),
    TriggerAction(Action),
    MouseInput { button: u32, modifiers: u32, n_press: i32 },
    ScrollInput { dy: f64, modifiers: u32 },
    ImageLoaded { index: usize, source: LoadedImageSource, path: PathBuf, generation: u32 },
    ViewportResized(f64, f64),
}

#[derive(Debug)]
pub enum ImageViewOutput {
    DirSortChanged(SortType),
    ImageSortChanged(SortType),
    SpreadModeChanged(bool),
    RTLChanged(bool),
    TriggerAction(Action),
}

#[relm4::component(pub)]
impl SimpleComponent for ImageViewModel {
    type Input = ImageViewMsg;
    type Output = ImageViewOutput;
    type Init = ();

    view! {
        #[root]
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            
            // Settings Header
            gtk4::ScrolledWindow {
                set_hscrollbar_policy: gtk4::PolicyType::Automatic,
                set_vscrollbar_policy: gtk4::PolicyType::Never,
                set_hexpand: true,
                #[watch]
                set_visible: !model.is_fullscreen,

                #[wrap(Some)]
                set_child = &gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_spacing: 10,
                    set_margin_all: 5,
                    
                    gtk4::Button {
                        #[watch]
                        set_tooltip_text: Some(&crate::i18n::localize("Toggle Spread View", model.language)),
                        set_focusable: false,
                        #[wrap(Some)]
                        set_child: spread_icon = &gtk4::Image {
                            #[watch]
                            set_paintable: (if model.spread_mode { crate::icon::spread_on() } else { crate::icon::spread_off() }).as_ref(),
                            set_pixel_size: 24,
                        },
                        connect_clicked[sender] => move |_| {
                            sender.input(ImageViewMsg::ToggleSpread);
                        }
                    },
                    
                    gtk4::Button {
                        #[watch]
                        set_tooltip_text: Some(&crate::i18n::localize("Toggle Right-to-Left", model.language)),
                        set_focusable: false,
                        #[wrap(Some)]
                        set_child: rtl_icon = &gtk4::Image {
                            #[watch]
                            set_paintable: (if model.right_to_left { crate::icon::binding_right() } else { crate::icon::binding_left() }).as_ref(), // Use .as_ref() for Option<&P>
                            set_pixel_size: 24,
                        },
                        connect_clicked[sender] => move |_| {
                            sender.input(ImageViewMsg::ToggleDirection);
                        }
                    },
                    
                    gtk4::Separator {},
                    
                    gtk4::Label {
                        set_label: "Dir Sort:",
                    },
                    
                    gtk4::DropDown {
                        set_model: Some(&gtk4::StringList::new(&[
                            "Name Asc", "Name Desc", "Date Asc", "Date Desc", "Size Asc", "Size Desc"
                        ])),
                        set_focusable: false,
                        #[watch]
                        set_selected: match model.dir_sort {
                            SortType::NameAsc => 0,
                            SortType::NameDesc => 1,
                            SortType::DateAsc => 2,
                            SortType::DateDesc => 3,
                            SortType::SizeAsc => 4,
                            SortType::SizeDesc => 5,
                        },
                        connect_selected_notify[sender] => move |dd| {
                            let sort = match dd.selected() {
                                0 => SortType::NameAsc,
                                1 => SortType::NameDesc,
                                2 => SortType::DateAsc,
                                3 => SortType::DateDesc,
                                4 => SortType::SizeAsc,
                                5 => SortType::SizeDesc,
                                _ => SortType::NameAsc,
                            };
                             sender.input(ImageViewMsg::ChangeDirSort(sort));
                             // Clear focus to return control to the window/view
                             if let Some(root) = dd.root() {
                                if let Ok(window) = root.downcast::<gtk4::Window>() {
                                    gtk4::prelude::GtkWindowExt::set_focus(&window, None::<&gtk4::Widget>);
                                }
                            }
                        }
                    },
                    
                    gtk4::Label {
                        set_label: "Img Sort:",
                    },
                    
                    gtk4::DropDown {
                        set_model: Some(&gtk4::StringList::new(&[
                            "Name Asc", "Name Desc", "Date Asc", "Date Desc", "Size Asc", "Size Desc"
                        ])),
                        set_focusable: false,
                        #[watch]
                        set_selected: match model.image_sort {
                            SortType::NameAsc => 0,
                            SortType::NameDesc => 1,
                            SortType::DateAsc => 2,
                            SortType::DateDesc => 3,
                            SortType::SizeAsc => 4,
                            SortType::SizeDesc => 5,
                        },
                        connect_selected_notify[sender] => move |dd| {
                             let sort = match dd.selected() {
                                0 => SortType::NameAsc,
                                1 => SortType::NameDesc,
                                2 => SortType::DateAsc,
                                3 => SortType::DateDesc,
                                4 => SortType::SizeAsc,
                                5 => SortType::SizeDesc,
                                _ => SortType::NameAsc,
                            };
                            sender.input(ImageViewMsg::ChangeImageSort(sort));
                             // Clear focus to return control to the window/view
                             if let Some(root) = dd.root() {
                                if let Ok(window) = root.downcast::<gtk4::Window>() {
                                    gtk4::prelude::GtkWindowExt::set_focus(&window, None::<&gtk4::Widget>);
                                }
                            }
                        }
                    },
                },
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                add_css_class: "image-view-background", 
            
                #[name(main_stack)]
                gtk4::Stack {
                    set_transition_type: gtk4::StackTransitionType::None,
                    #[watch]
                    set_visible_child_name: if model.visible_generation % 2 == 0 { "even" } else { "odd" },
                    
                    add_named[Some("even")] = &gtk4::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        #[watch]
                        set_hscrollbar_policy: if model.is_fit_to_window { gtk4::PolicyType::Never } else { gtk4::PolicyType::Automatic },
                        #[watch]
                        set_vscrollbar_policy: if model.is_fit_to_window { gtk4::PolicyType::Never } else { gtk4::PolicyType::Automatic },
                        
                        add_controller = gtk4::GestureClick {
                            set_button: 0, 
                            connect_pressed[sender] => move |gesture, n_press, _, _| {
                                let button = gesture.current_button();
                                let modifiers = gesture.current_event_state().bits();
                                sender.input(ImageViewMsg::MouseInput { button, modifiers, n_press });
                            }
                        },

                        add_controller = gtk4::EventControllerScroll {
                            set_flags: gtk4::EventControllerScrollFlags::VERTICAL,
                            connect_scroll[sender] => move |controller, _dx, dy| {
                                 let modifiers = controller.current_event_state().bits();
                                 sender.input(ImageViewMsg::ScrollInput { dy, modifiers }); 
                                 gtk4::glib::Propagation::Stop
                            }
                        },

                        add_controller = gtk4::GestureDrag {
                            set_button: gtk4::gdk::BUTTON_PRIMARY,
                            connect_drag_begin[drag_state] => move |gesture, _, _| {
                                if let Some(widget) = gesture.widget() {
                                     if let Some(sw) = widget.downcast_ref::<gtk4::ScrolledWindow>() {
                                         let h = sw.hadjustment().value();
                                         let v = sw.vadjustment().value();
                                         *drag_state.borrow_mut() = (h, v);
                                     }
                                }
                            },
                            connect_drag_update[drag_state_2] => move |gesture, offset_x, offset_y| {
                                if let Some(widget) = gesture.widget() {
                                     if let Some(sw) = widget.downcast_ref::<gtk4::ScrolledWindow>() {
                                         let (start_h, start_v) = *drag_state_2.borrow();
                                         sw.hadjustment().set_value(start_h - offset_x);
                                         sw.vadjustment().set_value(start_v - offset_y);
                                     }
                                }
                            }
                        },
                        
                        #[wrap(Some)]
                        set_child = &gtk4::CenterBox {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: true,

                            #[wrap(Some)]
                            set_center_widget = &gtk4::Box {
                                set_orientation: gtk4::Orientation::Horizontal,
                                #[watch]
                                set_halign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                #[watch]
                                set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                #[watch]
                                set_homogeneous: model.spread_mode && model.is_fit_to_window,
                                set_spacing: 0,
                                
                                append = &gtk4::Picture {
                                    #[watch]
                                    set_halign: if model.is_fit_to_window { 
                                        if model.spread_mode && model.textures_even.len() > 1 { gtk4::Align::End } else { gtk4::Align::Fill }
                                    } else { gtk4::Align::Center },
                                    #[watch]
                                    set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                    #[watch]
                                    set_hexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_vexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_paintable: if model.spread_mode && model.textures_even.len() > 1 {
                                         if model.right_to_left { model.textures_even.get(1) } else { model.textures_even.get(0) }
                                    } else {
                                         model.textures_even.get(0)
                                    },
                                    #[watch]
                                    set_can_shrink: true,
                                    #[watch]
                                    set_width_request: if model.is_fit_to_window { -1 } else { 
                                        let idx = if model.spread_mode && model.textures_even.len() > 1 && model.right_to_left { 1 } else { 0 };
                                        model.textures_even.get(idx).map_or(-1, |t| (t.intrinsic_width() as f64 * model.zoom) as i32) 
                                    },
                                    #[watch]
                                    set_height_request: if model.is_fit_to_window { -1 } else { 
                                         let idx = if model.spread_mode && model.textures_even.len() > 1 && model.right_to_left { 1 } else { 0 };
                                         model.textures_even.get(idx).map_or(-1, |t| (t.intrinsic_height() as f64 * model.zoom) as i32)
                                    },
                                    #[watch]
                                    set_visible: if model.spread_mode && model.textures_even.len() > 1 {
                                         true 
                                    } else {
                                         !model.textures_even.is_empty()
                                    }
                                },
                                
                                append = &gtk4::Picture {
                                    #[watch]
                                    set_halign: if model.is_fit_to_window { 
                                        if model.spread_mode { gtk4::Align::Start } else { gtk4::Align::Fill }
                                    } else { gtk4::Align::Center },
                                    #[watch]
                                    set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                    #[watch]
                                    set_hexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_vexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_paintable: if model.spread_mode && model.textures_even.len() > 1 {
                                         if model.right_to_left { model.textures_even.get(0) } else { model.textures_even.get(1) }
                                    } else {
                                         None
                                    },
                                    #[watch]
                                    set_can_shrink: true,
                                    #[watch]
                                    set_width_request: if model.is_fit_to_window { -1 } else { 
                                        let idx = if model.spread_mode && model.textures_even.len() > 1 && model.right_to_left { 0 } else { 1 };
                                        model.textures_even.get(idx).map_or(-1, |t| (t.intrinsic_width() as f64 * model.zoom) as i32) 
                                    },
                                    #[watch]
                                    set_height_request: if model.is_fit_to_window { -1 } else { 
                                         let idx = if model.spread_mode && model.textures_even.len() > 1 && model.right_to_left { 0 } else { 1 };
                                         model.textures_even.get(idx).map_or(-1, |t| (t.intrinsic_height() as f64 * model.zoom) as i32)
                                    },
                                    #[watch]
                                    set_visible: model.spread_mode && model.textures_even.len() > 1,
                                }
                            }
                        }
                    },


                    add_named[Some("odd")] = &gtk4::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        #[watch]
                        set_hscrollbar_policy: if model.is_fit_to_window { gtk4::PolicyType::Never } else { gtk4::PolicyType::Automatic },
                        #[watch]
                        set_vscrollbar_policy: if model.is_fit_to_window { gtk4::PolicyType::Never } else { gtk4::PolicyType::Automatic },
                        
                        add_controller = gtk4::GestureClick {
                            set_button: 0, 
                            connect_pressed[sender] => move |gesture, n_press, _, _| {
                                let button = gesture.current_button();
                                let modifiers = gesture.current_event_state().bits();
                                sender.input(ImageViewMsg::MouseInput { button, modifiers, n_press });
                            }
                        },

                        add_controller = gtk4::EventControllerScroll {
                            set_flags: gtk4::EventControllerScrollFlags::VERTICAL,
                            connect_scroll[sender] => move |controller, _dx, dy| {
                                 let modifiers = controller.current_event_state().bits();
                                 sender.input(ImageViewMsg::ScrollInput { dy, modifiers }); 
                                 gtk4::glib::Propagation::Stop
                            }
                        },

                        add_controller = gtk4::GestureDrag {
                            set_button: gtk4::gdk::BUTTON_PRIMARY,
                            connect_drag_begin[drag_state_odd] => move |gesture, _, _| {
                                if let Some(widget) = gesture.widget() {
                                     if let Some(sw) = widget.downcast_ref::<gtk4::ScrolledWindow>() {
                                         let h = sw.hadjustment().value();
                                         let v = sw.vadjustment().value();
                                         *drag_state_odd.borrow_mut() = (h, v);
                                     }
                                }
                            },
                            connect_drag_update[drag_state_odd_2] => move |gesture, offset_x, offset_y| {
                                if let Some(widget) = gesture.widget() {
                                     if let Some(sw) = widget.downcast_ref::<gtk4::ScrolledWindow>() {
                                         let (start_h, start_v) = *drag_state_odd_2.borrow();
                                         sw.hadjustment().set_value(start_h - offset_x);
                                         sw.vadjustment().set_value(start_v - offset_y);
                                     }
                                }
                            }
                        },
                        
                        #[wrap(Some)]
                        set_child = &gtk4::CenterBox {
                            set_orientation: gtk4::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: true,

                            #[wrap(Some)]
                            set_center_widget = &gtk4::Box {
                                set_orientation: gtk4::Orientation::Horizontal,
                                #[watch]
                                set_halign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                #[watch]
                                set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                #[watch]
                                set_homogeneous: model.spread_mode && model.is_fit_to_window,
                                set_spacing: 0,
                                
                                append = &gtk4::Picture {
                                    #[watch]
                                    set_halign: if model.is_fit_to_window { 
                                        if model.spread_mode && model.textures_odd.len() > 1 { gtk4::Align::End } else { gtk4::Align::Fill }
                                    } else { gtk4::Align::Center },
                                    #[watch]
                                    set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                    #[watch]
                                    set_hexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_vexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_paintable: if model.spread_mode && model.textures_odd.len() > 1 {
                                         if model.right_to_left { model.textures_odd.get(1) } else { model.textures_odd.get(0) }
                                    } else {
                                         model.textures_odd.get(0)
                                    },
                                    #[watch]
                                    set_can_shrink: true,
                                    #[watch]
                                    set_width_request: if model.is_fit_to_window { -1 } else { 
                                        let idx = if model.spread_mode && model.textures_odd.len() > 1 && model.right_to_left { 1 } else { 0 };
                                        model.textures_odd.get(idx).map_or(-1, |t| (t.intrinsic_width() as f64 * model.zoom) as i32) 
                                    },
                                    #[watch]
                                    set_height_request: if model.is_fit_to_window { -1 } else { 
                                         let idx = if model.spread_mode && model.textures_odd.len() > 1 && model.right_to_left { 1 } else { 0 };
                                         model.textures_odd.get(idx).map_or(-1, |t| (t.intrinsic_height() as f64 * model.zoom) as i32)
                                    },
                                    #[watch]
                                    set_visible: if model.spread_mode && model.textures_odd.len() > 1 {
                                         true 
                                    } else {
                                         !model.textures_odd.is_empty()
                                    }
                                },
                                
                                append = &gtk4::Picture {
                                    #[watch]
                                    set_halign: if model.is_fit_to_window { 
                                        if model.spread_mode { gtk4::Align::Start } else { gtk4::Align::Fill }
                                    } else { gtk4::Align::Center },
                                    #[watch]
                                    set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                    #[watch]
                                    set_hexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_vexpand: model.is_fit_to_window,
                                    #[watch]
                                    set_paintable: if model.spread_mode && model.textures_odd.len() > 1 {
                                         if model.right_to_left { model.textures_odd.get(0) } else { model.textures_odd.get(1) }
                                    } else {
                                         None
                                    },
                                    #[watch]
                                    set_can_shrink: true,
                                    #[watch]
                                    set_width_request: if model.is_fit_to_window { -1 } else { 
                                        let idx = if model.spread_mode && model.textures_odd.len() > 1 && model.right_to_left { 0 } else { 1 };
                                        model.textures_odd.get(idx).map_or(-1, |t| (t.intrinsic_width() as f64 * model.zoom) as i32) 
                                    },
                                    #[watch]
                                    set_height_request: if model.is_fit_to_window { -1 } else { 
                                         let idx = if model.spread_mode && model.textures_odd.len() > 1 && model.right_to_left { 0 } else { 1 };
                                         model.textures_odd.get(idx).map_or(-1, |t| (t.intrinsic_height() as f64 * model.zoom) as i32)
                                    },
                                    #[watch]
                                    set_visible: model.spread_mode && model.textures_odd.len() > 1,
                                }
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
        let model = ImageViewModel {
            current_paths: Vec::new(),
            textures_even: Vec::new(),
            textures_odd: Vec::new(),
            zoom: 1.0,
            is_fit_to_window: true,
            spread_mode: false,
            right_to_left: true,
            dir_sort: SortType::NameAsc,
            image_sort: SortType::NameAsc,
            is_fullscreen: false,
            temp_files_even: Vec::new(),
            temp_files_odd: Vec::new(),
            input_map: InputMap::default(),
            language: Language::default(),
            generation: 0,
            visible_generation: 0,
            viewport_size: (0.0, 0.0),
        };
        
        let drag_state = std::rc::Rc::new(std::cell::RefCell::new((0.0, 0.0)));
        let drag_state_2 = drag_state.clone();
        
        // Need separate drag states for odd stack page since gesture controller is unique instance
        let drag_state_odd = std::rc::Rc::new(std::cell::RefCell::new((0.0, 0.0)));
        let drag_state_odd_2 = drag_state_odd.clone();
        
        let widgets = view_output!();
        {
            let sender = sender.clone();
            let mut child = widgets.main_stack.first_child();
            while let Some(widget) = child {
                if let Some(sw) = widget.downcast_ref::<gtk4::ScrolledWindow>() {
                    let sender_clone = sender.clone();
                    let sw_clone = sw.clone();
                    sw.hadjustment().connect_notify_local(Some("page-size"), move |_, _| {
                        let w = sw_clone.hadjustment().page_size();
                        let h = sw_clone.vadjustment().page_size();
                        sender_clone.input(ImageViewMsg::ViewportResized(w, h));
                    });

                    let sender_clone = sender.clone();
                    let sw_clone = sw.clone();
                    sw.vadjustment().connect_notify_local(Some("page-size"), move |_, _| {
                        let w = sw_clone.hadjustment().page_size();
                        let h = sw_clone.vadjustment().page_size();
                        sender_clone.input(ImageViewMsg::ViewportResized(w, h));
                    });
                }
                child = widget.next_sibling();
            }
        }
        
        // Manual fix to properly get scroll modifiers involves getting the controller from widgets
        // For now, simpler implementation is accepted.
        
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
               ImageViewMsg::ShowPages(paths) => {
                   self.current_paths = paths.clone();
                   self.is_fit_to_window = true;
                   
                   // Increment generation for new request
                   self.generation += 1;
                   let current_gen = self.generation;
                   
                   // Clear the buffer we are about to load into (the one NOT visible ideally, but here determined by generation)
                   // If visible_generation == generation - 1, then visible_generation % 2 != generation % 2
                   // So we are safe to clear generation % 2
                   let is_even = current_gen % 2 == 0;
                   if is_even {
                       self.textures_even.clear();
                       self.temp_files_even.clear();
                   } else {
                       self.textures_odd.clear();
                       self.temp_files_odd.clear();
                   }
                   
                   if paths.is_empty() {
                       self.visible_generation = current_gen;
                       return;
                   }

                   let sender_clone = _sender.clone();
                   let paths_clone = paths.clone();

                   std::thread::spawn(move || {
                       for (index, path) in paths_clone.iter().enumerate() {
                            let mut found_source = LoadedImageSource::Error;
                            
                            if path.exists() {
                                // ... loading logic ...
                                let is_anim = path.extension().and_then(|s| s.to_str()).map_or(false, |ext| {
                                    let ext = ext.to_lowercase();
                                    if ext == "gif" || ext == "apng" { return true; }
                                    if ext == "webp" { return crate::utils::is_animated_webp(path); }
                                    if ext == "png" { return crate::utils::is_apng(path); }
                                    false
                                });

                                if is_anim {
                                     found_source = LoadedImageSource::AnimPath(path.clone());
                                } else {
                                    if let Ok(bytes) = std::fs::read(path) {
                                        found_source = LoadedImageSource::TextureBytes(bytes);
                                    }
                                }
                            }

                            if matches!(found_source, LoadedImageSource::Error) {
                                // Zip logic ...
                                // (Omitting full Zip logic copy for brevity if unchanged, but I must provide full replacement? 
                                //  Wait, multi_replace replaces chunk. I should keep the ZIP logic or copy it.
                                //  Since I can't put "..." in implementation, I have to copy it.
                                //  Fortunately, the ZIP logic inside thread is stateless regarding self.
                                //  So it matches the existing logic.)
                                
                                let mut zip_found = false;
                                let mut check_path = path.clone();
                                while let Some(parent) = check_path.parent() {
                                     if parent.is_file() {
                                         if let Some(ext) = parent.extension().and_then(|s| s.to_str()) {
                                             if ext.to_lowercase() == "zip" {
                                                 if let Ok(suffix) = path.strip_prefix(parent) {
                                                     let entry_name = suffix.to_string_lossy();
                                                     if let Ok(file) = std::fs::File::open(parent) {
                                                         if let Ok(mut archive) = zip::ZipArchive::new(file) {
                                                             if let Ok(mut entry) = archive.by_name(&entry_name) {
                                                                 use std::io::Read;
                                                                 let mut buffer = Vec::new();
                                                                 if entry.read_to_end(&mut buffer).is_ok() {
                                                                     let glib_bytes = gtk4::glib::Bytes::from(&buffer);
                                                                     let is_anim = path.extension().and_then(|s| s.to_str()).map_or(false, |ext| {
                                                                         let ext = ext.to_lowercase();
                                                                         if ext == "gif" || ext == "apng" { return true; }
                                                                         if ext == "webp" { return crate::utils::is_animated_webp_bytes(&glib_bytes); }
                                                                         if ext == "png" { return crate::utils::is_apng_bytes(&glib_bytes); }
                                                                         false
                                                                     });

                                                                     if is_anim {
                                                                         let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("tmp");
                                                                         if let Ok(mut temp_file) = tempfile::Builder::new().suffix(&format!(".{}", ext)).tempfile() {
                                                                             use std::io::Write;
                                                                             if temp_file.write_all(&buffer).is_ok() {
                                                                                 found_source = LoadedImageSource::AnimTemp(temp_file);
                                                                             }
                                                                         }
                                                                     } else {
                                                                         found_source = LoadedImageSource::TextureBytes(buffer);
                                                                     }
                                                                     zip_found = true;
                                                                 }
                                                             }
                                                         }
                                                     }
                                                 }
                                             }
                                         }
                                     }
                                     if zip_found { break; }
                                     check_path = parent.to_path_buf();
                                }
                            }
                            
                            sender_clone.input(ImageViewMsg::ImageLoaded { index, source: found_source, path: path.clone(), generation: current_gen });
                       }
                   });
               }
              ImageViewMsg::ImageLoaded { index, source, path, generation } => {
                  if generation != self.generation {
                      return;
                  }
                  
                  let is_even = generation % 2 == 0;

                  // Helper to push to correct vector
                  let (textures, temp_files) = if is_even {
                      (&mut self.textures_even, &mut self.temp_files_even)
                  } else {
                      (&mut self.textures_odd, &mut self.temp_files_odd)
                  };

                  match source {
                      LoadedImageSource::TextureBytes(bytes) => {
                          let glib_bytes = gtk4::glib::Bytes::from(&bytes);
                          if let Ok(texture) = gtk4::gdk::Texture::from_bytes(&glib_bytes) {
                              textures.push(texture.upcast());
                          }
                      }
                      LoadedImageSource::AnimPath(p) => {
                           let file = gtk4::gio::File::for_path(&p);
                           let media = gtk4::MediaFile::for_file(&file);
                           media.set_loop(true);
                           media.play();
                           let sender_clone = _sender.clone();
                           let idx = textures.len();
                           let p_clone = p.clone();
                           let gen = generation;
                           media.connect_notify(Some("error"), move |_, _| {
                               sender_clone.input(ImageViewMsg::LoadFallback(idx, p_clone.clone(), None, gen));
                           });
                           textures.push(media.upcast());
                      }
                      LoadedImageSource::AnimTemp(temp_file) => {
                           let temp_path = temp_file.path().to_path_buf();
                           let media = gtk4::MediaFile::for_filename(temp_path.to_str().unwrap_or(""));
                           media.set_loop(true);
                           media.play();
                           
                           let sender_clone = _sender.clone();
                           let idx = textures.len();
                           let p_clone = path.clone();
                           let temp_path_clone = temp_path.clone();
                           let gen = generation;
                           media.connect_notify(Some("error"), move |_, _| {
                               sender_clone.input(ImageViewMsg::LoadFallback(idx, p_clone.clone(), Some(temp_path_clone.clone()), gen));
                           });
                           
                           textures.push(media.upcast());
                           temp_files.push(temp_file);
                      }
                      LoadedImageSource::Error => {
                          if path.exists() {
                                if let Ok(texture) = gtk4::gdk::Texture::from_file(&gtk4::gio::File::for_path(&path)) {
                                    textures.push(texture.upcast());
                                }
                          }
                      }
                  }
                  
                  // Wait until we have loaded all expected images for this generation
                  // before we show them. This prevents flicker in spread view (1 image -> 2 images).
                  if index == self.current_paths.len().saturating_sub(1) {
                      self.visible_generation = generation;
                  }
              }
              ImageViewMsg::ViewportResized(w, h) => {
                  self.viewport_size = (w, h);
              }
              ImageViewMsg::ZoomIn => {
                  if self.is_fit_to_window {
                      let new_zoom = self.calculate_current_fit_zoom();
                      if new_zoom > 0.0 {
                          self.zoom = new_zoom;
                      }
                      self.is_fit_to_window = false;
                  }
                  self.zoom *= 1.05;
              }
              ImageViewMsg::ZoomOut => {
                  if self.is_fit_to_window {
                      // Already at fit-to-window (minimum zoom), so ignore zoom out.
                      return;
                  }

                  // Calculate the limit (fit zoom) based on current viewport
                  let fit_zoom = self.calculate_current_fit_zoom();
                  let mut new_zoom = self.zoom / 1.05;

                  // Enforce lower bound (cannot zoom out smaller than fit-to-window)
                  if new_zoom < fit_zoom {
                      new_zoom = fit_zoom;
                      // Optionally, we could set is_fit_to_window = true here if we snapped exactly,
                      // but keeping it simple as requested (just restricting size).
                  }

                  if new_zoom < 0.01 { new_zoom = 0.01; }
                  self.zoom = new_zoom;
              }
              ImageViewMsg::ResetZoom => {
                  self.is_fit_to_window = false;
                  self.zoom = 1.0;
              }
              ImageViewMsg::UpdateSettings { spread_mode, right_to_left, dir_sort, image_sort, input_map, language } => {
                  self.spread_mode = spread_mode;
                  self.right_to_left = right_to_left;
                  self.dir_sort = dir_sort;
                  self.image_sort = image_sort;
                  self.input_map = input_map;
                  self.language = language;
              }
              ImageViewMsg::ChangeDirSort(sort) => {
                  if self.dir_sort != sort {
                      self.dir_sort = sort;
                      let _ = _sender.output(ImageViewOutput::DirSortChanged(sort));
                  }
              }
              ImageViewMsg::ChangeImageSort(sort) => {
                  if self.image_sort != sort {
                      self.image_sort = sort;
                      let _ = _sender.output(ImageViewOutput::ImageSortChanged(sort));
                  }
              }
              ImageViewMsg::ToggleSpread => {
                   self.spread_mode = !self.spread_mode;
                   let _ = _sender.output(ImageViewOutput::SpreadModeChanged(self.spread_mode));
              }
              ImageViewMsg::ToggleDirection => {
                   self.right_to_left = !self.right_to_left;
                   let _ = _sender.output(ImageViewOutput::RTLChanged(self.right_to_left));
              }
               ImageViewMsg::UpdateFullscreen(val) => {
                   self.is_fullscreen = val;
               }
               ImageViewMsg::LoadFallback(index, path, fallback_path, generation) => {
                   let textures = if generation % 2 == 0 { &mut self.textures_even } else { &mut self.textures_odd };
                   
                   if let Some(texture) = textures.get_mut(index) {
                       let path_to_load = if let Some(p) = fallback_path {
                           if p.exists() { Some(p) } else { None } 
                       } else if path.exists() {
                           Some(path.clone())
                       } else {
                           None
                       };

                       if let Some(p) = path_to_load {
                            if let Ok(new_texture) = gtk4::gdk::Texture::from_file(&gtk4::gio::File::for_path(&p)) {
                                *texture = new_texture.upcast();
                            }
                       }
                   }
               }
               ImageViewMsg::TriggerAction(action) => {
                   match action {
                       Action::ZoomIn => { 
                           if self.is_fit_to_window {
                               self.is_fit_to_window = false;
                               self.zoom = self.calculate_current_fit_zoom();
                           }
                           self.zoom *= 1.05;
                       },
                       Action::ZoomOut => { 
                           if self.is_fit_to_window {
                               self.is_fit_to_window = false;
                               self.zoom = self.calculate_current_fit_zoom();
                           }
                           self.zoom /= 1.05;
                           if self.zoom < 0.01 { self.zoom = 0.01; }
                       },
                       Action::ResetZoom => { 
                           self.is_fit_to_window = false;
                           self.zoom = 1.0;
                       },
                        _ => {
                            let _ = _sender.output(ImageViewOutput::TriggerAction(action));
                        }
                   }
               }
               ImageViewMsg::MouseInput { button, modifiers, n_press } => {
                    if let Some(action) = self.input_map.get_action_for_mouse(button, gtk4::gdk::ModifierType::from_bits_truncate(modifiers), n_press == 2) {
                        _sender.input(ImageViewMsg::TriggerAction(action));
                    }
               }
               ImageViewMsg::ScrollInput { dy, modifiers } => {
                    let direction = if dy < 0.0 { ScrollDirection::Up } else { ScrollDirection::Down };
                    if let Some(action) = self.input_map.get_action_for_scroll(direction, gtk4::gdk::ModifierType::from_bits_truncate(modifiers)) {
                         _sender.input(ImageViewMsg::TriggerAction(action));
                    }
               }
         }
    }


    }

impl ImageViewModel {
    fn calculate_current_fit_zoom(&self) -> f64 {
        let (view_w, view_h) = self.viewport_size;
        if view_w <= 0.0 || view_h <= 0.0 { return 1.0; }

        let textures = if self.visible_generation % 2 == 0 { &self.textures_even } else { &self.textures_odd };
        if textures.is_empty() { return 1.0; }

        let mut total_w: f64 = 0.0;
        let mut max_h: f64 = 0.0;
        
        // Simulating the layout logic for spread/single to determine total content size
        if self.spread_mode && textures.len() > 1 {
             // For spread mode with >1 image, we display 2 images side-by-side
             // Width is sum of 2, Height is max of 2
             if let Some(t1) = textures.get(0) {
                 total_w += t1.intrinsic_width() as f64;
                 max_h = max_h.max(t1.intrinsic_height() as f64);
             }
             if let Some(t2) = textures.get(1) {
                 total_w += t2.intrinsic_width() as f64;
                 max_h = max_h.max(t2.intrinsic_height() as f64);
             }
        } else {
             // Single image
             if let Some(t) = textures.get(0) {
                 total_w = t.intrinsic_width() as f64;
                 max_h = t.intrinsic_height() as f64;
             }
        }
        
        if total_w <= 0.0 || max_h <= 0.0 { return 1.0; }

        let scale_w = view_w / total_w;
        let scale_h = view_h / max_h;
        
        scale_w.min(scale_h)
    }
}
