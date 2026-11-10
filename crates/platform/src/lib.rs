//! Platform abstraction layer for Creative Engine
//!
//! This crate defines traits for platform-specific functionality that must be
//! implemented for each target platform (Linux, Windows, macOS, Android, iOS).

use creative_domain::{Point2, Color, Rect};
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Platform abstraction errors
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Credential error: {0}")]
    CredentialError(String),
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    #[error("Dialog cancelled")]
    DialogCancelled,
}

/// File dialog options
#[derive(Debug, Clone, Default)]
pub struct FileDialogOptions {
    pub title: String,
    pub directory: Option<PathBuf>,
    pub filters: Vec<FileFilter>,
    pub allow_multiple: bool,
    pub save_as: bool,
}

#[derive(Debug, Clone)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Platform-specific file system operations
pub trait FileSystem: Send + Sync {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, PlatformError>;
    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), PlatformError>;
    fn exists(&self, path: &Path) -> Result<bool, PlatformError>;
    fn create_dir_all(&self, path: &Path) -> Result<(), PlatformError>;
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, PlatformError>;
    fn metadata(&self, path: &Path) -> Result<FileMetadata, PlatformError>;
    fn delete_file(&self, path: &Path) -> Result<(), PlatformError>;
    fn delete_dir(&self, path: &Path) -> Result<(), PlatformError>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), PlatformError>;
    fn copy_file(&self, from: &Path, to: &Path) -> Result<(), PlatformError>;
    fn app_data_dir(&self) -> PathBuf;
    fn cache_dir(&self) -> PathBuf;
    fn documents_dir(&self) -> Option<PathBuf>;
    fn temp_dir(&self) -> PathBuf;
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub modified: std::time::SystemTime,
    pub created: Option<std::time::SystemTime>,
}

/// Platform-specific file dialogs
pub trait FileDialogs: Send + Sync {
    fn open_file(&self, options: FileDialogOptions) -> Result<Vec<PathBuf>, PlatformError>;
    fn save_file(&self, options: FileDialogOptions) -> Result<PathBuf, PlatformError>;
    fn open_folder(&self, options: FileDialogOptions) -> Result<PathBuf, PlatformError>;
}

/// Platform-specific clipboard operations
pub trait Clipboard: Send + Sync {
    fn get_text(&self) -> Result<String, PlatformError>;
    fn set_text(&self, text: &str) -> Result<(), PlatformError>;
    fn get_image(&self) -> Result<Option<ClipboardImage>, PlatformError>;
    fn set_image(&self, image: ClipboardImage) -> Result<(), PlatformError>;
    fn has_text(&self) -> Result<bool, PlatformError>;
    fn has_image(&self) -> Result<bool, PlatformError>;
}

#[derive(Debug, Clone)]
pub struct ClipboardImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Platform-specific secure credential storage
pub trait SecureStorage: Send + Sync {
    fn store(&self, key: &str, value: &[u8]) -> Result<(), PlatformError>;
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, PlatformError>;
    fn delete(&self, key: &str) -> Result<(), PlatformError>;
    fn list_keys(&self) -> Result<Vec<String>, PlatformError>;
}

/// Platform-specific input handling
pub trait InputHandler: Send + Sync {
    fn mouse_position(&self) -> Result<Point2, PlatformError>;
    fn is_key_pressed(&self, key: KeyCode) -> Result<bool, PlatformError>;
    fn is_mouse_pressed(&self, button: MouseButton) -> Result<bool, PlatformError>;
    fn touch_points(&self) -> Result<Vec<TouchPoint>, PlatformError>;
    fn stylus_state(&self) -> Result<Option<StylusState>, PlatformError>;
    fn set_cursor(&self, cursor: CursorIcon) -> Result<(), PlatformError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Escape, Enter, Tab, Space, Backspace, Delete,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Home, End, PageUp, PageDown,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,
    ControlLeft, ControlRight, ShiftLeft, ShiftRight,
    AltLeft, AltRight, MetaLeft, MetaRight,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left, Right, Middle, Back, Forward, Other(u16),
}

