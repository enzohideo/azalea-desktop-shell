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

pub trait FromServices<Services>: Sized {
    fn inherit(value: &Services) -> Self;
}

pub trait IntoServices<Services>: Sized {
    fn strip(&self) -> Services;
}

impl<ChildServices, ParentServices> IntoServices<ChildServices> for ParentServices
where
    ChildServices: FromServices<ParentServices>,
{
    fn strip(&self) -> ChildServices {
        ChildServices::inherit(self)
    }
}
