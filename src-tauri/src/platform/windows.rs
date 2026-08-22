//! Windows backend.
//!
//! Three things the OS is asked for: keep the machine awake while a work
//! phase runs, say when it suspends and comes back, and let a global hotkey
//! reach the app. All three are plain Win32 calls, declared here by hand
//! rather than pulled in as a crate — this is two functions and one
//! callback, and a binding crate would be a dependency for eleven lines of
//! `extern "system"`.
//!
//! Sleep inhibition is per **thread** on Windows: `SetThreadExecutionState`
//! applies to whichever thread called it and lapses when that thread ends.
//! Tauri commands run on a pool, so the call cannot be made from whatever
//! thread happened to answer — it goes to one long-lived thread of our own,
//! which holds the state for as long as the session runs.

use std::ffi::c_void;
use std::sync::mpsc::{channel, Sender};

use super::{PlatformError, PlatformServices, SleepEvent, SleepWatcher};

/// Keep the current state until it is changed again.
const ES_CONTINUOUS: u32 = 0x8000_0000;
/// The machine may not go to sleep.
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
/// The screen may not blank.
const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;

/// Ask for the callback form of the notification rather than a window
/// message: the app has no window procedure of its own to hook into.
const DEVICE_NOTIFY_CALLBACK: u32 = 2;

/// The machine is about to suspend.
const PBT_APMSUSPEND: u32 = 0x0004;
/// It came back — with the user present, or without.
const PBT_APMRESUMESUSPEND: u32 = 0x0007;
const PBT_APMRESUMEAUTOMATIC: u32 = 0x0012;

/// What `PowerRegisterSuspendResumeNotification` returns when it worked.
const ERROR_SUCCESS: u32 = 0;

#[link(name = "kernel32")]
extern "system" {
    /// Tells Windows what the app needs kept alive. Per thread, not per
    /// process — see the module docs.
    fn SetThreadExecutionState(flags: u32) -> u32;
}

/// The callback Windows invokes on a power event.
type PowerCallback = unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> u32;

/// What `PowerRegisterSuspendResumeNotification` takes in callback mode.
#[repr(C)]
struct SubscribeParameters {
    callback: PowerCallback,
    context: *mut c_void,
}

#[link(name = "powrprof")]
extern "system" {
    fn PowerRegisterSuspendResumeNotification(
        flags: u32,
        recipient: *mut c_void,
        registration: *mut *mut c_void,
    ) -> u32;
}

/// What the awake thread is being asked for.
enum Awake {
    /// Hold the machine and the screen awake.
    Hold,
    /// Let them be again.
    Release,
}

/// Sleep inhibition and suspend notifications through Win32.
#[derive(Debug, Default)]
pub struct WindowsPlatform {
    /// The thread holding the execution state, started on first use.
    awake: Option<Sender<Awake>>,
}

impl WindowsPlatform {
    /// Sends one instruction to the awake thread, starting it if needed.
    fn tell(&mut self, what: Awake) -> Result<(), PlatformError> {
        if self.awake.is_none() {
            self.awake = Some(spawn_awake_thread()?);
        }

        self.awake
            .as_ref()
            .expect("thread was just started")
            .send(what)
            // Поток мог умереть — тогда состояние всё равно снято системой,
            // и следующий запрос поднимет новый.
            .map_err(|err| {
                self.awake = None;
                PlatformError::Backend(err.to_string())
            })
    }
}

/// Starts the thread that owns the execution state.
///
/// It outlives every request: the state belongs to the thread, so letting it
/// finish would quietly let the machine sleep mid-session.
fn spawn_awake_thread() -> Result<Sender<Awake>, PlatformError> {
    let (sender, receiver) = channel::<Awake>();

    std::thread::Builder::new()
        .name("lokked-awake".to_string())
        .spawn(move || {
            for request in receiver {
                let flags = match request {
                    Awake::Hold => ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED,
                    Awake::Release => ES_CONTINUOUS,
                };

                // SAFETY: одна функция без указателей, вызываемая всегда из
                // этого потока — ровно то, чего требует Windows.
                unsafe {
                    SetThreadExecutionState(flags);
                }
            }

            // Канал закрыт — приложение уходит; состояние снимается за собой.
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        })
        .map_err(|err| PlatformError::Backend(err.to_string()))?;

    Ok(sender)
}

/// What Windows calls on suspend and on resume.
///
/// # Safety
///
/// `context` is the pointer handed to `PowerRegisterSuspendResumeNotification`
/// and is a `SleepWatcher` leaked on purpose — see [`WindowsPlatform::watch_sleep`].
unsafe extern "system" fn on_power_event(
    context: *mut c_void,
    event: u32,
    _setting: *mut c_void,
) -> u32 {
    if context.is_null() {
        return ERROR_SUCCESS;
    }

    let watcher = &*(context as *const SleepWatcher);
    match event {
        PBT_APMSUSPEND => watcher(SleepEvent::GoingToSleep),
        PBT_APMRESUMESUSPEND | PBT_APMRESUMEAUTOMATIC => watcher(SleepEvent::WokeUp),
        // Смена батареи, спящий монитор и прочее — не наше дело.
        _ => {}
    }

    ERROR_SUCCESS
}

impl PlatformServices for WindowsPlatform {
    fn inhibit_sleep(&mut self) -> Result<(), PlatformError> {
        self.tell(Awake::Hold)
    }

    fn release_sleep(&mut self) -> Result<(), PlatformError> {
        self.tell(Awake::Release)
    }

    fn notify(&self, _title: &str, _body: &str) -> Result<(), PlatformError> {
        // Уведомления идут через tauri-plugin-notification, как и на Linux:
        // он один и тот же на всех платформах.
        Ok(())
    }

    fn watch_sleep(&mut self, on_event: SleepWatcher) -> Result<(), PlatformError> {
        // Колбэк живёт столько же, сколько приложение, и Windows держит на
        // него указатель — поэтому он намеренно утекает. Отписки нет и не
        // нужно: регистрация одна на весь запуск.
        let context = Box::into_raw(Box::new(on_event)) as *mut c_void;
        let mut parameters = SubscribeParameters {
            callback: on_power_event,
            context,
        };
        let mut registration: *mut c_void = std::ptr::null_mut();

        // SAFETY: `parameters` живёт до конца вызова, а `context` — до конца
        // работы приложения, как того и требует API.
        let status = unsafe {
            PowerRegisterSuspendResumeNotification(
                DEVICE_NOTIFY_CALLBACK,
                &mut parameters as *mut SubscribeParameters as *mut c_void,
                &mut registration,
            )
        };

        if status != ERROR_SUCCESS {
            // Подписка не удалась — забираем утечку обратно, раз некому
            // звонить.
            unsafe {
                drop(Box::from_raw(context as *mut SleepWatcher));
            }

            return Err(PlatformError::Backend(format!(
                "PowerRegisterSuspendResumeNotification: код {status}"
            )));
        }

        Ok(())
    }
}
