// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

void spin_acquire(unsigned int* locked) {
    __sync_synchronize();
    while(__sync_lock_test_and_set(locked, 1) != 0);
    __sync_synchronize();
}

void spin_release(unsigned int* locked) {
    __sync_synchronize();
    __sync_lock_release(locked);
    __sync_synchronize();
}
