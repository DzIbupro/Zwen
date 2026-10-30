use crate::selection::Mode;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAction { Show, Capsule, Recover, Quit, Selection(Mode) }
pub fn parse_action(args: &[&str]) -> Result<LaunchAction, &'static str> {
    match args {
        [] | ["show" | "--show"] => Ok(LaunchAction::Show),
        ["capsule" | "--capsule"] => Ok(LaunchAction::Capsule),
        ["select"] => Ok(LaunchAction::Selection(Mode::Raw)),
        ["explain-selection"] => Ok(LaunchAction::Selection(Mode::Explain)),
        ["translate-selection"] => Ok(LaunchAction::Selection(Mode::Translate)),
        ["recover" | "--recover"] => Ok(LaunchAction::Recover),
        ["quit" | "--quit"] => Ok(LaunchAction::Quit),
        _ => Err("用法：zwen [show|capsule|select|explain-selection|translate-selection|recover|quit]"),
    }
}
