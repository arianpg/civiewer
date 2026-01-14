use relm4::prelude::*;
use gtk4::prelude::*;
use std::path::PathBuf;
use crate::database::SortType;
use crate::input_settings::{InputMap, Action, ScrollDirection};

#[derive(Debug)]
pub struct ImageViewModel {
    current_paths: Vec<PathBuf>,
    textures: Vec<gtk4::gdk::Paintable>,
    zoom: f64,
    is_fit_to_window: bool,
    pub spread_mode: bool,
    pub right_to_left: bool,
    pub dir_sort: SortType,
    pub image_sort: SortType,
    pub is_fullscreen: bool,
    temp_files: Vec<tempfile::NamedTempFile>,
    pub input_map: InputMap,
}

#[derive(Debug)]
pub enum ImageViewMsg {
    ShowPages(Vec<PathBuf>),
    ZoomIn,
    ZoomOut,
    ResetZoom,
    UpdateSettings { spread_mode: bool, right_to_left: bool, dir_sort: SortType, image_sort: SortType, input_map: InputMap },
    ChangeDirSort(SortType),
    ChangeImageSort(SortType),
    ToggleSpreadMode(bool),
    ToggleRTL(bool),
    UpdateFullscreen(bool),
    LoadFallback(usize, PathBuf, Option<PathBuf>),
    TriggerAction(Action),
    MouseInput { button: u32, modifiers: u32, n_press: i32 },
    ScrollInput { dy: f64, modifiers: u32 },
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
                    
                    gtk4::CheckButton {
                        set_label: Some("Spread View"),
                        set_focusable: false,
                        #[watch]
                        set_active: model.spread_mode,
                        connect_toggled[sender] => move |btn| {
                            sender.input(ImageViewMsg::ToggleSpreadMode(btn.is_active()));
                        }
                    },
                    
