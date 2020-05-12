use super::metadata::{Date, Metadata};

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

impl Joinable<Metadata> for Metadata {
    fn join(&mut self, other: Metadata) {
        if other.has_keys {
            self.has_keys = true;
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
