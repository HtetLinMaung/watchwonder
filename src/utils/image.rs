pub fn get_image_format_from_path(path: &str) -> Option<image::ImageFormat> {
    if path.ends_with(".png") {
        Some(image::ImageFormat::Png)
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        Some(image::ImageFormat::Jpeg)
    } else if path.ends_with(".gif") {
        Some(image::ImageFormat::Gif)
    } else if path.ends_with(".bmp") {
        Some(image::ImageFormat::Bmp)
    } else if path.ends_with(".ico") {
        Some(image::ImageFormat::Ico)
    } else if path.ends_with(".tiff") {
        Some(image::ImageFormat::Tiff)
    } else {
        None
    }
}
