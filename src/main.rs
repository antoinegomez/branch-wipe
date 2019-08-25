//! # ListBox and ListModel Sample
//!
//! This sample demonstrates how to use gtk::ListBox in combination with
//! gio::ListStore as a model with a custom row type.
//!
//! It sets up a gtk::ListBox containing, per row, a label, spinbutton and
//! an edit button. The edit button allows to edit the underlying data structure
//! and changes are taking place immediately in the listbox by making use of GObject
//! property bindings.
//!
//! In addition it is possible to add new rows and delete old ones.

#[macro_use]
extern crate glib;
extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use std::env;

use row_data::RowData;

mod git;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

// upgrade weak reference or return
#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("branch wipe");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(480, 480);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

    // Create our list store and specify that the type stored in the
    // list should be the RowData GObject we define at the bottom
    let model = gio::ListStore::new(RowData::static_type());

    // And then create the UI part, the listbox and bind the list store
    // model to it. Whenever the UI needs to show a new row, e.g. because
    // it was notified that the model changed, it will call the callback
    // with the corresponding item from the model and will ask for a new
    // gtk::ListBoxRow that should be displayed.
    //
    // The gtk::ListBoxRow can contain any possible widgets.
    let listbox = gtk::ListBox::new();
    listbox.bind_model(Some(&model), clone!(model => move |item| {
        let box_ = gtk::ListBoxRow::new();
        box_.set_activatable(false);
        box_.set_selectable(false);
        let item = item.downcast_ref::<RowData>().expect("Row data is of wrong type");

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let label = gtk::Label::new(None);
        item.bind_property("name", &label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        hbox.pack_start(&label, true, true, 0);

        // Add our delete button to remove git branches
        let delete_button = gtk::Button::new_with_label("Delete");
        delete_button.connect_clicked(clone!(model, box_, item => move |_| {
            git::delete_branch(Some(&get_directory()), Some(&item.get_property("name").unwrap().get::<String>().unwrap()));
            model.remove(box_.get_index() as u32);
        }));

        hbox.add(&delete_button);
        box_.add(&hbox);
        box_.show_all();
        box_.upcast::<gtk::Widget>()
    }));

    let scrolled_window = gtk::ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
    scrolled_window.add(&listbox);

    vbox.pack_start(&scrolled_window, true, true, 0);

    window.add(&vbox);

    // Get branches in directory and create rows
    let branches = git::list_branches(Some(&get_directory()));
    for branch in branches {
        model.append(&RowData::new(&branch));
    }

    window.show_all();
}

fn get_directory() -> String {
    let path = env::current_dir().unwrap();
    path.to_string_lossy().to_string()
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.listbox-model"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

// Our GObject subclass for carrying a name for the ListBox model
//
// Name is stored in a RefCell to allow for interior mutability
// and are exposed via normal GObject properties. This allows us to use property
// bindings below to bind the values with what widgets display in the UI
mod row_data {
    use super::*;

    use glib::subclass;
    use glib::subclass::prelude::*;
    use glib::translate::*;

    // Implementation sub-module of the GObject
    mod imp {
        use super::*;
        use std::cell::RefCell;

        // The actual data structure that stores our values. This is not accessible
        // directly from the outside.
        pub struct RowData {
            name: RefCell<Option<String>>,
        }

        // GObject property definitions for our two values
        static PROPERTIES: [subclass::Property; 1] = [
            subclass::Property("name", |name| {
                glib::ParamSpec::string(
                    name,
                    "Name",
                    "Name",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
        ];

        // Basic declaration of our type for the GObject type system
        impl ObjectSubclass for RowData {
            const NAME: &'static str = "RowData";
            type ParentType = glib::Object;
            type Instance = subclass::simple::InstanceStruct<Self>;
            type Class = subclass::simple::ClassStruct<Self>;

            glib_object_subclass!();

            // Called exactly once before the first instantiation of an instance. This
            // sets up any type-specific things, in this specific case it installs the
            // properties so that GObject knows about their existence and they can be
            // used on instances of our type
            fn class_init(klass: &mut Self::Class) {
                klass.install_properties(&PROPERTIES);
            }

            // Called once at the very beginning of instantiation of each instance and
            // creates the data structure that contains all our state
            fn new() -> Self {
                Self {
                    name: RefCell::new(None),
                }
            }
        }

        // The ObjectImpl trait provides the setters/getters for GObject properties.
        // Here we need to provide the values that are internally stored back to the
        // caller, or store whatever new value the caller is providing.
        //
        // This maps between the GObject properties and our internal storage of the
        // corresponding values of the properties.
        impl ObjectImpl for RowData {
            glib_object_impl!();

            fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("name", ..) => {
                        let name = value.get();
                        self.name.replace(name);
                    }
                    _ => unimplemented!(),
                }
            }

            fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("name", ..) => Ok(self.name.borrow().to_value()),
                    _ => unimplemented!(),
                }
            }
        }
    }

    // Public part of the RowData type. This behaves like a normal gtk-rs-style GObject
    // binding
    glib_wrapper! {
        pub struct RowData(Object<subclass::simple::InstanceStruct<imp::RowData>, subclass::simple::ClassStruct<imp::RowData>, RowDataClass>);

        match fn {
            get_type => || imp::RowData::get_type().to_glib(),
        }
    }

    // Constructor for new instances. This simply calls glib::Object::new() with
    // initial values for our two properties and then returns the new instance
    impl RowData {
        pub fn new(name: &str) -> RowData {
            glib::Object::new(Self::static_type(), &[("name", &name)])
                .expect("Failed to create row data")
                .downcast()
                .expect("Created row data is of wrong type")
        }
    }
}
