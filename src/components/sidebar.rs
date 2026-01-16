#![allow(unused_assignments)]
use crate::database::SortType;
use natord::compare as natural_lexical_cmp;
use crate::utils::is_supported_image;

use relm4::prelude::*;
use relm4::factory::FactoryVecDeque;
use std::path::PathBuf;
use image::ImageReader;
use gtk4::prelude::*;

#[derive(Debug)]
pub struct DirectoryItem {
    pub name: String,
    pub path: PathBuf,
    pub is_archive: bool,
    pub is_selected: bool,
}

#[derive(Debug)]
pub struct ImageItem {
    pub filename: String,
    pub path: PathBuf,
    pub index: usize,
    pub is_selected: bool,
}

#[derive(Debug)]
pub struct SidebarModel {
    current_path: String,
    directories: FactoryVecDeque<DirectoryItem>,
    images: FactoryVecDeque<ImageItem>,
    dir_sort: SortType,
    image_sort: SortType,
    spread_view: bool, 
    selected_path: Option<PathBuf>, // For image selection
    selected_dir_path: Option<PathBuf>, // For directory/archive selection
    preview_archive_path: Option<PathBuf>, // If set, images are loaded from this archive, but dir list matches current_path
    images_scrolled_window: Option<gtk::ScrolledWindow>,
    directories_scrolled_window: Option<gtk::ScrolledWindow>,
    loop_images: bool,
    single_first_page: bool,
    archives_on_top: bool,
}

#[derive(Debug)]
pub enum SidebarMsg {
    GoUp,
    UpdatePath(PathBuf),
    OpenDirectory(PathBuf),
    OpenImage(PathBuf),

    ChangeDirSort(SortType),
    ChangeImageSort(SortType),
    UpdateSpreadMode(bool),
    GetSpreadPages(PathBuf),
    OpenNextImage(PathBuf),
    OpenPrevImage(PathBuf),
    OpenNextImageSingle(PathBuf),
    OpenPrevImageSingle(PathBuf),
    SelectImage(PathBuf),
    UpdateLoopImages(bool),
    UpdateSingleFirstPage(bool),
    UpdateArchivesOnTop(bool),
    OpenFirstImage,
    ScrollToSelection,
}

#[derive(Debug, Clone)]
pub enum DirectoryItemMsg {
    UpdateSelection(Option<PathBuf>),
}

#[derive(Debug, Clone)]
pub enum ImageItemMsg {
    UpdateSelection(Option<PathBuf>),
}

#[derive(Debug)]
pub enum SidebarOutput {
    OpenImage(PathBuf),
    // OpenDirectory(PathBuf),
    SpreadPages(Vec<PathBuf>),
    PathChanged(String),
    DirSortChanged(SortType),
    ImageSortChanged(SortType),
    ClearImage,
    RequestNextDir,
    RequestPrevDir,
}

#[relm4::factory(pub)]
impl FactoryComponent for DirectoryItem {
    type Init = (String, PathBuf, bool, bool);
    type Input = DirectoryItemMsg;
    type Output = SidebarMsg;
    type CommandOutput = ();
    type ParentWidget = gtk4::Box;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 5,
            
            gtk4::Image {
                set_icon_name: Some(if self.is_archive { "package-x-generic" } else { "folder" }),
                set_pixel_size: 16,
            },

            gtk4::Label {
                set_label: &self.name,
                set_hexpand: true,
                set_xalign: 0.0,
                set_ellipsize: gtk4::pango::EllipsizeMode::End,
                #[watch]
                set_css_classes: if self.is_selected { &["selected-image"] } else { &[] },
            },
            
