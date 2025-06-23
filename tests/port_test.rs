#[test]
fn env_port_overrides_default() {
    unsafe {
        std::env::set_var("PORT", "5555");
    }
    assert_eq!(vpscheck::web::get_port(), 5555);
    unsafe {
        std::env::remove_var("PORT");
    }
    assert_eq!(vpscheck::web::get_port(), 8080);
}
