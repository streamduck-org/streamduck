use std::fs;
use std::sync::Arc;
use rusttype::Font;

static mut LOADED_FONTS: Vec<(String, Arc<Font<'static>>)> = vec![];

/// Adds font to global collection
pub fn add_font_to_collection(name: String, font: Font<'static>) {
    unsafe {
        LOADED_FONTS.push((name, Arc::new(font)));
    }
}

/// Loads default font for everything
pub fn load_default_font() {
    let bytes = include_bytes!("DejaVuSans.ttf").to_vec();
    if let Some(font) = Font::try_from_vec(bytes) {
        add_font_to_collection("default".to_string(), font);
    }
}

/// Loads fonts into global collection from fonts folder
pub fn load_fonts_from_resources() {
    let mut counter = 0;

    match fs::read_dir("fonts") {
        Ok(directory) => {
            for entry in directory {
                if let Ok(entry) = entry {
                    if entry.path().is_file() {
                        match fs::read(entry.path()) {
                            Ok(bytes) => {
                                if let Some(font) = Font::try_from_vec(bytes) {
                                    add_font_to_collection(entry.file_name().to_string_lossy().to_string(), font);
                                    counter += 1;
                                } else {
                                    log::error!("Failed to load {:?}: Not a font file", entry.file_name())
                                }
                            }
                            Err(err) => {
                                log::error!("Failed to load {:?}: {}", entry.file_name(), err)
                            }
                        }
                    }
                }
            }

            log::info!("Loaded {} fonts", counter);
        }
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                log::info!("Loaded no external fonts, missing fonts folder");
            } else {
                log::error!("Failed to locate fonts folder: {:?}", e);
            }
        }
    }
}

/// Gets font reference from global collection
pub fn get_font_from_collection(name: &str) -> Option<Arc<Font<'static>>> {
    for (font_name, font) in unsafe { &LOADED_FONTS } {
        if *font_name == name {
            return Some(font.clone())
        }
    }

    None
}

/// Returns names of fonts in global collection
pub fn get_font_names() -> Vec<String> {
    unsafe { &LOADED_FONTS }.iter().map(|(n, ..)| n.to_string()).collect()
}