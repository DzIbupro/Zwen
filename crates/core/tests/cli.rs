use study_assistant_core::cli::{parse_action, LaunchAction};
#[test] fn empty_command_only_shows_panel(){assert_eq!(parse_action(&[]).unwrap(),LaunchAction::Show);}
#[test] fn explicit_actions_are_distinct(){
 for (arg,action) in [("show",LaunchAction::Show),("recover",LaunchAction::Recover),("quit",LaunchAction::Quit)] {
  assert_eq!(parse_action(&[arg]).unwrap(),action);assert_eq!(parse_action(&[&format!("--{arg}")]).unwrap(),action);
 }
}
#[test] fn unknown_or_ambiguous_actions_are_rejected(){assert!(parse_action(&["capture"]).is_err());assert!(parse_action(&["show","quit"]).is_err());}
