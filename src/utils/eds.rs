use gdk::glib;
use zbus::zvariant::OwnedObjectPath;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SourceInfo {
    pub uid: String,
    pub path: OwnedObjectPath,
    pub display_name: String,
    pub enabled: bool,
    pub backend_name: String,
}

pub fn parse_source_data(path: OwnedObjectPath, uid: String, data: String) -> Option<SourceInfo> {
    let key_file = glib::KeyFile::new();
    key_file
        .load_from_data(&data, glib::KeyFileFlags::NONE)
        .ok()?;

    // Check if source is enabled
    let enabled = key_file.boolean("Data Source", "Enabled").unwrap_or(false);
    if !enabled {
        return None;
    }

    let display_name = key_file
        .string("Data Source", "DisplayName")
        .unwrap_or_else(|_| "Unknown".into())
        .to_string();

    // Check what type of source this is
    let backend_name = if key_file.has_group("Calendar") {
        let backend_name = key_file
            .string("Calendar", "BackendName")
            .unwrap_or_else(|_| "unknown".into());
        backend_name.to_string()
    } else {
        return None;
    };

    Some(SourceInfo {
        uid,
        path,
        display_name,
        enabled,
        backend_name,
    })
}