            add_controller = gtk4::GestureClick {
                connect_released[sender, path = self.path.clone()] => move |_, _, _, _| {
                    let _ = sender.output(SidebarMsg::OpenDirectory(path.clone()));
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name: init.0,
            path: init.1,
            is_archive: init.2,
            is_selected: init.3,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
             DirectoryItemMsg::UpdateSelection(selected_path) => {
                 self.is_selected = selected_path.as_ref() == Some(&self.path);
             }
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for ImageItem {
    type Init = (String, PathBuf, usize);
    type Input = ImageItemMsg;
    type Output = SidebarMsg;
    type CommandOutput = ();
    type ParentWidget = gtk4::Box;

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 5,
            #[watch]
            set_css_classes: if self.is_selected { &["selected-image"] } else { &[] },
            
            gtk4::Label {
                set_label: &format!("{}", self.index),
                set_width_chars: 4,
                set_xalign: 1.0,
            },

            gtk4::Label {
                set_label: &self.filename,
                set_hexpand: true,
                set_xalign: 0.0,
                set_ellipsize: gtk4::pango::EllipsizeMode::End,
            },

            add_controller = gtk4::GestureClick {
                connect_released[sender, path = self.path.clone()] => move |_, _, _, _| {
                    let _ = sender.output(SidebarMsg::OpenImage(path.clone()));
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            filename: init.0,
            path: init.1,
            index: init.2,
            is_selected: false,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            ImageItemMsg::UpdateSelection(selected_path) => {
                 let _was_selected = self.is_selected;
                 self.is_selected = selected_path.as_ref() == Some(&self.path);
            }
        }
    }
}



#[relm4::component(pub)]
impl SimpleComponent for SidebarModel {
    type Input = SidebarMsg;
    type Output = SidebarOutput;
    type Init = ();

    view! {
        gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 5,
            set_margin_all: 5,

            // Path Bar
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 5,
                
                gtk4::Box {
                     set_hexpand: true,
                     set_orientation: gtk4::Orientation::Horizontal,
                     set_spacing: 5,
                     
                     gtk4::Label {
                        #[watch]
                        set_text: model.preview_archive_path.as_ref().map(|p| p.to_string_lossy()).as_deref().unwrap_or(&model.current_path),
                        set_hexpand: true,
                        set_xalign: 0.0,
                        set_ellipsize: gtk4::pango::EllipsizeMode::Middle,
                     },
                },
                
                gtk4::Button {
                    set_label: "<",
                    set_focusable: false,
                    connect_clicked => SidebarMsg::GoUp,
                }
            },
            
            gtk4::Separator {},
            
            
            // Directories List
            #[name(directories_sw)]
            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_min_content_height: 100,
                
                #[local_ref]
                directories_box -> gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 2,
                }
            },
            
            gtk4::Separator {},

            // Images List
            #[name(images_sw)]
            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_min_content_height: 100,
                
                #[local_ref]
                images_box -> gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_spacing: 2,
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let directories = FactoryVecDeque::builder()
            .launch(gtk4::Box::default())
            .forward(sender.input_sender(), |msg| msg);
            
        let images = FactoryVecDeque::builder()
            .launch(gtk4::Box::default())
            .forward(sender.input_sender(), |msg| msg);

        let mut model = SidebarModel {
            current_path: "/".to_string(), // Default path, maybe should be std::env::current_dir()
            directories,
            images,
            dir_sort: SortType::NameAsc,
            image_sort: SortType::NameAsc,
            spread_view: false, 
            selected_path: None,
            selected_dir_path: None,
            preview_archive_path: None,
            images_scrolled_window: None,
            directories_scrolled_window: None,
            loop_images: false,
            single_first_page: false,
            archives_on_top: true,
        };
        
        let _initial_path = PathBuf::from(&model.current_path);
        model.refresh_view();

        let directories_box = model.directories.widget();
        let images_box = model.images.widget();
        
        let widgets = view_output!();
        
        model.images_scrolled_window = Some(widgets.images_sw.clone());
        model.directories_scrolled_window = Some(widgets.directories_sw.clone());
        
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SidebarMsg::GoUp => {
                if self.preview_archive_path.is_some() {
                     self.preview_archive_path = None;
                     self.selected_dir_path = None;
                     let _ = _sender.output(SidebarOutput::PathChanged(self.current_path.clone()));
                     self.refresh_view();
                     self.directories.broadcast(DirectoryItemMsg::UpdateSelection(None));
                     return;
                }

                let current = PathBuf::from(&self.current_path);
                if let Some(parent) = current.parent() {
                     let parent_path = parent.to_path_buf();
                      self.current_path = parent_path.to_string_lossy().to_string();
                      let _ = _sender.output(SidebarOutput::PathChanged(self.current_path.clone()));
                      self.refresh_view();
                      
                       
                       // Removed eager OpenImage
                }
            }
            SidebarMsg::UpdatePath(path) => {
                self.current_path = path.to_string_lossy().to_string();
                self.preview_archive_path = None;
                self.selected_dir_path = None;
                let _ = _sender.output(SidebarOutput::PathChanged(self.current_path.clone()));
                self.refresh_view();
                self.refresh_view();
                // Removed eager OpenImage
            }
            SidebarMsg::OpenDirectory(path) => {
                let is_archive = path.extension().and_then(|s| s.to_str()).map_or(false, |ext| ext.eq_ignore_ascii_case("zip"));
                
                if is_archive {
                    self.preview_archive_path = Some(path.clone());
                    self.selected_dir_path = Some(path.clone());
                    
                    // Update current_path to parent of archive so directory list shows context
                    if let Some(parent) = path.parent() {
                        self.current_path = parent.to_string_lossy().to_string();
                    }
                    
                    let _ = _sender.output(SidebarOutput::PathChanged(path.to_string_lossy().to_string()));
                    
                    // Refresh view to populate BOTH directory list (from current_path) AND images (from preview_archive_path)
                    self.refresh_view();
                    
                    self.directories.broadcast(DirectoryItemMsg::UpdateSelection(Some(path.clone())));
                    
                    let sender_clone = _sender.clone();
                    gtk4::glib::timeout_add_local(
                        std::time::Duration::from_millis(100),
                        move || {
                            sender_clone.input(SidebarMsg::ScrollToSelection);
                            gtk4::glib::ControlFlow::Break
                        }
                    );

                } else {
                    self.current_path = path.to_string_lossy().to_string();
                    self.preview_archive_path = None;
                    self.selected_dir_path = None;
                    let _ = _sender.output(SidebarOutput::PathChanged(self.current_path.clone()));
                    self.refresh_view();
                    // Removed eager OpenImage
                }
            }
            SidebarMsg::OpenImage(path) => {
                self.selected_path = Some(path.clone());
                self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                let _ = _sender.output(SidebarOutput::OpenImage(path));
            }

            SidebarMsg::ChangeDirSort(sort) => {
                if self.dir_sort != sort {
                     self.dir_sort = sort;
                     self.refresh_view();
                     let _ = _sender.output(SidebarOutput::DirSortChanged(sort));
                }
            }
            SidebarMsg::ChangeImageSort(sort) => {
                if self.image_sort != sort {
                    self.image_sort = sort;
                    self.refresh_view(); // Actually we only need reload_images but safe to full refresh
                    let _ = _sender.output(SidebarOutput::ImageSortChanged(sort));
                }
            }
            SidebarMsg::UpdateSpreadMode(is_spread) => {
                self.spread_view = is_spread;
            }
            SidebarMsg::GetSpreadPages(path) => {
                 let mut paths = Vec::new();
                 let mut found_idx = None;
                 
                 for (i, item) in self.images.iter().enumerate() {
                     if item.path == path {
                         found_idx = Some(i);
                         break;
                     }
                 }
                 
                 if let Some(idx) = found_idx {
                     paths.push(path.clone());
                     if self.spread_view {
                         let is_forced_single = self.single_first_page && idx == 0;
                         if !is_forced_single {
                             // Check if current is portrait
                             if self.is_portrait(&path) {
                                 // Check next
                                 if let Some(next) = self.images.get(idx + 1) {
                                     if self.is_portrait(&next.path) {
                                         paths.push(next.path.clone());
                                     }
                                 }
                             }
                         }
                     }
                     let _ = _sender.output(SidebarOutput::SpreadPages(paths));
                 } else {
                     let _ = _sender.output(SidebarOutput::SpreadPages(vec![path]));
                 }
            }
            SidebarMsg::OpenNextImage(path) => {
                 let mut found_idx = None;
                 for (i, item) in self.images.iter().enumerate() {
                     if item.path == path {
                         found_idx = Some(i);
                         break;
                     }
                 }
                 if let Some(idx) = found_idx {
                     let mut jump = 1;
                      if self.spread_view {
                          let is_forced_single = self.single_first_page && idx == 0;
                          
                          if !is_forced_single {
                              if self.is_portrait(&path) {
                                  if let Some(next) = self.images.get(idx + 1) {
                                      if self.is_portrait(&next.path) {
                                          jump = 2;
                                      }
                                  }
                              }
                          }
                      }
                     
                     if let Some(target) = self.images.get(idx + jump) {
                        let _ = _sender.output(SidebarOutput::OpenImage(target.path.clone()));
                        self.selected_path = Some(target.path.clone());
                        self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                     } else {
                         if self.loop_images {
                             if let Some(first) = self.images.get(0) {
                                 let _ = _sender.output(SidebarOutput::OpenImage(first.path.clone()));
                                 self.selected_path = Some(first.path.clone());
                                 self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                             }
                         } else {
                             let _ = _sender.output(SidebarOutput::RequestNextDir);
                         }
                     }
                 }
            }
            SidebarMsg::OpenPrevImage(path) => {
                 let mut found_idx = None;
                 for (i, item) in self.images.iter().enumerate() {
                     if item.path == path {
                         found_idx = Some(i);
                         break;
                     }
                 }
                 if let Some(idx) = found_idx {
                     if idx > 0 {
                         let mut target_idx = idx - 1;
                         
                         if self.spread_view {
                             if idx >= 2 {
                                 let prev = self.images.get(idx - 1).unwrap();
                                 if self.is_portrait(&prev.path) {
                                     let prev_prev = self.images.get(idx - 2).unwrap();
                                     if self.is_portrait(&prev_prev.path) {
                                         target_idx = idx - 2;
                                     }
                                 }
                             }
                         }
                         
                         if let Some(target) = self.images.get(target_idx) {
                            let _ = _sender.output(SidebarOutput::OpenImage(target.path.clone()));
                            self.selected_path = Some(target.path.clone());
                            self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                         }
                     } else {
                           if self.loop_images {
                               if let Some(last) = self.images.back() {
                                    let _ = _sender.output(SidebarOutput::OpenImage(last.path.clone()));
                                    self.selected_path = Some(last.path.clone());
                                    self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                               }
                           } else {
                               let _ = _sender.output(SidebarOutput::RequestPrevDir);
                           }
                     }
                 }
            }
            SidebarMsg::OpenNextImageSingle(path) => {
                 let mut found_idx = None;
                 for (i, item) in self.images.iter().enumerate() {
                     if item.path == path {
                         found_idx = Some(i);
                         break;
                     }
                 }
                 if let Some(idx) = found_idx {
                     if let Some(next) = self.images.get(idx + 1) {
                         let _ = _sender.output(SidebarOutput::OpenImage(next.path.clone()));
                          self.selected_path = Some(next.path.clone());
                          self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                     }
                 }
            }
            SidebarMsg::OpenPrevImageSingle(path) => {
                 let mut found_idx = None;
                 for (i, item) in self.images.iter().enumerate() {
                     if item.path == path {
                         found_idx = Some(i);
                         break;
                     }
                 }
                 if let Some(idx) = found_idx {
                     if idx > 0 {
                         if let Some(prev) = self.images.get(idx - 1) {
                             let _ = _sender.output(SidebarOutput::OpenImage(prev.path.clone()));
                              self.selected_path = Some(prev.path.clone());
                              self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                         }
                     }
                 }
            }
             SidebarMsg::SelectImage(path) => {
                 self.selected_path = Some(path);
                 self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                 self.scroll_to_selected();
             }
             SidebarMsg::UpdateLoopImages(val) => {
                 self.loop_images = val;
             }
             SidebarMsg::UpdateSingleFirstPage(val) => {
                 self.single_first_page = val;
             }
             SidebarMsg::UpdateArchivesOnTop(val) => {
                 self.archives_on_top = val;
                 self.refresh_view();
             }
             SidebarMsg::OpenFirstImage => {
                 if let Some(first) = self.images.get(0) {
                     let _ = _sender.output(SidebarOutput::OpenImage(first.path.clone()));
                     self.selected_path = Some(first.path.clone());
                     self.images.broadcast(ImageItemMsg::UpdateSelection(self.selected_path.clone()));
                 } else {
                     let _ = _sender.output(SidebarOutput::ClearImage);
                 }
             }
             SidebarMsg::ScrollToSelection => {
                 self.scroll_to_selected_directory();
             }
        }
    }
}

