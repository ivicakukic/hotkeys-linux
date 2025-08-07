/// Cairo-based rendering for the 3x3 board window
/// Handles all drawing operations for board display

use crate::core::{Board, ColorScheme, ModifierState, Pad, TextStyle, Resources};
use super::layout::{BoardLayout, Rect};
use std::fs::File;

// use gtk4::prelude::*;
use gtk4::cairo::{Context, FontSlant, FontWeight, ImageSurface};
use pango::{FontDescription, Weight};
use pangocairo::functions as pangocairo;


pub fn draw_board(ctx: &Context, board: &dyn Board, layout: &BoardLayout, resources: &Resources, selected_pad: Option<u8>, remaining_time: Option<u64>, current_modifiers: &ModifierState) {
    BoardRenderer::new(
        board.color_scheme(), board.text_style(), layout, resources
    ).draw_board(ctx, board, selected_pad, remaining_time, current_modifiers);
}



struct BoardRenderer<'a> {
    color_scheme: &'a ColorScheme,
    text_style: &'a TextStyle,
    layout: &'a BoardLayout,
    resources: &'a Resources,
}

impl<'a> BoardRenderer<'a> {
    /// Create new renderer with configuration
    fn new(color_scheme: &'a ColorScheme, text_style: &'a TextStyle, layout: &'a BoardLayout, resources: &'a Resources) -> Self {
        Self {
            color_scheme,
            text_style,
            layout,
            resources,
        }
    }

    /// Draw the complete 3x3 board using Board interface
    fn draw_board(&self, ctx: &Context, board: &dyn Board, selected_pad: Option<u8>, remaining_time: Option<u64>, current_modifiers: &ModifierState) {
        let fg1_color = self.color_scheme.foreground1().to_rgb();
        let fg2_color = self.color_scheme.foreground2().to_rgb();

        // Draw header using layout dimensions
        self.draw_header(ctx, board.title(), &fg2_color, board.icon());

        // Draw countdown timer if active
        if let Some(time_left) = remaining_time {
            if time_left > 0 {
                self.draw_countdown(ctx, time_left, &fg2_color);
            }
        }

        // Draw grid lines using layout calculations
        self.draw_grid_lines(ctx, &fg1_color);

        // Draw tiles
        for tile_id in 1..=9 {
            let is_selected = selected_pad == Some(tile_id);

            // Determine which pad to use based on current modifier state - using Board interface
            let pad = board.pads(Some(current_modifiers.clone())).get_or_default((tile_id - 1) as usize);

            // Get tile rectangle from layout
            if let Some(tile_rect) = self.layout.get_tile_rect(tile_id) {
                self.draw_tile(ctx, &pad, tile_id, tile_rect, is_selected);
            }
        }
    }


    /// Draw header with board name using layout dimensions
    fn draw_header(&self, ctx: &Context, name: &str, color: &(f64, f64, f64), icon: Option<&str>) {
        let header_rect = self.layout.get_header_rect();

        ctx.set_source_rgba(color.0, color.1, color.2, 1.0);
        apply_text_style(ctx, &self.text_style.header_font, "Impact");

        let text_extents = ctx.text_extents(name).unwrap();
        let h_extents = ctx.text_extents("H").unwrap();

        // Calculate icon size based on text height
        let icon_size_addition = 10.0; // How much is added to text height for icon
        let icon_size = h_extents.height() + icon_size_addition;
        let icon_spacing = 8.0; // Space between icon and text

        // Calculate total width (icon + spacing + text) for centering
        let total_width = if icon.is_some() {
            icon_size + icon_spacing + text_extents.width()
        } else {
            text_extents.width()
        };

        // Center the combined icon+text
        let start_x = (header_rect.width() - total_width) / 2.0;
        let text_y = header_rect.height() / 2.0 + text_extents.height() / 2.0;

        // Draw icon if configured
        if let Some(icon) = icon {
            self.draw_icon(ctx, icon, start_x, text_y - h_extents.height() - icon_size_addition / 2.0, icon_size, color.0, color.1, color.2);
            // Draw text after icon
            ctx.move_to(start_x + icon_size + icon_spacing, text_y);
        } else {
            // Draw text centered (no icon)
            ctx.move_to(start_x, text_y);
        }

        ctx.show_text(name).unwrap();
    }