                    gtk4::CheckButton {
                        set_label: Some("Right to Left"),
                        set_focusable: false,
                        #[watch]
                        set_active: model.right_to_left,
                        connect_toggled[sender] => move |btn| {
                            sender.input(ImageViewMsg::ToggleRTL(btn.is_active()));
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
            
                gtk4::ScrolledWindow {
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
                        connect_scroll[sender] => move |_, _dx, dy| {
                             // Modifiers are hard to retrieve in connect_scroll without direct event access
                             // Default to 0 (no modifiers) for scroll actions for now
                             let modifiers = 0;
                             
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
                            
                            // Left Page (or single)
                            append = &gtk4::Picture {
                                #[watch]
                                set_halign: if model.is_fit_to_window { 
                                    if model.spread_mode && model.textures.len() > 1 { gtk4::Align::End } else { gtk4::Align::Fill }
                                } else { gtk4::Align::Center },
                                #[watch]
                                set_valign: if model.is_fit_to_window { gtk4::Align::Fill } else { gtk4::Align::Center },
                                
                                #[watch]
                                set_hexpand: model.is_fit_to_window,
                                #[watch]
                                set_vexpand: model.is_fit_to_window,

                                #[watch]
                                set_paintable: if model.spread_mode && model.textures.len() > 1 {
                                     if model.right_to_left { model.textures.get(1) } else { model.textures.get(0) }
                                } else {
                                     model.textures.get(0)
                                },
                                #[watch]
                                set_can_shrink: model.is_fit_to_window,
                                
                                #[watch]
                                set_width_request: if model.is_fit_to_window { -1 } else { 
                                    let idx = if model.spread_mode && model.textures.len() > 1 && model.right_to_left { 1 } else { 0 };
                                    model.textures.get(idx).map_or(-1, |t| (t.intrinsic_width() as f64 * model.zoom) as i32) 
                                },
                                #[watch]
                                set_height_request: if model.is_fit_to_window { -1 } else { 
                                     let idx = if model.spread_mode && model.textures.len() > 1 && model.right_to_left { 1 } else { 0 };
                                     model.textures.get(idx).map_or(-1, |t| (t.intrinsic_height() as f64 * model.zoom) as i32)
                                },
                                #[watch]
                                set_visible: if model.spread_mode && model.textures.len() > 1 {
                                     true 
                                } else {
                                     !model.textures.is_empty()
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
                                set_paintable: if model.spread_mode && model.textures.len() > 1 {
                                     if model.right_to_left { model.textures.get(0) } else { model.textures.get(1) }
                                } else {
                                     None
                                },
                                #[watch]
                                set_can_shrink: model.is_fit_to_window,
                                #[watch]
                                set_width_request: if model.is_fit_to_window { -1 } else { 
                                    let idx = if model.spread_mode && model.textures.len() > 1 && model.right_to_left { 0 } else { 1 };
                                    model.textures.get(idx).map_or(-1, |t| (t.intrinsic_width() as f64 * model.zoom) as i32) 
                                },
                                #[watch]
                                set_height_request: if model.is_fit_to_window { -1 } else { 
                                     let idx = if model.spread_mode && model.textures.len() > 1 && model.right_to_left { 0 } else { 1 };
                                     model.textures.get(idx).map_or(-1, |t| (t.intrinsic_height() as f64 * model.zoom) as i32)
                                },
                                #[watch]
                                set_visible: model.spread_mode && model.textures.len() > 1,
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
            textures: Vec::new(),
            zoom: 1.0,
            is_fit_to_window: true,
            spread_mode: false,
            right_to_left: true,
            dir_sort: SortType::NameAsc,
            image_sort: SortType::NameAsc,
            is_fullscreen: false,
            temp_files: Vec::new(),
            input_map: InputMap::default(),
        };
        
        let drag_state = std::rc::Rc::new(std::cell::RefCell::new((0.0, 0.0)));
        let drag_state_2 = drag_state.clone();
        
        let widgets = view_output!();
        
        // Manual fix to properly get scroll modifiers involves getting the controller from widgets
        // For now, simpler implementation is accepted.
        
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
             ImageViewMsg::ShowPages(paths) => {
                  self.current_paths = paths.clone();
                  self.is_fit_to_window = true;
                  self.textures.clear();
                  let old_files = std::mem::take(&mut self.temp_files);
                  std::thread::spawn(move || { drop(old_files); });
                  
                  for path in &paths {
                      let mut found = false;
                      if path.exists() {
                          let is_anim = path.extension().and_then(|s| s.to_str()).map_or(false, |ext| {
                              let ext = ext.to_lowercase();
                              if ext == "gif" || ext == "webp" || ext == "apng" { return true; }
                              if ext == "png" { return crate::utils::is_apng(path); }
                              false
                          });

                          if is_anim {
                              let file = gtk4::gio::File::for_path(path);
                              let media = gtk4::MediaFile::for_file(&file);
                               media.set_loop(true);
                               media.play();
                               
                               let sender_clone = _sender.clone();
                               let index = self.textures.len();
                               let path_clone = path.clone();
                               media.connect_notify(Some("error"), move |_, _| {
                                   sender_clone.input(ImageViewMsg::LoadFallback(index, path_clone.clone(), None));
                               });

                              self.textures.push(media.upcast());
                              found = true;
                          } else if let Ok(texture) = gtk4::gdk::Texture::from_file(&gtk4::gio::File::for_path(path)) {
                              self.textures.push(texture.upcast());
                              found = true;
                          }
                      }
                      
                      if !found {
                           // Zip logic
                          let mut current = path.clone();
                          while let Some(parent) = current.parent() {
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
                                                              let bytes = gtk4::glib::Bytes::from(&buffer);
                                                              let is_anim = path.extension().and_then(|s| s.to_str()).map_or(false, |ext| {
                                                                  let ext = ext.to_lowercase();
                                                                  if ext == "gif" || ext == "webp" || ext == "apng" { return true; }
                                                                  if ext == "png" { return crate::utils::is_apng_bytes(&bytes); }
                                                                  false
                                                              });

                                                              if is_anim {
                                                                  let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("tmp");
                                                                  if let Ok(mut temp_file) = tempfile::Builder::new().suffix(&format!(".{}", ext)).tempfile() {
                                                                      use std::io::Write;
                                                                      if temp_file.write_all(&bytes).is_ok() {
                                                                          let temp_path = temp_file.path().to_path_buf();
                                                                          let media = gtk4::MediaFile::for_filename(temp_path.to_str().unwrap_or(""));
                                                                          media.set_loop(true);
                                                                          media.play();
                                                                          
                                                                          let sender_clone = _sender.clone();
                                                                          let index = self.textures.len();
                                                                          let path_clone = path.clone();
                                                                          let temp_path_clone = temp_path.clone(); 
                                                                          media.connect_notify(Some("error"), move |_, _| {
                                                                              sender_clone.input(ImageViewMsg::LoadFallback(index, path_clone.clone(), Some(temp_path_clone.clone())));
                                                                          });

                                                                          self.textures.push(media.upcast());
                                                                          self.temp_files.push(temp_file);
                                                                          found = true;
                                                                      }
                                                                  }
                                                              } else if let Ok(texture) = gtk4::gdk::Texture::from_bytes(&bytes) {
                                                                 self.textures.push(texture.upcast());
                                                                 found = true;
                                                              }
                                                          }
                                                      }
                                                  }
                                              }
                                          }
                                      }
                                  }
                                  if found { break; }
                              }
                              current = parent.to_path_buf();
                          }
                      }
                      
                      if !found && path.exists() {
                          // Try loading as simple texture again if no other special handling
                          // (This is a fallback if is_anim check failed but it is an image)
                           if let Ok(texture) = gtk4::gdk::Texture::from_file(&gtk4::gio::File::for_path(path)) {
                              self.textures.push(texture.upcast());
                          }
                      }
                  }
              }
              ImageViewMsg::ZoomIn => {
                  self.is_fit_to_window = false;
                  self.zoom *= 1.2;
              }
              ImageViewMsg::ZoomOut => {
                  self.is_fit_to_window = false;
                  self.zoom /= 1.2;
                  if self.zoom < 0.1 { self.zoom = 0.1; }
              }
              ImageViewMsg::ResetZoom => {
                  self.is_fit_to_window = false;
                  self.zoom = 1.0;
              }
              ImageViewMsg::UpdateSettings { spread_mode, right_to_left, dir_sort, image_sort, input_map } => {
                  self.spread_mode = spread_mode;
                  self.right_to_left = right_to_left;
                  self.dir_sort = dir_sort;
                  self.image_sort = image_sort;
                  self.input_map = input_map;
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
              ImageViewMsg::ToggleSpreadMode(val) => {
                  if self.spread_mode != val {
                      self.spread_mode = val;
                      let _ = _sender.output(ImageViewOutput::SpreadModeChanged(val));
                  }
              }
              ImageViewMsg::ToggleRTL(val) => {
                  if self.right_to_left != val {
                      self.right_to_left = val;
                      let _ = _sender.output(ImageViewOutput::RTLChanged(val));
                  }
              }
               ImageViewMsg::UpdateFullscreen(val) => {
                   self.is_fullscreen = val;
               }
               ImageViewMsg::LoadFallback(index, path, fallback_path) => {
                   if let Some(texture) = self.textures.get_mut(index) {
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
                           self.is_fit_to_window = false;
                           self.zoom *= 1.2;
                       },
                       Action::ZoomOut => { 
                           self.is_fit_to_window = false;
                           self.zoom /= 1.2;
                           if self.zoom < 0.1 { self.zoom = 0.1; }
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
