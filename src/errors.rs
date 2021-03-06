// Errors
// Powered by error_chain.

extern crate mem;

error_chain! {
    types {
        Error, ErrorKind, ResultExt;
    }

    links {
        Mem(mem::errors::Error, mem::errors::ErrorKind);
    }

    errors {
        CPUNotRunning {
            description("cpu's state is not running")
        }
    }
}
