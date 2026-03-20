use tray_icon::Icon as TrayIcon;

pub fn create_icon() -> TrayIcon {
    let (w, h) = (22u32, 22u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);

            // "W" letter shape
            let visible = (0.2..0.8).contains(&fy)
                && ((fx - 0.15).abs() < 0.08
                    || (fx - 0.85).abs() < 0.08
                    || ((fx - 0.35).abs() < 0.08 && fy > 0.5)
                    || ((fx - 0.65).abs() < 0.08 && fy > 0.5)
                    || (fy > 0.7 && (0.15..0.35).contains(&fx))
                    || (fy > 0.7 && (0.65..0.85).contains(&fx))
                    || ((0.6..0.7).contains(&fy) && (0.35..0.65).contains(&fx)));

            if visible {
                rgba[i..i + 4].copy_from_slice(&[30, 130, 230, 255]);
            }
        }
    }

    TrayIcon::from_rgba(rgba, w, h).expect("Failed to create tray icon")
}
