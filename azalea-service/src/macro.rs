#[macro_export]
macro_rules! services {
    (
        $(use $name: ident: $service: ty;)*
        $(use_option $name_option: ident: $service_option:ty;)*
    ) => {
        use std::cell::RefCell;

        #[derive(Clone)]
        pub struct Services {
            $($name: std::rc::Rc<RefCell<$crate::Handler<$service>>>),*
            $($name_option: Option<std::rc::Rc<RefCell<$crate::Handler<$service_option>>>>),*
        }

        $(impl $crate::HasService<$service> for Services {
            fn get_service(
                &self,
            ) -> Option<std::rc::Rc<RefCell<$crate::Handler<$service>>>> {
                Some(self.$name.clone())
            }
        })*

        $(impl $crate::HasService<$service_option> for Services {
            fn get_service(
                &self,
            ) -> Option<std::rc::Rc<RefCell<$crate::Handler<$service_option>>>> {
                self.$name_option.clone()
            }
        })*

        impl<ParentServices> $crate::FromServices<ParentServices> for Services
        where
            $(ParentServices: $crate::HasService<$service>),*
            $(ParentServices: $crate::HasService<$service_option>),*
        {
            fn inherit(value: &ParentServices) -> Self {
                Self {
                    $($name: <ParentServices as $crate::HasService<$service>>::get_service(value).unwrap()),*
                    $($name_option: <ParentServices as $crate::HasService<$service_option>>::get_service(value)),*
                }
            }
        }
    };
}