#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: u64,
    pub position: Point2,
    pub pressure: f32,
    pub radius_x: f32,
    pub radius_y: f32,
}

#[derive(Debug, Clone)]
pub struct StylusState {
    pub position: Point2,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub azimuth: f32,
    pub altitude: f32,
    pub button_pressed: bool,
    pub eraser: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorIcon {
    Default, Crosshair, Pointer, Move, Text, Wait, Help,
    Progress, NotAllowed, Cell, VerticalText, Alias, Copy,
    NoDrop, Grab, Grabbing, EResize, NResize, NeResize,
    NwResize, SResize, SeResize, SwResize, WResize,
    EwResize, NsResize, NeswResize, NwseResize,
    ColResize, RowResize, ZoomIn, ZoomOut,
}

/// Platform-specific display information
pub trait DisplayInfo: Send + Sync {
    fn scale_factor(&self) -> Result<f32, PlatformError>;
    fn monitors(&self) -> Result<Vec<MonitorInfo>, PlatformError>;
    fn safe_area(&self) -> Result<SafeArea, PlatformError>;
}

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub size: (u32, u32),
    pub position: (i32, i32),
    pub scale_factor: f32,
    pub is_primary: bool,
    pub refresh_rate: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct SafeArea {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

/// Platform-specific GPU surface creation
pub trait GpuSurface: Send + Sync {
    fn create_surface(&self, window_handle: WindowHandle) -> Result<SurfaceHandle, PlatformError>;
    fn preferred_format(&self) -> Result<SurfaceFormat, PlatformError>;
    fn surface_capabilities(&self, surface: &SurfaceHandle) -> Result<SurfaceCapabilities, PlatformError>;
}

#[derive(Debug, Clone)]
pub struct WindowHandle { pub raw: RawWindowHandle }
#[derive(Debug, Clone)]
pub struct SurfaceHandle { pub raw: RawSurfaceHandle }

#[derive(Debug, Clone)]
pub enum RawWindowHandle {
    #[cfg(target_os = "linux")] Wayland(WaylandWindowHandle),
    #[cfg(target_os = "linux")] X11(X11WindowHandle),
    #[cfg(target_os = "windows")] Windows(WindowsWindowHandle),
    #[cfg(target_os = "macos")] MacOS(MacOSWindowHandle),
    #[cfg(target_os = "android")] Android(AndroidWindowHandle),
    #[cfg(target_os = "ios")] IOS(IOSWindowHandle),
    Other,
}

#[derive(Debug, Clone)]
pub enum RawSurfaceHandle {
    #[cfg(target_os = "linux")] Wayland(WaylandSurfaceHandle),
    #[cfg(target_os = "linux")] X11(X11SurfaceHandle),
    #[cfg(target_os = "windows")] Windows(WindowsSurfaceHandle),
    #[cfg(target_os = "macos")] MacOS(MacOSSurfaceHandle),
    #[cfg(target_os = "android")] Android(AndroidSurfaceHandle),
    #[cfg(target_os = "ios")] IOS(IOSSurfaceHandle),
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct SurfaceFormat {
    pub format: wgpu::TextureFormat,
    pub color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace { Srgb, Linear, DisplayP3, Hdr10 }

#[derive(Debug, Clone)]
pub struct SurfaceCapabilities {
    pub formats: Vec<SurfaceFormat>,
    pub present_modes: Vec<wgpu::PresentMode>,
    pub max_extent: (u32, u32),
    pub min_extent: (u32, u32),
    pub current_extent: (u32, u32),
    pub alpha_modes: Vec<wgpu::CompositeAlphaMode>,
}

/// Platform-specific haptics
pub trait Haptics: Send + Sync {
    fn vibrate(&self, pattern: VibrationPattern) -> Result<(), PlatformError>;
    fn supported(&self) -> Result<bool, PlatformError>;
}

#[derive(Debug, Clone)]
pub enum VibrationPattern {
    Click, DoubleClick, HeavyClick, Success, Warning, Error,
    Custom { durations: Vec<u32>, amplitudes: Vec<f32> },
}

/// Platform-specific share functionality
pub trait Share: Send + Sync {
    fn share_text(&self, text: &str, subject: Option<&str>) -> Result<(), PlatformError>;
    fn share_file(&self, path: &Path, mime_type: &str) -> Result<(), PlatformError>;
    fn share_image(&self, image: ClipboardImage) -> Result<(), PlatformError>;
}

/// Platform-specific notifications
pub trait Notifications: Send + Sync {
    fn notify(&self, title: &str, body: &str, icon: Option<&Path>) -> Result<(), PlatformError>;
    fn request_permission(&self) -> Result<bool, PlatformError>;
}

/// Composite platform interface
pub trait Platform: FileSystem + FileDialogs + Clipboard + SecureStorage + InputHandler + DisplayInfo + GpuSurface + Haptics + Share + Notifications {
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
}

// Placeholder types for window/surface handles
#[derive(Debug, Clone, Copy)] pub struct WaylandWindowHandle;
#[derive(Debug, Clone, Copy)] pub struct X11WindowHandle;
#[derive(Debug, Clone, Copy)] pub struct WindowsWindowHandle;
#[derive(Debug, Clone, Copy)] pub struct MacOSWindowHandle;
#[derive(Debug, Clone, Copy)] pub struct AndroidWindowHandle;
#[derive(Debug, Clone, Copy)] pub struct IOSWindowHandle;
#[derive(Debug, Clone, Copy)] pub struct WaylandSurfaceHandle;
#[derive(Debug, Clone, Copy)] pub struct X11SurfaceHandle;
#[derive(Debug, Clone, Copy)] pub struct WindowsSurfaceHandle;
#[derive(Debug, Clone, Copy)] pub struct MacOSSurfaceHandle;
#[derive(Debug, Clone, Copy)] pub struct AndroidSurfaceHandle;
#[derive(Debug, Clone, Copy)] pub struct IOSSurfaceHandle;

// Desktop implementations
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub mod desktop {
    use super::*;
    use std::sync::Arc;
    use std::path::Path;
    
    pub struct DesktopFileSystem;
    
    impl FileSystem for DesktopFileSystem {
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, PlatformError> {
            std::fs::read(path).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), PlatformError> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| PlatformError::IoError(e.to_string()))?;
            }
            let temp_path = path.with_extension("tmp");
            std::fs::write(&temp_path, data).map_err(|e| PlatformError::IoError(e.to_string()))?;
            std::fs::rename(&temp_path, path).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn exists(&self, path: &Path) -> Result<bool, PlatformError> { Ok(path.exists()) }
        
        fn create_dir_all(&self, path: &Path) -> Result<(), PlatformError> {
            std::fs::create_dir_all(path).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, PlatformError> {
            let mut entries = Vec::new();
            for entry in std::fs::read_dir(path).map_err(|e| PlatformError::IoError(e.to_string()))? {
                let entry = entry.map_err(|e| PlatformError::IoError(e.to_string()))?;
                let path = entry.path();
                let metadata = entry.metadata().map_err(|e| PlatformError::IoError(e.to_string()))?;
                entries.push(DirEntry {
                    path: path.clone(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    is_file: metadata.is_file(),
                    size: if metadata.is_file() { Some(metadata.len()) } else { None },
                    modified: metadata.modified().ok(),
                });
            }
            Ok(entries)
        }
        
        fn metadata(&self, path: &Path) -> Result<FileMetadata, PlatformError> {
            let metadata = std::fs::metadata(path).map_err(|e| PlatformError::IoError(e.to_string()))?;
            Ok(FileMetadata {
                size: metadata.len(),
                is_dir: metadata.is_dir(),
                is_file: metadata.is_file(),
                modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                created: metadata.created().ok(),
            })
        }
        
        fn delete_file(&self, path: &Path) -> Result<(), PlatformError> {
            std::fs::remove_file(path).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn delete_dir(&self, path: &Path) -> Result<(), PlatformError> {
            std::fs::remove_dir_all(path).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn rename(&self, from: &Path, to: &Path) -> Result<(), PlatformError> {
            std::fs::rename(from, to).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn copy_file(&self, from: &Path, to: &Path) -> Result<(), PlatformError> {
            std::fs::copy(from, to).map_err(|e| PlatformError::IoError(e.to_string()))?;
            Ok(())
        }
        
        fn app_data_dir(&self) -> PathBuf {
            directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.data_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        }
        
        fn cache_dir(&self) -> PathBuf {
            directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.cache_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        }
        
        fn documents_dir(&self) -> Option<PathBuf> {
            directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.data_local_dir().to_path_buf())
        }
        
        fn temp_dir(&self) -> PathBuf { std::env::temp_dir() }
    }
    
    pub struct DesktopClipboard;
    
    impl DesktopClipboard {
        pub fn new() -> Result<Self, PlatformError> {
            Ok(Self)
        }
    }
    
    impl Clipboard for DesktopClipboard {
        fn get_text(&self) -> Result<String, PlatformError> { Err(PlatformError::Unsupported("Use Iced clipboard".to_string())) }
        fn set_text(&self, _text: &str) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Use Iced clipboard".to_string())) }
        fn get_image(&self) -> Result<Option<ClipboardImage>, PlatformError> { Err(PlatformError::Unsupported("Use Iced clipboard".to_string())) }
        fn set_image(&self, _image: ClipboardImage) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Use Iced clipboard".to_string())) }
        fn has_text(&self) -> Result<bool, PlatformError> { Ok(false) }
        fn has_image(&self) -> Result<bool, PlatformError> { Ok(false) }
    }
    
    pub struct DesktopSecureStorage;
    
    impl SecureStorage for DesktopSecureStorage {
        fn store(&self, key: &str, value: &[u8]) -> Result<(), PlatformError> {
            let dir = directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.data_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&dir).map_err(|e| PlatformError::IoError(e.to_string()))?;
            let file = dir.join(format!("{}.cred", key));
            std::fs::write(file, value).map_err(|e| PlatformError::IoError(e.to_string()))
        }
        
        fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, PlatformError> {
            let dir = directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.data_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let file = dir.join(format!("{}.cred", key));
            match std::fs::read(file) {
                Ok(data) => Ok(Some(data)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(PlatformError::IoError(e.to_string())),
            }
        }
        
        fn delete(&self, key: &str) -> Result<(), PlatformError> {
            let dir = directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.data_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let file = dir.join(format!("{}.cred", key));
            match std::fs::remove_file(file) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(PlatformError::IoError(e.to_string())),
            }
        }
        
        fn list_keys(&self) -> Result<Vec<String>, PlatformError> {
            let dir = directories::ProjectDirs::from("com", "creative", "engine")
                .map(|dirs| dirs.data_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let mut keys = Vec::new();
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if let Some(key) = name.strip_suffix(".cred") {
                            keys.push(key.to_string());
                        }
                    }
                }
            }
            Ok(keys)
        }
    }
    
    pub struct DesktopFileDialogs;
    
    impl FileDialogs for DesktopFileDialogs {
        fn open_file(&self, _options: FileDialogOptions) -> Result<Vec<PathBuf>, PlatformError> {
            Err(PlatformError::Unsupported("Use Iced file dialogs".to_string()))
        }
        fn save_file(&self, _options: FileDialogOptions) -> Result<PathBuf, PlatformError> {
            Err(PlatformError::Unsupported("Use Iced file dialogs".to_string()))
        }
        fn open_folder(&self, _options: FileDialogOptions) -> Result<PathBuf, PlatformError> {
            Err(PlatformError::Unsupported("Use Iced file dialogs".to_string()))
        }
    }
    
    pub struct DesktopInputHandler;
    impl InputHandler for DesktopInputHandler {
        fn mouse_position(&self) -> Result<Point2, PlatformError> { Err(PlatformError::Unsupported("Use Iced event loop".to_string())) }
        fn is_key_pressed(&self, _: KeyCode) -> Result<bool, PlatformError> { Err(PlatformError::Unsupported("Use Iced event loop".to_string())) }
        fn is_mouse_pressed(&self, _: MouseButton) -> Result<bool, PlatformError> { Err(PlatformError::Unsupported("Use Iced event loop".to_string())) }
        fn touch_points(&self) -> Result<Vec<TouchPoint>, PlatformError> { Ok(Vec::new()) }
        fn stylus_state(&self) -> Result<Option<StylusState>, PlatformError> { Ok(None) }
        fn set_cursor(&self, _: CursorIcon) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Use Iced window".to_string())) }
    }
    
    pub struct DesktopDisplayInfo;
    impl DisplayInfo for DesktopDisplayInfo {
        fn scale_factor(&self) -> Result<f32, PlatformError> { Ok(1.0) }
        fn monitors(&self) -> Result<Vec<MonitorInfo>, PlatformError> { Ok(vec![MonitorInfo { name: "Primary".to_string(), size: (1920, 1080), position: (0, 0), scale_factor: 1.0, is_primary: true, refresh_rate: Some(60) }]) }
        fn safe_area(&self) -> Result<SafeArea, PlatformError> { Ok(SafeArea::default()) }
    }
    
    pub struct DesktopGpuSurface;
    impl GpuSurface for DesktopGpuSurface {
        fn create_surface(&self, _: WindowHandle) -> Result<SurfaceHandle, PlatformError> { Err(PlatformError::Unsupported("Use Iced wgpu integration".to_string())) }
        fn preferred_format(&self) -> Result<SurfaceFormat, PlatformError> { Ok(SurfaceFormat { format: wgpu::TextureFormat::Bgra8UnormSrgb, color_space: ColorSpace::Srgb }) }
        fn surface_capabilities(&self, _: &SurfaceHandle) -> Result<SurfaceCapabilities, PlatformError> { Err(PlatformError::Unsupported("Not implemented".to_string())) }
    }
    
    pub struct DesktopHaptics;
    impl Haptics for DesktopHaptics { fn vibrate(&self, _: VibrationPattern) -> Result<(), PlatformError> { Ok(()) } fn supported(&self) -> Result<bool, PlatformError> { Ok(false) } }
    
    pub struct DesktopShare;
    impl Share for DesktopShare { fn share_text(&self, _: &str, _: Option<&str>) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Not implemented".to_string())) } fn share_file(&self, _: &Path, _: &str) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Not implemented".to_string())) } fn share_image(&self, _: ClipboardImage) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Not implemented".to_string())) } }
    
    pub struct DesktopNotifications;
    impl Notifications for DesktopNotifications { fn notify(&self, _: &str, _: &str, _: Option<&Path>) -> Result<(), PlatformError> { Err(PlatformError::Unsupported("Use system notification library".to_string())) } fn request_permission(&self) -> Result<bool, PlatformError> { Ok(true) } }
    
    pub struct DesktopPlatform { 
        pub file_system: DesktopFileSystem, 
        pub clipboard: DesktopClipboard, 
        pub secure_storage: DesktopSecureStorage, 
        pub file_dialogs: DesktopFileDialogs, 
        pub input: DesktopInputHandler, 
        pub display: DesktopDisplayInfo, 
        pub gpu: DesktopGpuSurface, 
        pub haptics: DesktopHaptics, 
        pub share: DesktopShare, 
        pub notifications: DesktopNotifications 
    }
    
    impl DesktopPlatform { 
        pub fn new() -> Result<Self, PlatformError> { 
            Ok(Self { 
                file_system: DesktopFileSystem, 
                clipboard: DesktopClipboard, 
                secure_storage: DesktopSecureStorage, 
                file_dialogs: DesktopFileDialogs, 
                input: DesktopInputHandler, 
                display: DesktopDisplayInfo, 
                gpu: DesktopGpuSurface, 
                haptics: DesktopHaptics, 
                share: DesktopShare, 
                notifications: DesktopNotifications 
            }) 
        } 
    }
    
    impl FileSystem for DesktopPlatform {
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, PlatformError> { self.file_system.read_file(path) }
        fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), PlatformError> { self.file_system.write_file(path, data) }
        fn exists(&self, path: &Path) -> Result<bool, PlatformError> { self.file_system.exists(path) }
        fn create_dir_all(&self, path: &Path) -> Result<(), PlatformError> { self.file_system.create_dir_all(path) }
        fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, PlatformError> { self.file_system.read_dir(path) }
        fn metadata(&self, path: &Path) -> Result<FileMetadata, PlatformError> { self.file_system.metadata(path) }
        fn delete_file(&self, path: &Path) -> Result<(), PlatformError> { self.file_system.delete_file(path) }
        fn delete_dir(&self, path: &Path) -> Result<(), PlatformError> { self.file_system.delete_dir(path) }
        fn rename(&self, from: &Path, to: &Path) -> Result<(), PlatformError> { self.file_system.rename(from, to) }
        fn copy_file(&self, from: &Path, to: &Path) -> Result<(), PlatformError> { self.file_system.copy_file(from, to) }
        fn app_data_dir(&self) -> PathBuf { self.file_system.app_data_dir() }
        fn cache_dir(&self) -> PathBuf { self.file_system.cache_dir() }
        fn documents_dir(&self) -> Option<PathBuf> { self.file_system.documents_dir() }
        fn temp_dir(&self) -> PathBuf { self.file_system.temp_dir() }
    }
    