    /// Draw countdown timer as dotted string in header area (right-aligned, vertically aligned as continuation of header text)
    fn draw_countdown(&self, ctx: &Context, seconds_left: u64, color: &(f64, f64, f64)) {
        let header_rect = self.layout.get_header_rect();

        // Create dot string: each second = one dot (e.g. 4 seconds = "....")
        let dots = ".".repeat(seconds_left as usize);

        ctx.set_source_rgba(color.0, color.1, color.2, 1.0);
        apply_text_style(ctx, &self.text_style.header_font, "Impact");

        let dots_extents = ctx.text_extents(&dots).unwrap();
        let text_extents = ctx.text_extents("T").unwrap();

        // Right-aligned: position at right edge minus text width and small margin
        let x = header_rect.width() - dots_extents.width() - 10.0;

        // Vertically aligned as continuation of header text (same y)
        let y = header_rect.height() / 2.0 + text_extents.height() / 2.0;

        ctx.move_to(x, y);
        ctx.show_text(&dots).unwrap();
    }

    /// Draw grid lines using layout calculations
    fn draw_grid_lines(&self, ctx: &Context, color: &(f64, f64, f64)) {
        let window_rect = self.layout.get_window_rect();
        let grid_rect = self.layout.get_grid_rect();

        ctx.set_source_rgba(color.0, color.1, color.2, 1.0);
        ctx.set_line_width(2.0);

        let tile_width = grid_rect.width() / 3.0;
        let tile_height = grid_rect.height() / 3.0;

        // 2 vertical lines
        for i in 1..3 {
            let x = i as f64 * tile_width;
            ctx.move_to(x, grid_rect.y());
            ctx.line_to(x, window_rect.bottom);
            ctx.stroke().unwrap();
        }

        // 3 horizontal lines
        for i in 0..3 {
            let y = grid_rect.y() + i as f64 * tile_height;
            ctx.move_to(0.0, y);
            ctx.line_to(window_rect.width(), y);
            ctx.stroke().unwrap();
        }

        // Grid border
        ctx.rectangle(0.0, 0.0, window_rect.width(), window_rect.height());
        ctx.stroke().unwrap();
    }

    /// Draw individual tile with content
    fn draw_tile(&self, ctx: &Context, pad: &Pad, tile_id: u8, rect: Rect, selected: bool) {
        // Resolve color scheme: pad-specific or board default
        let color_scheme = pad.color_scheme.as_ref().unwrap_or(self.color_scheme);
        let text_style = pad.text_style.as_ref().unwrap_or(self.text_style);

        let fg2_color = color_scheme.foreground2().to_rgb();
        let bg_color = color_scheme.background().to_rgb();

        // Highlight selected tile
        if selected {
            ctx.set_source_rgba(fg2_color.0, fg2_color.1, fg2_color.2, 0.3);
            ctx.rectangle(rect.x(), rect.y(), rect.width(), rect.height());
            ctx.fill().unwrap();
        } else if bg_color != self.color_scheme.background().to_rgb() {
            // Draw tile background if different from board default
            ctx.set_source_rgba(bg_color.0, bg_color.1, bg_color.2, color_scheme.opacity);
            ctx.rectangle(rect.x(), rect.y(), rect.width(), rect.height());
            ctx.fill().unwrap();
        }

        ctx.set_source_rgba(fg2_color.0, fg2_color.1, fg2_color.2, 1.0);

        {
            // Draw tile ID (bottom right corner)
            let id_layout = pangocairo::create_layout(ctx);
            id_layout.set_font_description(Some(&FontDescription::from_string(&text_style.pad_id_font)));
            let id_text = tile_id.to_string();
            id_layout.set_text(&id_text);

            let (id_width, id_height) = id_layout.size().scaled();

            // Baseline bottom
            let id_x = rect.x() + rect.width() - id_width - 10.0;
            let id_y = rect.y() + rect.height() - id_height - 10.0;

            ctx.move_to(id_x, id_y);
            pangocairo::show_layout(ctx, &id_layout);
        }

        // Draw header (top center)
        if !pad.header.is_empty() {
            let layout = pangocairo::create_layout(ctx);
            layout.set_font_description(Some(&FontDescription::from_string(&text_style.pad_header_font)));
            layout.set_text(&pad.header);
            layout.set_alignment(pango::Alignment::Center);

            let (header_width, _) = layout.size().scaled();

            // Center header horizontally, position near top
            let header_x = rect.x() + (rect.width() - header_width) / 2.0;
            let header_y = rect.y() + 10.0;

            ctx.move_to(header_x, header_y);
            pangocairo::show_layout(ctx, &layout);
        }

        // Draw text (center)
        if !pad.icon.is_empty() {
            // Draw icon if configured
            self.draw_icon(ctx, &pad.icon, rect.x() + rect.width() / 2.0 - 16.0, rect.y() + rect.height() / 2.0 - 16.0, 32.0, fg2_color.0, fg2_color.1, fg2_color.2);
        }
        else if !pad.text.is_empty() {
            let layout = pangocairo::create_layout(ctx);
            layout.set_font_description(Some(&FontDescription::from_string(&text_style.pad_text_font)));
            layout.set_text(&pad.text);
            layout.set_alignment(pango::Alignment::Center);

            let (text_width, text_height) = layout.size().scaled();

            // center text in header area
            let x = rect.x() + (rect.width() - text_width) / 2.0;
            let y = rect.y() + (rect.height() - text_height) / 2.0;

            ctx.move_to(x, y);
            pangocairo::show_layout(ctx, &layout);
        }
    }