impl SidebarModel {
    fn scroll_to_selected_directory(&self) {
        if let Some(sw) = &self.directories_scrolled_window {
            let mut found_idx = None;
            if let Some(target) = &self.selected_dir_path {
                for (i, item) in self.directories.iter().enumerate() {
                    if &item.path == target {
                        found_idx = Some(i);
                        break;
                    }
                }
            }
            
             if let Some(i) = found_idx {
                 let container = self.directories.widget();
                 let mut child = container.first_child();
                 for _ in 0..i {
                     child = child.and_then(|c: gtk4::Widget| c.next_sibling());
                 }
                 if let Some(w) = child {
                      let w: gtk4::Widget = w;
                      let adjustment = sw.vadjustment();
                      let alloc = w.allocation();
                      let y = alloc.y() as f64;
                      let height = alloc.height() as f64;
                      
                      let page_size = adjustment.page_size();
                      let current_val = adjustment.value();
                      
                      if y < current_val {
                          adjustment.set_value(y);
                      } else if y + height > current_val + page_size {
                          adjustment.set_value(y + height - page_size);
                      }
                 }
             }
        }
    }

    fn scroll_to_selected(&self) {
        if let Some(sw) = &self.images_scrolled_window {
            if let Some(path) = &self.selected_path {
                let mut idx = None;
                for (i, item) in self.images.iter().enumerate() {
                     if &item.path == path { idx = Some(i); break; }
                }
                if let Some(i) = idx {
                     let container = self.images.widget();
                     let mut child = container.first_child();
                     for _ in 0..i {
                         child = child.and_then(|c: gtk4::Widget| c.next_sibling());
                     }
                     if let Some(w) = child {
                          let w: gtk4::Widget = w;
                          let adjustment = sw.vadjustment();
                          let alloc = w.allocation();
                          let y = alloc.y() as f64;
                          let height = alloc.height() as f64;
                          
                          let page_size = adjustment.page_size();
                          let current_val = adjustment.value();
                          
                          if y < current_val {
                              adjustment.set_value(y);
                          } else if y + height > current_val + page_size {
                              adjustment.set_value(y + height - page_size);
                          }
                     }
                }
            }
        }
    }

