use super::{Pad, ColorScheme, TextStyle, ModifierState};

pub trait PadSet {
    fn pads(&self) -> &Vec<Pad>;
    fn get_or_default(&self, index: usize) -> Pad {
        self.pads().get(index).cloned().unwrap_or_default()
    }
    fn clone_box(&self) -> Box<dyn PadSet>;
}

impl Clone for Box<dyn PadSet> {
    fn clone(&self) -> Box<dyn PadSet> {
        self.clone_box()
    }
}

pub trait Board {
    fn title(&self) -> &str;
    fn icon(&self) -> Option<&str>;
    fn color_scheme(&self) -> &ColorScheme;
    fn text_style(&self) -> &TextStyle;
    fn pads(&self, modifier: Option<ModifierState>) -> Box<dyn PadSet>;
    fn clone_box(&self) -> Box<dyn Board>;
}

impl Clone for Box<dyn Board> {
    fn clone(&self) -> Box<dyn Board> {
        self.clone_box()
    }
}

impl PadSet for Vec<Pad> {
    fn pads(&self) -> &Vec<Pad> {
        self
    }

    fn clone_box(&self) -> Box<dyn PadSet> {
        Box::new(self.clone())
    }
}