    impl FileDialogs for DesktopPlatform {
        fn open_file(&self, options: FileDialogOptions) -> Result<Vec<PathBuf>, PlatformError> { self.file_dialogs.open_file(options) }
        fn save_file(&self, options: FileDialogOptions) -> Result<PathBuf, PlatformError> { self.file_dialogs.save_file(options) }
        fn open_folder(&self, options: FileDialogOptions) -> Result<PathBuf, PlatformError> { self.file_dialogs.open_folder(options) }
    }
    
    impl Clipboard for DesktopPlatform {
        fn get_text(&self) -> Result<String, PlatformError> { self.clipboard.get_text() }
        fn set_text(&self, text: &str) -> Result<(), PlatformError> { self.clipboard.set_text(text) }
        fn get_image(&self) -> Result<Option<ClipboardImage>, PlatformError> { self.clipboard.get_image() }
        fn set_image(&self, image: ClipboardImage) -> Result<(), PlatformError> { self.clipboard.set_image(image) }
        fn has_text(&self) -> Result<bool, PlatformError> { self.clipboard.has_text() }
        fn has_image(&self) -> Result<bool, PlatformError> { self.clipboard.has_image() }
    }
    
    impl SecureStorage for DesktopPlatform {
        fn store(&self, key: &str, value: &[u8]) -> Result<(), PlatformError> { self.secure_storage.store(key, value) }
        fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, PlatformError> { self.secure_storage.retrieve(key) }
        fn delete(&self, key: &str) -> Result<(), PlatformError> { self.secure_storage.delete(key) }
        fn list_keys(&self) -> Result<Vec<String>, PlatformError> { self.secure_storage.list_keys() }
    }
    
