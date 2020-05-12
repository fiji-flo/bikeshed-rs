use super::metadata::{Date, MetadataManager};

pub trait Joinable<T> {
    fn join(&mut self, other: T);
}

impl Joinable<Option<String>> for Option<String> {
    fn join(&mut self, other: Option<String>) {
        if other.is_some() {
            *self = other;
        }
    }
}

impl Joinable<Option<Vec<String>>> for Option<Vec<String>> {
    fn join(&mut self, other: Option<Vec<String>>) {
        if self.is_some() && other.is_some() {
            self.as_mut()
                .unwrap()
                .extend(other.as_ref().unwrap().clone())
        } else if other.is_some() {
            *self = other;
        }
    }
}

impl Joinable<Option<Date>> for Option<Date> {
    fn join(&mut self, other: Option<Date>) {
        if other.is_some() {
            *self = other;
        }
    }
}

impl Joinable<MetadataManager> for MetadataManager {
    fn join(&mut self, other: MetadataManager) {
        if other.has_metadata {
            self.has_metadata = true;
            self.abs.join(other.abs);
            self.canonical_url.join(other.canonical_url);
            self.date.join(other.date);
            self.ed.join(other.ed);
            self.editors.join(other.editors);
            self.group.join(other.group);
            self.level.join(other.level);
            self.shortname.join(other.shortname);
            self.raw_status.join(other.raw_status);
            self.title.join(other.title);
        }
    }
}
