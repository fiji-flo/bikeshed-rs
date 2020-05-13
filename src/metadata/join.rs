use super::metadata::Metadata;
use crate::util::date::Date;

pub trait Joinable {
    fn join(&mut self, other: &Self);
}

impl<T> Joinable for Option<T>
where
    T: Clone,
{
    fn join(&mut self, other: &Self) {
        if let Some(val) = other {
            *self = Some(val.clone());
        }
    }
}

impl<T> Joinable for Vec<T>
where
    T: Clone,
{
    fn join(&mut self, other: &Self) {
        self.extend(other.iter().cloned());
    }
}

impl Joinable for Date {
    fn join(&mut self, other: &Self) {
        self.date = other.date;
    }
}

impl Joinable for Metadata {
    fn join(&mut self, other: &Self) {
        if other.has_keys {
            self.has_keys = true;
            self.abs.join(&other.abs);
            self.canonical_url.join(&other.canonical_url);
            self.date.join(&other.date);
            self.ed.join(&other.ed);
            self.editors.join(&other.editors);
            self.group.join(&other.group);
            self.level.join(&other.level);
            self.shortname.join(&other.shortname);
            self.raw_status.join(&other.raw_status);
            self.title.join(&other.title);
        }
    }
}
