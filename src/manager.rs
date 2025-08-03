use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use gdk::{
    RGBA,
    gio::{self, BusType, DBusCallFlags, DBusProxy, DBusProxyFlags, ListStore},
    glib::{self, Object, clone},
    prelude::*,
    subclass::prelude::*,
};
use tracing::{debug, info, warn};
use tsparql::{Notifier, NotifierEvent, NotifierEventType, SparqlConnection, prelude::*};

use crate::{
    Calendar, Collection, CollectionsModel, Event, Provider, Resource, Timeframe, Zoned,
    pre_resource::PreResource, spawn, utils::*,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::Manager)]
    pub struct Manager {
        connection: OnceCell<zbus::blocking::Connection>,
        resource_pool: OnceCell<Mutex<HashMap<String, Resource>>>,
        #[property(get)]
        collections_model: OnceCell<CollectionsModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Manager {
        const NAME: &'static str = "Manager";
        type Type = super::Manager;
        type ParentType = Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Manager {
        fn constructed(&self) {
            self.parent_constructed();

            let Ok(_) = self
                .connection
                .set(zbus::blocking::Connection::session().unwrap())
            else {
                panic!("Failed to set session connection");
            };

            self.collections_model
                .get_or_init(CollectionsModel::default);

            spawn!(clone!(
                #[weak(rename_to = imp)]
                self,
                async move {
                    imp.retrieve_resources();
                }
            ));
        }
    }

    impl Manager {
        pub(super) fn resource_pool(&self) -> MutexGuard<'_, HashMap<String, Resource>> {
            self.resource_pool
                .get()
                .expect("resource pool should be initialized")
                .lock()
                .unwrap()
        }

        fn retrieve_resources(&self) {
            let connection = self
                .connection
                .get()
                .expect("Connection should be initialized");

            let proxy = zbus::blocking::fdo::ObjectManagerProxy::builder(connection)
                .destination("org.gnome.evolution.dataserver.Sources5")
                .unwrap()
                .path("/org/gnome/evolution/dataserver/SourceManager")
                .unwrap()
                .build()
                .unwrap();

            let mut sources = HashMap::new();

            // Get all managed objects
            let objects = proxy.get_managed_objects().unwrap();
            for (object_path, _) in objects {
                let proxy = zbus::blocking::Proxy::new(
                    connection,
                    "org.gnome.evolution.dataserver.Sources5",
                    object_path.clone(),
                    "org.gnome.evolution.dataserver.Source",
                )
                .unwrap();
                let data = proxy.get_property::<String>("Data").unwrap();
                let uid = proxy.get_property::<String>("UID").unwrap();

                if let Some(source_info) = parse_source_data(object_path.clone(), uid, data) {
                    sources.insert(object_path, (source_info, proxy));
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct Manager(ObjectSubclass<imp::Manager>);
}

impl Manager {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn find_resource(&self, uri: &str) -> Option<Resource> {
        self.imp().resource_pool().get(uri).cloned()
    }

    pub(crate) fn create_calendar(&self, _collection_uri: &str, _name: &str, _color: RGBA) {
        dbg!("TODO");
    }

    pub(crate) fn update_calendar(&self, _uri: &str, _name: Option<&str>, _color: Option<RGBA>) {
        dbg!("TODO");
    }

    pub(crate) fn delete_calendar(&self, _uri: &str) {
        dbg!("TODO");
    }

    pub(crate) fn create_event(&self, _calendar_uri: &str, _name: &str, _description: &str) {
        dbg!("TODO");
    }

    pub fn search_events(&self, _query: &str) -> ListStore {
        ListStore::new::<Event>()
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
