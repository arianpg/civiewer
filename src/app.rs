#![allow(unused_assignments)]

use relm4::prelude::*;
use crate::utils::is_supported_image;
use gtk4::prelude::*;
use std::path::PathBuf;
use dirs;

use crate::components::sidebar::{SidebarModel, SidebarMsg, SidebarOutput};
use crate::components::image_view::{ImageViewModel, ImageViewMsg, ImageViewOutput};
use crate::components::settings_dialog::{SettingsDialogModel, SettingsDialogMsg, SettingsDialogOutput};

use crate::database::{AppSettings, AppState, DbHelper, SortType, DirectorySettings};
use crate::input_settings::{InputMap, Action};
use crate::i18n::{localize, Language};

pub struct AppModel {
    sidebar: Controller<SidebarModel>,
    image_view: Controller<ImageViewModel>,
    settings_dialog: Controller<SettingsDialogModel>,
    settings: AppSettings,
    db_helper: Option<DbHelper>,
    current_image: Option<PathBuf>,
    pending_open_image: Option<PathBuf>,
    
    // View State
    current_dir_sort: SortType,
    current_image_sort: SortType,
    spread_view: bool,
    right_to_left: bool,
    last_path: Option<String>,
    
    is_fullscreen: bool,
    cursor_timeout: Option<gtk4::glib::SourceId>,
    last_cursor_motion: std::time::Instant,
    shared_input_map: std::rc::Rc<std::cell::RefCell<InputMap>>,
    menu_model: gtk4::gio::Menu,
}


#[derive(Debug)]
pub enum AppMsg {
    Quit,
    OpenImage(PathBuf),
    OpenFile,
    OpenDir,
    OpenPath(PathBuf),
    SpreadPages(Vec<PathBuf>),
    NextPage,
    PrevPage,
    ToggleSpread,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    OpenSettings,
    SaveSettings(AppSettings),
    ToggleFullscreen,
    ToggleDirection,
    NextPageSingle,
    PrevPageSingle,
    PathChanged(String),
    DirSortChanged(SortType),
    ImageSortChanged(SortType),
    SpreadModeChanged(bool),
    RTLChanged(bool),
    NextDir,
    PrevDir,
    ClearImage,
    CursorMotion,
    CheckCursorHide,
    TriggerAction(Action),
    KeyInput(gtk4::gdk::Key, gtk4::gdk::ModifierType),
    NoOp,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type Init = ();

