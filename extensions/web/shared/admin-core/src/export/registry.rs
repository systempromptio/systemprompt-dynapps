//! Every exportable table, by id.
//!
//! Each dataset registers itself with `inventory::submit!` from the crate
//! that owns its data. `order` is the export dialog's listing order, so
//! [`all`] sorts on it and two registrations never share a number.

use super::model::DataSet;

#[derive(Clone, Copy)]
pub struct DataSetRegistration {
    pub order: u16,
    pub dataset: &'static dyn DataSet,
}

impl std::fmt::Debug for DataSetRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataSetRegistration")
            .field("order", &self.order)
            .field("dataset", &self.dataset.id())
            .finish()
    }
}

inventory::collect!(DataSetRegistration);

#[must_use]
pub fn all() -> Vec<&'static DataSetRegistration> {
    let mut registrations: Vec<&'static DataSetRegistration> =
        inventory::iter::<DataSetRegistration>.into_iter().collect();
    registrations.sort_by_key(|r| r.order);
    registrations
}

#[must_use]
pub fn find(id: &str) -> Option<&'static dyn DataSet> {
    inventory::iter::<DataSetRegistration>
        .into_iter()
        .map(|r| r.dataset)
        .find(|d| d.id() == id)
}