    impl InputHandler for DesktopPlatform {
        fn mouse_position(&self) -> Result<Point2, PlatformError> { self.input.mouse_position() }
        fn is_key_pressed(&self, key: KeyCode) -> Result<bool, PlatformError> { self.input.is_key_pressed(key) }
        fn is_mouse_pressed(&self, button: MouseButton) -> Result<bool, PlatformError> { self.input.is_mouse_pressed(button) }
        fn touch_points(&self) -> Result<Vec<TouchPoint>, PlatformError> { self.input.touch_points() }
        fn stylus_state(&self) -> Result<Option<StylusState>, PlatformError> { self.input.stylus_state() }
        fn set_cursor(&self, cursor: CursorIcon) -> Result<(), PlatformError> { self.input.set_cursor(cursor) }
    }
    
    impl DisplayInfo for DesktopPlatform {
        fn scale_factor(&self) -> Result<f32, PlatformError> { self.display.scale_factor() }
        fn monitors(&self) -> Result<Vec<MonitorInfo>, PlatformError> { self.display.monitors() }
        fn safe_area(&self) -> Result<SafeArea, PlatformError> { self.display.safe_area() }
    }
    
    impl GpuSurface for DesktopPlatform {
        fn create_surface(&self, window_handle: WindowHandle) -> Result<SurfaceHandle, PlatformError> { self.gpu.create_surface(window_handle) }
        fn preferred_format(&self) -> Result<SurfaceFormat, PlatformError> { self.gpu.preferred_format() }
        fn surface_capabilities(&self, surface: &SurfaceHandle) -> Result<SurfaceCapabilities, PlatformError> { self.gpu.surface_capabilities(surface) }
    }
    
