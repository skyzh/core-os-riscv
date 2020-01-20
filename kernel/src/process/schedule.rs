use crate::wait_forever;
use crate::cpu;
use crate::trap::usertrapret;

pub fn scheduler() -> ! {
    usertrapret();
    crate::wait_forever()
}
