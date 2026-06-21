//! Estrazione dell'icona di un eseguibile Windows e conversione in PNG
//! (data URL base64) per il frontend. Con cache per path.

use std::collections::HashMap;
use std::sync::Mutex;

use base64::Engine;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

/// Cache: path eseguibile -> data URL PNG (o None se non estraibile).
pub static CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);

/// Ritorna l'icona del processo come data URL PNG, usando la cache.
pub fn icon_for(exe: &str) -> Option<String> {
    {
        let guard = CACHE.lock().ok()?;
        if let Some(map) = guard.as_ref() {
            if let Some(cached) = map.get(exe) {
                return cached.clone();
            }
        }
    }

    let result = extract(exe);

    if let Ok(mut guard) = CACHE.lock() {
        guard
            .get_or_insert_with(HashMap::new)
            .insert(exe.to_string(), result.clone());
    }
    result
}

fn extract(exe: &str) -> Option<String> {
    let wide: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut info = SHFILEINFOW::default();
        let ret = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if ret == 0 || info.hIcon.is_invalid() {
            return None;
        }

        let png = hicon_to_png(info.hIcon);
        let _ = DestroyIcon(info.hIcon);

        let bytes = png?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        Some(format!("data:image/png;base64,{b64}"))
    }
}

unsafe fn hicon_to_png(hicon: HICON) -> Option<Vec<u8>> {
    let mut icon_info = ICONINFO::default();
    GetIconInfo(hicon, &mut icon_info).ok()?;

    let color = icon_info.hbmColor;
    let mask = icon_info.hbmMask;

    // Dimensioni dalla bitmap colore.
    let mut bmp = BITMAP::default();
    let got = GetObjectW(
        HGDIOBJ(color.0),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bmp as *mut _ as *mut _),
    );
    if got == 0 {
        cleanup_bitmaps(color, mask);
        return None;
    }
    let width = bmp.bmWidth;
    let height = bmp.bmHeight;
    if width <= 0 || height <= 0 {
        cleanup_bitmaps(color, mask);
        return None;
    }

    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let dc = CreateCompatibleDC(None);
    let mut buf = vec![0u8; (width * height * 4) as usize];
    let lines = GetDIBits(
        dc,
        color,
        0,
        height as u32,
        Some(buf.as_mut_ptr() as *mut _),
        &mut bi,
        DIB_RGB_COLORS,
    );
    let _ = DeleteDC(dc);
    cleanup_bitmaps(color, mask);

    if lines == 0 {
        return None;
    }

    // BGRA -> RGBA, con fallback alpha se tutto trasparente.
    let any_alpha = buf.chunks_exact(4).any(|p| p[3] != 0);
    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2); // B<->R
        if !any_alpha {
            px[3] = 255;
        }
    }

    encode_png(width as u32, height as u32, &buf)
}

unsafe fn cleanup_bitmaps(
    color: windows::Win32::Graphics::Gdi::HBITMAP,
    mask: windows::Win32::Graphics::Gdi::HBITMAP,
) {
    if !color.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(color.0));
    }
    if !mask.is_invalid() {
        let _ = DeleteObject(HGDIOBJ(mask.0));
    }
}

fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};

    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .ok()?;
    Some(out)
}
