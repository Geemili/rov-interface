
error_chain! {
    errors {
        #[doc = "An error message from the SDL2 crate."]
        SdlMsg(msg: ::std::string::String) {
            description("sdl error")
            display("{}", msg)
        }
    }
}
