use gtk4::gdk;
use gtk4::gdk_pixbuf::PixbufLoader;
use gtk4::prelude::*;

// Embed SVG assets directly into the binary as string literals
// Adjusted with fill="currentColor" and fill-opacity="0.5" for better visibility

const VIEW_SPREAD_ON: &[u8] = br##"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M2 6C2 4.89543 2.89543 4 4 4H10C11.1046 4 12 4.89543 12 6V20C12 21.1046 11.1046 22 10 22H4C2.89543 22 2 21.1046 2 20V6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff" fill-opacity="0.5"/><path d="M22 6C22 4.89543 21.1046 4 20 4H14C12.8954 4 12 4.89543 12 6V20C12 21.1046 12.8954 22 14 22H20C21.1046 22 22 21.1046 22 20V6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff" fill-opacity="0.5"/></svg>"##;

const VIEW_SPREAD_OFF: &[u8] = br##"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M6 4C4.89543 4 4 4.89543 4 6V20C4 21.1046 4.89543 22 6 22H18C19.1046 22 20 21.1046 20 20V6C20 4.89543 19.1046 4 18 4H6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff" fill-opacity="0.5"/><path d="M18 4V20" stroke="#ffffff" stroke-width="2" stroke-linejoin="round"/></svg>"##;

const VIEW_BINDING_LEFT: &[u8] = br##"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path opacity="0.3" d="M2 6C2 4.89543 2.89543 4 4 4H10C11.1046 4 12 4.89543 12 6V20C12 21.1046 11.1046 22 10 22H4C2.89543 22 2 21.1046 2 20V6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff"/><path opacity="0.3" d="M22 6C22 4.89543 21.1046 4 20 4H14C12.8954 4 12 4.89543 12 6V20C12 21.1046 12.8954 22 14 22H20C21.1046 22 22 21.1046 22 20V6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff"/><path d="M7 13H17" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M14 10L17 13L14 16" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

const VIEW_BINDING_RIGHT: &[u8] = br##"<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path opacity="0.3" d="M2 6C2 4.89543 2.89543 4 4 4H10C11.1046 4 12 4.89543 12 6V20C12 21.1046 11.1046 22 10 22H4C2.89543 22 2 21.1046 2 20V6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff"/><path opacity="0.3" d="M22 6C22 4.89543 21.1046 4 20 4H14C12.8954 4 12 4.89543 12 6V20C12 21.1046 12.8954 22 14 22H20C21.1046 22 22 21.1046 22 20V6Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" fill="#ffffff"/><path d="M17 13H7" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M10 10L7 13L10 16" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

fn load_icon(bytes: &[u8]) -> Option<gdk::Paintable> {
    let loader = PixbufLoader::new();
    if let Err(_) = loader.write(bytes) {
        return None;
    }
    if let Err(_) = loader.close() {
        return None;
    }
    loader.pixbuf().map(|pixbuf| gdk::Texture::for_pixbuf(&pixbuf).upcast())
}


pub fn spread_on() -> Option<gdk::Paintable> {
    load_icon(VIEW_SPREAD_ON)
}

pub fn spread_off() -> Option<gdk::Paintable> {
    load_icon(VIEW_SPREAD_OFF)
}

pub fn binding_left() -> Option<gdk::Paintable> {
    load_icon(VIEW_BINDING_LEFT)
}

pub fn binding_right() -> Option<gdk::Paintable> {
    load_icon(VIEW_BINDING_RIGHT)
}
