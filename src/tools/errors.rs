use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum ResourceErrorIO {
        InsufficientPermissions {
            display("InsufficientPermissions: cannot save resource data")
        }
    }
}
