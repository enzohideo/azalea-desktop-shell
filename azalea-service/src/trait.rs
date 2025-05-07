use std::rc::Rc;

use crate::Service;

pub trait HasService<Model>
where
    Model: relm4::Worker,
{
    fn get_service(&self) -> Option<Rc<Service<Model>>> {
        None
    }
}
