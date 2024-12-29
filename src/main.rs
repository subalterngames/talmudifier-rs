use cosmic_text::{Attrs, FontSystem, Buffer, Metrics, Shaping};


fn main() {
    let mut font_system = FontSystem::new();
    let metrics = Metrics::new(14.0, 20.0);

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(500.), None);
    // Attributes indicate what font to choose
    let attrs = Attrs::new();
    buffer.set_text(&mut font_system, include_str!("lorem.txt"), attrs, Shaping::Advanced);
    buffer.shape_until_scroll(&mut font_system, true);
    println!("{:?}", buffer.layout_runs().count());
}
