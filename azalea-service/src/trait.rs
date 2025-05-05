use std::rc::Rc;

use crate::Service;

pub trait HasService<Model>
where
    Model: relm4::Worker,
{
    fn get_service() -> Rc<Service<Model>>;
}
