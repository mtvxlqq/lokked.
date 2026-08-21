//! What the command line asks the running app to do.
//!
//! On Wayland an application cannot grab a global hotkey by itself, so the
//! shortcut is registered in GNOME Settings and runs `lokked --zen`. The
//! second launch hands its argv to the instance that is already running (see
//! the single-instance plugin in `lib.rs`), and this module is where that
//! argv turns into an intention.
//!
//! Pure parsing, no clap: three flags do not need an argument parser, and
//! `core` takes no dependencies it can do without.

/// What a second launch asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommand {
    /// Пауза, если сессия идёт; продолжить, если она на паузе.
    Toggle,
    /// Открыть чёрный экран.
    Zen,
    /// Остановить сессию и записать её.
    Stop,
}

impl CliCommand {
    /// The flag that asks for this command.
    pub fn flag(self) -> &'static str {
        match self {
            Self::Toggle => "--toggle",
            Self::Zen => "--zen",
            Self::Stop => "--stop",
        }
    }
}

/// The first recognised flag in `args`, or `None` for a plain launch.
///
/// The first element is the binary path and is skipped. Unknown arguments are
/// ignored rather than rejected: a desktop file, a session manager or a
/// student's own script can add flags of their own, and none of that is worth
/// refusing to start over.
pub fn parse_args(args: &[String]) -> Option<CliCommand> {
    args.iter().skip(1).find_map(|arg| match arg.as_str() {
        "--toggle" => Some(CliCommand::Toggle),
        "--zen" => Some(CliCommand::Zen),
        "--stop" => Some(CliCommand::Stop),
        _ => None,
    })
}
