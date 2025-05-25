use std::rc::Rc;

use crate::{Handler, Service};

pub trait HasService<ServiceKind>
where
    ServiceKind: Service,
{
    fn get_service(&self) -> Option<Rc<Handler<ServiceKind>>> {
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
