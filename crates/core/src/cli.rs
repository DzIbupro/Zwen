#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchAction { Show, Recover, Quit }
pub fn parse_action(args: &[&str]) -> Result<LaunchAction, &'static str> {
    match args {
        [] | ["show" | "--show"] => Ok(LaunchAction::Show),
        ["recover" | "--recover"] => Ok(LaunchAction::Recover),
        ["quit" | "--quit"] => Ok(LaunchAction::Quit),
        _ => Err("用法：study-assistant [show|recover|quit]"),
    }
}
