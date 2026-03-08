pub mod timer;
pub mod keyboard;

pub fn init_all() {
    timer::init();
    keyboard::init();
}