    view! {
        main_window = gtk4::Window {
            set_title: Some("CIViewer"),
            set_default_size: (1024, 768),
            
            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                
                // Menubar
                gtk4::PopoverMenuBar::from_model(None::<&gtk4::gio::MenuModel>) {
                    #[watch]
                    set_menu_model: Some(&model.menu_model),
                    #[watch]
                    set_visible: !model.is_fullscreen,
                },

                gtk4::Paned {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_position: 250,
                    set_hexpand: true,
                    set_vexpand: true,
                    
                    #[wrap(Some)]
                    set_start_child = &gtk4::Box {
                        #[watch]
                        set_visible: !model.is_fullscreen,
                        append = model.sidebar.widget(),
                    },
                    
                    #[wrap(Some)]
                    set_end_child = model.image_view.widget(),
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize CSS
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(
            ".image-view-background {
                background-color: black;
            }
            .title-4 {
                font-size: 14px;
                font-weight: bold;
            }
            .selected-image {
                background-color: alpha(@theme_selected_bg_color, 0.5);
                color: @theme_selected_fg_color;
                font-weight: bold;
            }
            scrollbar slider {
                min-width: 10px;
                min-height: 10px;
            }"
        );
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let sidebar = SidebarModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                SidebarOutput::OpenImage(path) => AppMsg::OpenImage(path),
                // SidebarOutput::OpenDirectory(path) => AppMsg::DirSelected(path),
                SidebarOutput::SpreadPages(paths) => AppMsg::SpreadPages(paths),
                SidebarOutput::PathChanged(p) => AppMsg::PathChanged(p),
                SidebarOutput::DirSortChanged(s) => AppMsg::DirSortChanged(s),
                SidebarOutput::ImageSortChanged(s) => AppMsg::ImageSortChanged(s),
                SidebarOutput::ClearImage => AppMsg::ClearImage,
            });
        let image_view = ImageViewModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                ImageViewOutput::DirSortChanged(s) => AppMsg::DirSortChanged(s),
                ImageViewOutput::ImageSortChanged(s) => AppMsg::ImageSortChanged(s),
                ImageViewOutput::SpreadModeChanged(v) => AppMsg::SpreadModeChanged(v),
                ImageViewOutput::RTLChanged(v) => AppMsg::RTLChanged(v),
                ImageViewOutput::TriggerAction(a) => AppMsg::TriggerAction(a),
            });

        let settings_dialog = SettingsDialogModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                SettingsDialogOutput::SaveSettings(settings) => AppMsg::SaveSettings(settings),
                SettingsDialogOutput::Close => { AppMsg::NoOp },
            });

        // Menu Model
        // Removed static menu_model init, using get_menu_model instead
        
        // Actions
        let action_group = gtk4::gio::SimpleActionGroup::new();
        
        let sender_clone = sender.clone();
        let open_file_action = gtk4::gio::SimpleAction::new("open-file", None);
        open_file_action.connect_activate(move |_, _| { sender_clone.input(AppMsg::OpenFile); });
        action_group.add_action(&open_file_action);

        let sender_clone = sender.clone();
        let open_dir_action = gtk4::gio::SimpleAction::new("open-dir", None);
        open_dir_action.connect_activate(move |_, _| { sender_clone.input(AppMsg::OpenDir); });
        action_group.add_action(&open_dir_action);

        let sender_clone = sender.clone();
        let settings_action = gtk4::gio::SimpleAction::new("settings", None);
        settings_action.connect_activate(move |_, _| { sender_clone.input(AppMsg::OpenSettings); });
        action_group.add_action(&settings_action);

        let sender_clone = sender.clone();
        let quit_action = gtk4::gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(move |_, _| { sender_clone.input(AppMsg::Quit); });
        action_group.add_action(&quit_action);
        
        let sender_clone = sender.clone();
        let toggle_spread = gtk4::gio::SimpleAction::new("toggle-spread", None);
        toggle_spread.connect_activate(move |_, _| { sender_clone.input(AppMsg::ToggleSpread); });
        action_group.add_action(&toggle_spread);
        
        let sender_clone = sender.clone();
        let toggle_rtl = gtk4::gio::SimpleAction::new("toggle-rtl", None);
        toggle_rtl.connect_activate(move |_, _| { sender_clone.input(AppMsg::ToggleDirection); });
        action_group.add_action(&toggle_rtl);
        
        root.insert_action_group("win", Some(&action_group));

        let mut settings = AppSettings::default();
        let mut app_state = AppState::default();
        let mut db_helper = None;

        // Initialize DB
        if let Some(config_dir) = dirs::config_dir() {
            let db_path = config_dir.join("civiewer").join("settings.polodb");
            match DbHelper::new(db_path) {
                Ok(helper) => {
                    if let Ok(s) = helper.get_settings() {
                        settings = s;
                    }
                    if let Ok(s) = helper.get_app_state() {
                        app_state = s;
                    }
                    db_helper = Some(helper);
                }
                Err(e) => eprintln!("Failed to init DB: {}", e),
            }
        }
        
        // Parse startup args
        let args: Vec<String> = std::env::args().collect();
        // Determine initial view state based on defaults
        let spread_view = settings.default_spread_view;
        let right_to_left = settings.default_right_to_left;
        let current_dir_sort = settings.default_dir_sort;
        let current_image_sort = settings.default_image_sort;

        // Shared InputMap for controller
        let shared_input_map = std::rc::Rc::new(std::cell::RefCell::new(settings.input_map.clone()));
        
        // Initial Menu Model
        let menu_model = create_menu_model(settings.language);

        let model = AppModel {
            sidebar,
            image_view,
            settings_dialog,
            settings: settings.clone(),
            db_helper,
            current_image: None, 
            pending_open_image: None,
            current_dir_sort,
            current_image_sort,
            spread_view,
            right_to_left,
            last_path: app_state.last_path.clone(),
            is_fullscreen: false,
            cursor_timeout: None,
            last_cursor_motion: std::time::Instant::now(),
            shared_input_map: shared_input_map.clone(),
            menu_model,
        };
        
        // Handle startup target
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            sender.input(AppMsg::OpenPath(path));
        } else if let Some(last_path_str) = &model.last_path {
            let path = PathBuf::from(last_path_str);
             // Verify it exists, else do nothing
             if path.exists() {
                 sender.input(AppMsg::OpenPath(path));
             }
        }
        
        if let Some(gtk_settings) = gtk4::Settings::default() {
            gtk_settings.set_gtk_application_prefer_dark_theme(model.settings.dark_mode);
        }

        // Initialize ImageView with current state
        model.image_view.emit(ImageViewMsg::UpdateSettings {
            spread_mode: model.spread_view,
            right_to_left: model.right_to_left,
            dir_sort: model.current_dir_sort,
            image_sort: model.current_image_sort,
            input_map: model.settings.input_map.clone(),
        });
        model.sidebar.emit(SidebarMsg::UpdateSpreadMode(model.spread_view));
        model.sidebar.emit(SidebarMsg::ChangeDirSort(model.current_dir_sort));
        model.sidebar.emit(SidebarMsg::ChangeImageSort(model.current_image_sort));
        model.sidebar.emit(SidebarMsg::UpdateLoopImages(model.settings.loop_images));
        model.sidebar.emit(SidebarMsg::UpdateSingleFirstPage(model.settings.single_first_page));

        let widgets = view_output!();
        
        // Key Controller
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let sender_key = sender.clone();
        let map_clone = shared_input_map.clone();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            // Check InputMap first
            if let Some(action) = map_clone.borrow().get_action_for_key(key, modifiers) {
                 sender_key.input(AppMsg::TriggerAction(action));
                 return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        widgets.main_window.add_controller(key_controller);

        // Motion Controller
        let motion_controller = gtk4::EventControllerMotion::new();
        let sender_motion = sender.clone();
        motion_controller.connect_motion(move |_, _, _| {
            sender_motion.input(AppMsg::CursorMotion);
        });
        widgets.main_window.add_controller(motion_controller);

        // Drop Target
        let drop_target = gtk4::DropTarget::new(gtk4::gio::File::static_type(), gtk4::gdk::DragAction::COPY);
        let sender_drop = sender.clone();
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(file) = value.get::<gtk4::gio::File>() {
                if let Some(path) = file.path() {
                    sender_drop.input(AppMsg::OpenPath(path));
                    return true;
                }
            }
            false
        });
        widgets.main_window.add_controller(drop_target);
        
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Quit => {
                relm4::main_application().quit();
            }
             AppMsg::OpenImage(path) => {
                self.current_image = Some(path.clone());
                self.sidebar.emit(SidebarMsg::SelectImage(path.clone()));
                if self.spread_view {
                    self.sidebar.emit(SidebarMsg::GetSpreadPages(path));
                } else {
                    self.image_view.emit(ImageViewMsg::ShowPages(vec![path]));
                }
            }
            AppMsg::SpreadPages(paths) => {
                self.image_view.emit(ImageViewMsg::ShowPages(paths));
            }
            AppMsg::NextPage => {
                if let Some(path) = &self.current_image {
                    self.sidebar.emit(SidebarMsg::OpenNextImage(path.clone()));
                }
            }
            AppMsg::PrevPage => {
                if let Some(path) = &self.current_image {
                    self.sidebar.emit(SidebarMsg::OpenPrevImage(path.clone()));
                }
            }
            AppMsg::ToggleSpread => {
                _sender.input(AppMsg::SpreadModeChanged(!self.spread_view));
            }
            AppMsg::ToggleDirection => {
                 _sender.input(AppMsg::RTLChanged(!self.right_to_left));
            }
            AppMsg::SpreadModeChanged(val) => {
                self.handle_spread_mode_changed(val);
            }
            AppMsg::RTLChanged(val) => {
                self.handle_rtl_changed(val);
            }
            AppMsg::ZoomIn => self.image_view.emit(ImageViewMsg::ZoomIn),
            AppMsg::ZoomOut => self.image_view.emit(ImageViewMsg::ZoomOut),
            AppMsg::ResetZoom => self.image_view.emit(ImageViewMsg::ResetZoom),
            
            AppMsg::OpenFile => {
                 let dialog = gtk4::FileChooserNative::new(
                    Some("Open Image"),
                    gtk4::Window::NONE,
                    gtk4::FileChooserAction::Open,
                    Some("Open"),
                    Some("Cancel"),
                );
                let sender = _sender.clone();
                dialog.connect_response(move |d, response| {
                    if response == gtk4::ResponseType::Accept {
                        if let Some(file) = d.file() {
                            if let Some(path) = file.path() {
                                sender.input(AppMsg::OpenPath(path));
                            }
                        }
                    }
                    d.destroy();
                });
                dialog.show();
            }
            AppMsg::OpenDir => {
                 let dialog = gtk4::FileChooserNative::new(
                    Some("Open Directory"),
                    gtk4::Window::NONE,
                    gtk4::FileChooserAction::SelectFolder,
                    Some("Open"),
                    Some("Cancel"),
                );
                let sender = _sender.clone();
                dialog.connect_response(move |d, response| {
                    if response == gtk4::ResponseType::Accept {
                        if let Some(file) = d.file() {
                            if let Some(path) = file.path() {
                                sender.input(AppMsg::OpenPath(path));
                            }
                        }
                    }
                    d.destroy();
                });
                dialog.show();
            }
            AppMsg::OpenPath(path) => {
                if path.is_dir() {
                     self.sidebar.emit(SidebarMsg::OpenDirectory(path));
                } else if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("zip")) {
                     self.sidebar.emit(SidebarMsg::OpenDirectory(path));
                } else if is_supported_image(&path) {
                     if let Some(parent) = path.parent() {
                          self.pending_open_image = Some(path.clone());
                          self.sidebar.emit(SidebarMsg::UpdatePath(parent.to_path_buf()));
                     }
                }
            }
            AppMsg::OpenSettings => {
                self.settings_dialog.emit(SettingsDialogMsg::Open(self.settings.clone()));
            }
            AppMsg::SaveSettings(new_settings) => {
                self.settings = new_settings;
                 if let Some(helper) = &self.db_helper {
                    if let Err(e) = helper.save_settings(&self.settings) {
                         eprintln!("Failed to save settings: {}", e);
                    }
                }
                 
                 // Update shared map
                 self.shared_input_map.replace(self.settings.input_map.clone());

                 // Apply update loops, dark mode, single first page
                self.sidebar.emit(SidebarMsg::UpdateLoopImages(self.settings.loop_images));
                self.sidebar.emit(SidebarMsg::UpdateSingleFirstPage(self.settings.single_first_page));
                if let Some(gtk_settings) = gtk4::Settings::default() {
                    gtk_settings.set_gtk_application_prefer_dark_theme(self.settings.dark_mode);
                }
                
                self.image_view.emit(ImageViewMsg::UpdateSettings {
                    spread_mode: self.settings.default_spread_view,
                    right_to_left: self.settings.default_right_to_left,
                    dir_sort: self.settings.default_dir_sort,
                    image_sort: self.settings.default_image_sort,
                    input_map: self.settings.input_map.clone(),
                });
                
                // Update Menu
                self.menu_model = create_menu_model(self.settings.language);
            }
            AppMsg::ToggleFullscreen => {
                let window = &self.sidebar.widget().root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                if let Some(win) = window {
                     // Cancel any pending timer
                     if let Some(source_id) = self.cursor_timeout.take() {
                         source_id.remove();
                     }

                     if win.is_fullscreen() {
                         win.unfullscreen();
                         self.is_fullscreen = false;
                         
                         // Show cursor
                         win.set_cursor(None::<&gtk4::gdk::Cursor>);
                     } else {
                         win.fullscreen();
                         self.is_fullscreen = true;
                         self.last_cursor_motion = std::time::Instant::now();
                         
                         // Start polling timer (repeating)
                         let sender = _sender.clone();
                         self.cursor_timeout = Some(gtk4::glib::timeout_add_local(
                             std::time::Duration::from_millis(500),
                             move || {
                                 sender.input(AppMsg::CheckCursorHide);
                                 gtk4::glib::ControlFlow::Continue
                             }
                         ));
                     }
                     self.image_view.emit(ImageViewMsg::UpdateFullscreen(self.is_fullscreen));
                }
            }
            AppMsg::NextPageSingle => {
                if let Some(path) = &self.current_image {
                    self.sidebar.emit(SidebarMsg::OpenNextImageSingle(path.clone())); 
                }
            }
            AppMsg::PrevPageSingle => {
                 if let Some(path) = &self.current_image {
                    self.sidebar.emit(SidebarMsg::OpenPrevImageSingle(path.clone()));
                 }
            }
            //
            // Cursor Logic
            //
            AppMsg::CursorMotion => {
                // Just update timestamp and show cursor
                self.last_cursor_motion = std::time::Instant::now();
                if let Some(window) = &self.sidebar.widget().root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                     window.set_cursor(None::<&gtk4::gdk::Cursor>);
                }
            }
            AppMsg::CheckCursorHide => {
                if self.is_fullscreen {
                    if self.last_cursor_motion.elapsed() > std::time::Duration::from_secs(3) {
                        if let Some(window) = &self.sidebar.widget().root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                            let cursor = gtk4::gdk::Cursor::from_name("none", None);
                            window.set_cursor(cursor.as_ref());
                        }
                    }
                }
            }
            AppMsg::PathChanged(path_str) => {
                self.handle_path_changed(path_str);
            }
            AppMsg::DirSortChanged(sort) => {
                self.current_dir_sort = sort;
                if let Some(path_str) = &self.last_path {
                     if let Some(helper) = &self.db_helper {
                          let is_archive = path_str.to_lowercase().ends_with(".zip");
                          let target_path_str = if is_archive {
                              let p = std::path::Path::new(path_str.as_str());
                              p.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or(path_str.clone())
                          } else {
                             path_str.clone()
                         };
                         
                         let mut ds = if let Ok(Some(existing)) = helper.get_directory_settings(&target_path_str) {
                             existing
                         } else {
                             DirectorySettings {
                                 path: target_path_str.clone(),
                                 spread_view: self.settings.default_spread_view, 
                                 right_to_left: self.settings.default_right_to_left,
                                 dir_sort: self.settings.default_dir_sort,
                                 image_sort: self.settings.default_image_sort,
                             }
                         };
                         
                         ds.dir_sort = self.current_dir_sort;
                         let _ = helper.save_directory_settings(&ds);
                     }
                }
                self.sidebar.emit(SidebarMsg::ChangeDirSort(sort));
            }
             AppMsg::ImageSortChanged(sort) => {
                self.current_image_sort = sort;
                if let Some(path) = &self.last_path {
                     if let Some(helper) = &self.db_helper {
                         let ds = DirectorySettings {
                             path: path.clone(),
                             spread_view: self.spread_view,
                             right_to_left: self.right_to_left,
                             dir_sort: self.current_dir_sort,
                             image_sort: self.current_image_sort,
                         };
                         let _ = helper.save_directory_settings(&ds);
                     }
                }
                self.sidebar.emit(SidebarMsg::ChangeImageSort(sort));
            }
            AppMsg::NextDir => {
                self.sidebar.emit(SidebarMsg::OpenNextDir);
            }
            AppMsg::PrevDir => {
                self.sidebar.emit(SidebarMsg::OpenPrevDir);
            }
            AppMsg::ClearImage => {
                self.image_view.emit(ImageViewMsg::ShowPages(vec![]));
            }
            AppMsg::TriggerAction(action) => {
                 match action {
                    Action::PrevDir => _sender.input(AppMsg::PrevDir),
                    Action::NextDir => _sender.input(AppMsg::NextDir),
                    Action::PrevPage => _sender.input(AppMsg::PrevPage),
                    Action::NextPage => _sender.input(AppMsg::NextPage),
                    Action::ToggleFullscreen => _sender.input(AppMsg::ToggleFullscreen),
                    Action::ZoomIn => _sender.input(AppMsg::ZoomIn),
                    Action::ZoomOut => _sender.input(AppMsg::ZoomOut),
                    Action::ResetZoom => _sender.input(AppMsg::ResetZoom),
                    Action::ToggleSpread => _sender.input(AppMsg::ToggleSpread),
                    Action::ToggleRTL => _sender.input(AppMsg::ToggleDirection),
                    Action::PrevPageSingle => _sender.input(AppMsg::PrevPageSingle),
                    Action::NextPageSingle => _sender.input(AppMsg::NextPageSingle),
                }
            }
            AppMsg::KeyInput(_, _) => {
                 // Deprecated / Handled by controller directly now
            }
            AppMsg::NoOp => {}
        }
    }
}

