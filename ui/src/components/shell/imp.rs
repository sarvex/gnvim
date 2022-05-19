use std::cell::{Cell, RefCell};

use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::components::grid::Grid;
use crate::font::Font;
use crate::nvim::Neovim;

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/com/github/vhakulinen/gnvim/shell.ui")]
pub struct Shell {
    #[template_child(id = "msg-fixed")]
    pub msg_fixed: TemplateChild<gtk::Fixed>,
    #[template_child(id = "root-grid")]
    pub root_grid: TemplateChild<Grid>,

    pub nvim: RefCell<Neovim>,

    pub grids: RefCell<Vec<Grid>>,
    /// Current grid.
    ///
    /// On startup this will be an invalid grid, but the first cursor goto
    /// event will fix that.
    pub current_grid: RefCell<Grid>,
    pub font: RefCell<Font>,
    pub busy: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for Shell {
    const NAME: &'static str = "Shell";
    type Type = super::Shell;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        Grid::ensure_type();

        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Shell {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // Add the root grid to the grids list.
        self.grids.borrow_mut().push(self.root_grid.clone());
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::new(
                    "font",
                    "font",
                    "Font",
                    Font::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecObject::new(
                    "nvim",
                    "nvim",
                    "Neovim client",
                    Neovim::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecBoolean::new(
                    "busy",
                    "busy",
                    "Busy",
                    false,
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "font" => self.font.borrow().to_value(),
            "busy" => self.busy.get().to_value(),
            "nvim" => self.nvim.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "font" => {
                self.font
                    .replace(value.get().expect("font value must be an Font object"));
            }
            "busy" => self
                .busy
                .set(value.get().expect("busy value needs to be a bool")),
            "nvim" => {
                self.nvim.replace(
                    value
                        .get()
                        .expect("nvim value needs to be an Neovim object"),
                );
            }
            _ => unimplemented!(),
        };
    }
}

impl WidgetImpl for Shell {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: gtk::Orientation,
        for_size: i32,
    ) -> (i32, i32, i32, i32) {
        // Currently, the shell's size is the same as the root grid's size.
        // Note that for the min width we need to report something smaller so
        // that the top level window remains resizable (since its using the
        // shell as the root widget).
        let (mw, nw, mb, nb) = self.root_grid.measure(orientation, for_size);
        (mw.min(1), nw, mb, nb)
    }

    fn size_allocate(&self, widget: &Self::Type, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(widget, width, height, baseline);

        let mut child: Option<gtk::Widget> = widget.first_child();
        while let Some(sib) = child {
            if sib.should_layout() {
                sib.allocate(width, height, -1, None);
            }

            child = sib.next_sibling();
        }
    }
}
