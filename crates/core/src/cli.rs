#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAction { Show, Capsule, Recover, Quit }
pub fn parse_action(args: &[&str]) -> Result<LaunchAction, &'static str> {
    match args {
        [] | ["show" | "--show"] => Ok(LaunchAction::Show),
        ["capsule" | "--capsule"] => Ok(LaunchAction::Capsule),
        ["recover" | "--recover"] => Ok(LaunchAction::Recover),
        ["quit" | "--quit"] => Ok(LaunchAction::Quit),
        _ => Err("用法：zwen [show|capsule|recover|quit]"),
    }
}