    /// Draw icon in header area based on board configuration
    fn draw_icon(&self, ctx: &Context, icon: &str, x: f64, y: f64, size: f64, red: f64, green: f64, blue: f64) {
        if let Some(icon_path) = self.resources.icon(icon) {
            let icon_path = icon_path.to_str().unwrap();

            if icon_path.ends_with(".png") {
                // Load PNG icon
                if let Ok(mut file) = File::open(&icon_path) {
                    if let Ok(surface) = ImageSurface::create_from_png(&mut file) {
                        // Scale icon to match text height
                        let scale_x = size / surface.width() as f64;
                        let scale_y = size / surface.height() as f64;

                        ctx.save().unwrap();
                        ctx.translate(x, y);
                        ctx.scale(scale_x, scale_y);
                        ctx.set_source_surface(&surface, 0.0, 0.0).unwrap();
                        ctx.paint().unwrap();
                        ctx.restore().unwrap();
                    }
                }
            } else if icon_path.ends_with(".svg") {
                // Load SVG icon using rsvg
                if let Ok(mut handle) = rsvg::Loader::new().read_path(&icon_path) {

                    let color_str = format!("rgb({}, {}, {})", (red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8);
                    let stylesheet = format!(".board-s {{ stroke: {}; }}  .board-f {{ fill: {}; }}  .board-sf {{ stroke: {}; fill: {}; }} ", color_str, color_str , color_str, color_str);
                    handle.set_stylesheet(&stylesheet).expect("Failed to set stylesheet");

                    let renderer = rsvg::CairoRenderer::new(&handle);

                    ctx.save().unwrap();
                    ctx.translate(x, y);
                    renderer.render_document(ctx, &cairo::Rectangle::new(0.0, 0.0, size, size)).unwrap();
                    ctx.restore().unwrap();
                } else {
                    log::warn!("Failed to load SVG icon: {:?}", icon_path);
                }
            }

        }
    }

}

fn apply_text_style(ctx: &Context, font: &str, default_family: &str) {
    let font = FontDescription::from_string(font);

    let family = font.family();
    let family = family.as_deref().unwrap_or(default_family);

    let weight = font.weight();
    let font_weight = match weight {
        Weight::Normal => FontWeight::Normal,
        Weight::Bold => FontWeight::Bold,
        _ => FontWeight::Normal,
    };

    let style = font.style();
    let font_slant = match style {
        pango::Style::Normal => FontSlant::Normal,
        pango::Style::Italic => FontSlant::Italic,
        pango::Style::Oblique => FontSlant::Oblique,
        _ => FontSlant::Normal,
    };

    let font_size = font.size() as f64 / pango::SCALE as f64;

    ctx.select_font_face(family, font_slant, font_weight);
    ctx.set_font_size(font_size);
}

trait ScaledSize {
    fn scaled(&self) -> (f64, f64);
}

impl ScaledSize for (i32, i32) {
    fn scaled(&self) -> (f64, f64) {
        let (width, height) = *self;
        ((width as f64) / pango::SCALE as f64, (height as f64) / pango::SCALE as f64)
    }
}