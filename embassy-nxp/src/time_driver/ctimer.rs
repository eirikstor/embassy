use crate::pac;
use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Waker;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time_driver::Driver;

// ------------------------------------------------------------------------
// GLOBAL STATE
// ------------------------------------------------------------------------

static OVERFLOWS: AtomicU32 = AtomicU32::new(0);

struct AlarmState {
    waker: Option<Waker>,
}

unsafe impl Send for AlarmState {}

static ALARM_STATE: Mutex<CriticalSectionRawMutex, RefCell<AlarmState>> =
    Mutex::new(RefCell::new(AlarmState { waker: None }));

// ------------------------------------------------------------------------
// DRIVER IMPLEMENTATION
// ------------------------------------------------------------------------

struct CtimerDriver;

embassy_time_driver::time_driver_impl!(static DRIVER: CtimerDriver = CtimerDriver);

impl Driver for CtimerDriver {
    fn now(&self) -> u64 {
        // No dereference needed for unit struct handles
        let ctimer = pac::CTIMER0;

        loop {
            let high = OVERFLOWS.load(Ordering::Relaxed);

            // FIX: Use .tc() method. Read returns value directly.
            let low = ctimer.tc().read().tcval();

            let high_after = OVERFLOWS.load(Ordering::Relaxed);

            if high == high_after {
                return ((high as u64) << 32) | (low as u64);
            }
        }
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        let ctimer = pac::CTIMER0;

        ALARM_STATE.lock(|cell| {
            let mut state = cell.borrow_mut();
            state.waker = Some(waker.clone());
        });

        let current = self.now();
        if at <= current {
            waker.wake_by_ref();
            return;
        }

        let target_low = (at & 0xFFFFFFFF) as u32;

        // FIX: Use .mr(0) for array access. Use .set_match_(val) for writing.
        ctimer.mr(0).write(|w| w.set_match_(target_low));

        // FIX: Use .set_mr0i(true)
        ctimer.mcr().modify(|w| w.set_mr0i(true));
    }
}

// ------------------------------------------------------------------------
// INITIALIZATION
// ------------------------------------------------------------------------

pub fn init() {
    let syscon = pac::SYSCON;
    let ctimer = pac::CTIMER0;

    unsafe {
        // 1. Enable Clock
        // FIX: .sysahbclkctrl0() method and .set_ctimer0(true)
        syscon.sysahbclkctrl0().modify(|w| w.set_ctimer(true));

        // 2. Reset
        // FIX: .presetctrl0() method, set bit to true (assert), then false (clear)
        syscon
            .presetctrl0()
            .modify(|w| w.set_ctimer0_rst_n(nxp_pac::syscon::vals::Ctimer0RstN::ASSERT));
        syscon
            .presetctrl0()
            .modify(|w| w.set_ctimer0_rst_n(nxp_pac::syscon::vals::Ctimer0RstN::CLEAR));

        // 3. Configure Prescaler for 1 MHz
        // FIX: .pr() method and .set_prval(val)
        ctimer.pr().write(|w| w.set_prval(29));

        // 4. Set MR3 to Max for Overflow Handling
        // FIX: .mr(3) method
        ctimer.mr(3).write(|w| w.set_match_(0xFFFF_FFFF));

        // 5. Enable Overflow Interrupt (MR3I)
        // FIX: .mcr() method and .set_mr3i(true)
        ctimer.mcr().write(|w| w.set_mr3i(true));

        // 6. Start Timer
        // FIX: .tcr() method and .set_cen(true)
        ctimer.tcr().write(|w| w.set_cen(true));

        // 7. Enable NVIC Interrupt
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::CTIMER0);
    }
}

// ------------------------------------------------------------------------
// INTERRUPT HANDLER
// ------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn CTIMER0() {
    let ctimer = pac::CTIMER0;

    // FIX: .ir() method
    let ir = ctimer.ir().read();

    // 1. Handle Overflow (MR3)
    // FIX: ir.mr3int() returns bool directly, no .bit() needed
    if ir.mr3int() {
        let current = OVERFLOWS.load(Ordering::Relaxed);
        OVERFLOWS.store(current.wrapping_add(1), Ordering::Relaxed);

        // Clear flag
        ctimer.ir().write(|w| w.set_mr3int(true));
    }

    // 2. Handle Alarm (MR0)
    if ir.mr0int() {
        // Clear flag
        ctimer.ir().write(|w| w.set_mr0int(true));

        // Disable Alarm Interrupt
        ctimer.mcr().modify(|w| w.set_mr0i(false));

        ALARM_STATE.lock(|cell| {
            let state = cell.borrow();
            if let Some(waker) = &state.waker {
                waker.wake_by_ref();
            }
        });
    }
}