    fn is_portrait(&self, path: &PathBuf) -> bool {
        if let Some((w, h)) = self.get_image_dimensions(path) {
            return h > w;
        }
        false
    }
    
    fn get_image_dimensions(&self, path: &PathBuf) -> Option<(u32, u32)> {
        use std::io::Read;
        if path.exists() && path.is_file() {
            if let Ok(reader) = ImageReader::open(path) {
                if let Ok(reader) = reader.with_guessed_format() {
                     if let Ok(dim) = reader.into_dimensions() {
                         return Some(dim);
                     }
                }
            }
        } else {
         // Zip handling
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
                                          let mut buffer = Vec::new();
                                          if entry.read_to_end(&mut buffer).is_ok() {
                                              if let Ok(reader) = ImageReader::new(std::io::Cursor::new(buffer)).with_guessed_format() {
                                                  if let Ok(dim) = reader.into_dimensions() {
                                                      return Some(dim);
                                                  }
                                              }
                                          }
                                      }
                                  }
                             }
                         }
                     }
                 }
                 break;
             }
             current = parent.to_path_buf();
         }
    }
    None
}

    fn refresh_view(&mut self) {
        self.reload_directories();
        self.reload_images();
    }
    
    fn reload_directories(&mut self) {
        let current_path = PathBuf::from(&self.current_path);
        let (dir_entries, _) = self.scan_directory(&current_path);
        
        {
            let mut dirs = self.directories.guard();
            dirs.clear();
            for (name, path, is_archive) in dir_entries {
                 let is_selected = Some(&path) == self.selected_dir_path.as_ref();
                 dirs.push_back((name, path, is_archive, is_selected));
            }
        }
        self.scroll_to_selected_directory();
    }

    fn reload_images(&mut self) {
        let current_path = PathBuf::from(&self.current_path);
        let image_source = self.preview_archive_path.as_ref().unwrap_or(&current_path);
        let (_, img_entries) = self.scan_directory(image_source);
        
        let mut imgs = self.images.guard();
        imgs.clear();
        for (i, (name, path)) in img_entries.into_iter().enumerate() {
             imgs.push_back((name, path, i + 1));
        }
    }

    fn scan_directory(&self, path: &PathBuf) -> (Vec<(String, PathBuf, bool)>, Vec<(String, PathBuf)>) {
        scan_directory_custom(path, &self.dir_sort, &self.image_sort, self.archives_on_top)
    }

}
pub fn scan_directory_custom(
    path: &PathBuf, 
    dir_sort: &SortType, 
    image_sort: &SortType, 
    archives_on_top: bool
) -> (Vec<(String, PathBuf, bool)>, Vec<(String, PathBuf)>) {
         let mut dir_entries = Vec::new();
         let mut img_entries = Vec::new();

        if path.is_file() {
            // Check for ZIP
             if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
                 if ext == "zip" {
                     if let Ok(file) = std::fs::File::open(&path) {
                         if let Ok(mut archive) = zip::ZipArchive::new(file) {
                             let mut raw_entries = Vec::new();
                             for i in 0..archive.len() {
                                 if let Ok(file) = archive.by_index(i) {
                                     let name = file.name().to_string();
                                     raw_entries.push(name);
                                 }
                             }
                             
                             for name in raw_entries {
                                 // Check extension of entry
                                 if is_supported_image(std::path::Path::new(&name)) {
                                     let entry_path = path.join(&name);
                                     img_entries.push((name, entry_path));
                                 }
                             }
                             // Sort Images in Archive
                             match image_sort {
                                SortType::NameAsc => img_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0)),
                                SortType::NameDesc => { img_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0)); img_entries.reverse(); },
                                SortType::DateAsc | SortType::DateDesc | SortType::SizeAsc | SortType::SizeDesc => {
                                    // Zip entries don't easily support metadata access without costly lookups.
                                    // Default to name sort for now or implement if needed.
                                    img_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0));
                                     if matches!(image_sort, SortType::DateDesc | SortType::SizeDesc) {
                                         img_entries.reverse();
                                     }
                                }
                             }
                         }
                     }
                 }
             }
        } else if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                         if !name.starts_with('.') {
                             dir_entries.push((name.to_string(), path, false));
                         }
                    }
                } else if path.is_file() {
                    // Check extension
                    if is_supported_image(&path) {
                         if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                             img_entries.push((name.to_string(), path));
                         }
                    } else if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
                        match ext.as_str() {
                            "zip" => {
                                // Archive is treated as directory
                                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                                    dir_entries.push((name.to_string(), path, true));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Sort Directories
            // First sort by key based on sort type
            match dir_sort {
                 SortType::NameAsc => dir_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0)),
                 SortType::NameDesc => { dir_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0)); dir_entries.reverse(); },
                 SortType::DateAsc => dir_entries.sort_by_key(|a| std::fs::metadata(&a.1).and_then(|m| m.modified()).ok()),
                 SortType::DateDesc => { dir_entries.sort_by_key(|a| std::fs::metadata(&a.1).and_then(|m| m.modified()).ok()); dir_entries.reverse(); },
                 SortType::SizeAsc => dir_entries.sort_by_key(|a| std::fs::metadata(&a.1).map(|m| m.len()).unwrap_or(0)),
                 SortType::SizeDesc => { dir_entries.sort_by_key(|a| std::fs::metadata(&a.1).map(|m| m.len()).unwrap_or(0)); dir_entries.reverse(); },
            }
            // Then stable sort by is_archive vs is_dir based on setting
            if archives_on_top {
                // Archives (is_archive = true) come first (Ordering::Less)
                dir_entries.sort_by(|a, b| {
                    match (a.2, b.2) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            } else {
                 // Dirs (is_archive = false) come first
                 dir_entries.sort_by(|a, b| {
                    match (a.2, b.2) {
                        (false, true) => std::cmp::Ordering::Less,
                        (true, false) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            }

            // Sort Images
            match image_sort {
                 SortType::NameAsc => img_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0)),
                 SortType::NameDesc => { img_entries.sort_by(|a, b| natural_lexical_cmp(&a.0, &b.0)); img_entries.reverse(); },
                 SortType::DateAsc => img_entries.sort_by_key(|a| std::fs::metadata(&a.1).and_then(|m| m.modified()).ok()),
                 SortType::DateDesc => { img_entries.sort_by_key(|a| std::fs::metadata(&a.1).and_then(|m| m.modified()).ok()); img_entries.reverse(); },
                 SortType::SizeAsc => img_entries.sort_by_key(|a| std::fs::metadata(&a.1).map(|m| m.len()).unwrap_or(0)),
                 SortType::SizeDesc => { img_entries.sort_by_key(|a| std::fs::metadata(&a.1).map(|m| m.len()).unwrap_or(0)); img_entries.reverse(); },
            }
        }
        (dir_entries, img_entries)
    }