    impl Haptics for DesktopPlatform {
        fn vibrate(&self, pattern: VibrationPattern) -> Result<(), PlatformError> { self.haptics.vibrate(pattern) }
        fn supported(&self) -> Result<bool, PlatformError> { self.haptics.supported() }
    }
    
    impl Share for DesktopPlatform {
        fn share_text(&self, text: &str, subject: Option<&str>) -> Result<(), PlatformError> { self.share.share_text(text, subject) }
        fn share_file(&self, path: &Path, mime_type: &str) -> Result<(), PlatformError> { self.share.share_file(path, mime_type) }
        fn share_image(&self, image: ClipboardImage) -> Result<(), PlatformError> { self.share.share_image(image) }
    }
    
    impl Notifications for DesktopPlatform {
        fn notify(&self, title: &str, body: &str, icon: Option<&Path>) -> Result<(), PlatformError> { self.notifications.notify(title, body, icon) }
        fn request_permission(&self) -> Result<bool, PlatformError> { self.notifications.request_permission() }
    }
    
    impl Platform for DesktopPlatform { 
        fn name(&self) -> &'static str { "Desktop" } 
        fn version(&self) -> String { format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH) } 
    }
}

// Android stub
#[cfg(target_os = "android")] pub mod android { use super::*; pub struct AndroidPlatform; impl Platform for AndroidPlatform { fn name(&self) -> &'static str { "Android" } fn version(&self) -> String { std::env::consts::OS.to_string() } } }

// iOS stub
#[cfg(target_os = "ios")] pub mod ios { use super::*; pub struct IOSPlatform; impl Platform for IOSPlatform { fn name(&self) -> &'static str { "iOS" } fn version(&self) -> String { std::env::consts::OS.to_string() } } }

pub fn create_platform() -> Result<Box<dyn Platform>, PlatformError> {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))] { Ok(Box::new(desktop::DesktopPlatform::new()?)) }
    #[cfg(target_os = "android")] { Ok(Box::new(android::AndroidPlatform)) }
    #[cfg(target_os = "ios")] { Ok(Box::new(ios::IOSPlatform)) }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos", target_os = "android", target_os = "ios")))] { Err(PlatformError::Unsupported("Platform not supported".to_string())) }
}

#[cfg(test)] mod tests { use super::*; #[test] fn test_file_dialog_options_default() { let options = FileDialogOptions::default(); assert!(options.title.is_empty()); assert!(options.directory.is_none()); } #[test] fn test_key_code_variants() { let _ = KeyCode::Escape; let _ = KeyCode::A; } #[test] fn test_vibration_patterns() { let _ = VibrationPattern::Click; let _ = VibrationPattern::Custom { durations: vec![100], amplitudes: vec![1.0] }; } }