impl AppModel {
    fn handle_path_changed(&mut self, path_str: String) {
        self.last_path = Some(path_str.clone());
        
        if let Some(helper) = &self.db_helper {
            // Save AppState
            let state = crate::database::AppState { key: "global".to_string(), last_path: Some(path_str.clone()) };
            let _ = helper.save_app_state(&state);
            
            // Defaults
            self.spread_view = self.settings.default_spread_view;
            self.right_to_left = self.settings.default_right_to_left;
            self.current_dir_sort = self.settings.default_dir_sort;
            self.current_image_sort = self.settings.default_image_sort;

            let is_archive = path_str.to_lowercase().ends_with(".zip");

            // Load Image Settings (from Archive or Dir)
            if let Ok(Some(dir_settings)) = helper.get_directory_settings(&path_str) {
                 self.spread_view = dir_settings.spread_view;
                 self.right_to_left = dir_settings.right_to_left;
                 self.current_image_sort = dir_settings.image_sort;
                 if !is_archive {
                     self.current_dir_sort = dir_settings.dir_sort;
                 }
            }

            // Load Directory Sort (from Parent if Archive)
            if is_archive {
                let path = std::path::Path::new(&path_str);
                if let Some(parent) = path.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    if let Ok(Some(parent_settings)) = helper.get_directory_settings(&parent_str) {
                        self.current_dir_sort = parent_settings.dir_sort;
                    }
                }
            }
             
             // Update UI
             self.image_view.emit(ImageViewMsg::UpdateSettings { 
                spread_mode: self.spread_view, 
                right_to_left: self.right_to_left,
                dir_sort: self.current_dir_sort,
                image_sort: self.current_image_sort,
                input_map: self.settings.input_map.clone(),
             });
             self.sidebar.emit(SidebarMsg::UpdateSpreadMode(self.spread_view));
             self.sidebar.emit(SidebarMsg::ChangeDirSort(self.current_dir_sort));
             self.sidebar.emit(SidebarMsg::ChangeImageSort(self.current_image_sort));
             
             // Check pending image open
             if let Some(pending) = &self.pending_open_image {
                 self.image_view.emit(ImageViewMsg::ShowPages(vec![pending.clone()]));
                 self.sidebar.emit(SidebarMsg::OpenImage(pending.clone()));
                 self.current_image = Some(pending.clone());
                 self.pending_open_image = None;
             } else {
                 self.sidebar.emit(SidebarMsg::OpenFirstImage);
             }
        }
    }

    fn handle_spread_mode_changed(&mut self, val: bool) {
        self.spread_view = val;
        
        // Save directory settings
        if let Some(helper) = &self.db_helper {
             if let Some(path) = &self.last_path {
                 let ds = crate::database::DirectorySettings {
                     path: path.clone(),
                     spread_view: self.spread_view,
                     right_to_left: self.right_to_left,
                     dir_sort: self.current_dir_sort,
                     image_sort: self.current_image_sort,
                  };
                   let _ = helper.save_directory_settings(&ds);
            }
        }
        
        self.image_view.emit(ImageViewMsg::UpdateSettings { 
            spread_mode: self.spread_view, 
            right_to_left: self.right_to_left,
            dir_sort: self.current_dir_sort,
            image_sort: self.current_image_sort,
            input_map: self.settings.input_map.clone(),
        });
        self.sidebar.emit(SidebarMsg::UpdateSpreadMode(self.spread_view));
        
         if let Some(path) = &self.current_image {
            if self.spread_view {
                self.sidebar.emit(SidebarMsg::GetSpreadPages(path.clone()));
            } else {
                self.image_view.emit(ImageViewMsg::ShowPages(vec![path.clone()]));
            }
        }
    }

    fn handle_rtl_changed(&mut self, val: bool) {
        self.right_to_left = val;
        if let Some(helper) = &self.db_helper {
             if let Some(path_str) = &self.last_path {
                  let ds = crate::database::DirectorySettings {
                     path: path_str.clone(),
                     spread_view: self.spread_view,
                     right_to_left: self.right_to_left,
                     dir_sort: self.current_dir_sort,
                     image_sort: self.current_image_sort,
                  };
                   let _ = helper.save_directory_settings(&ds);
             }
        }
        self.image_view.emit(ImageViewMsg::UpdateSettings { 
            spread_mode: self.spread_view, 
            right_to_left: self.right_to_left,
             dir_sort: self.current_dir_sort,
             image_sort: self.current_image_sort,
             input_map: self.settings.input_map.clone(),
        });
        
        if let Some(path) = &self.current_image {
            if self.spread_view {
                self.sidebar.emit(SidebarMsg::GetSpreadPages(path.clone()));
            } else {
                self.image_view.emit(ImageViewMsg::ShowPages(vec![path.clone()]));
            }
        }
    }
}

fn create_menu_model(lang: Language) -> gtk4::gio::Menu {
    let menu_model = gtk4::gio::Menu::new();
    
    let file_menu = gtk4::gio::Menu::new();
    file_menu.append(Some(&localize("Open File", lang)), Some("win.open-file"));
    file_menu.append(Some(&localize("Open Directory", lang)), Some("win.open-dir"));
    file_menu.append(Some(&localize("Quit", lang)), Some("win.quit"));
    menu_model.append_submenu(Some(&localize("File", lang)), &file_menu);
    
    let settings_menu = gtk4::gio::Menu::new();
    settings_menu.append(Some(&localize("Preferences", lang)), Some("win.settings"));
    menu_model.append_submenu(Some(&localize("Settings", lang)), &settings_menu);

    menu_model
}
