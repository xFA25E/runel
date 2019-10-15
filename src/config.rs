use crate::{command::RunelCommand, mode::RunelMode};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

#[derive(Debug)]
pub enum Config {
    Daemon(Vec<String>),
    Remote(RunelCommand),
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::new_app().get_matches();
        match matches.subcommand_name() {
            Some("daemon") => Self::new_daemon(&matches),
            Some("remote") => Self::new_remote(&matches),
            _ => unreachable!(),
        }
    }

    fn new_remote(matches: &ArgMatches) -> Self {
        let sub_cmd = matches.subcommand_matches("remote").unwrap();
        Config::Remote(RunelCommand::from_matches(&sub_cmd))
    }

    fn new_daemon(matches: &ArgMatches) -> Self {
        let sub_cmd = matches.subcommand_matches("daemon").unwrap();
        Config::Daemon(
            sub_cmd
                .values_of("lemonbar args")
                .map_or_else(Vec::new, |vs| vs.map(String::from).collect()),
        )
    }

    fn new_app<'a, 'b>() -> App<'a, 'b> {
        App::new("runel")
            .setting(AppSettings::SubcommandRequired)
            .max_term_width(80)
            .version("1.0")
            .author("xFA25E")
            .about(include_str!("../help/runel.txt"))
            .before_help("Rust <3")
            .after_help(include_str!("../help/runel_after.txt"))
            .subcommand(
                SubCommand::with_name("daemon")
                    .about(include_str!("../help/runel_daemon.txt"))
                    .arg(
                        Arg::with_name("lemonbar args")
                            .value_name("LARGS")
                            .help(include_str!("../help/runel_daemon_lemonbar_args.txt"))
                            .takes_value(true)
                            .use_delimiter(true)
                            .empty_values(false)
                            .multiple(true)
                            .env("LEMONBAR_ARGS")
                            .last(true),
                    ),
            )
            .subcommand(
                SubCommand::with_name("remote")
                    .about(include_str!("../help/runel_remote.txt"))
                    .after_help(include_str!("../help/runel_remote_after.txt"))
                    .arg(
                        Arg::with_name("remote command")
                            .possible_values(&["reload", "quit", "set", "mode"])
                            .value_name("CMD")
                            .required(true)
                            .help(include_str!("../help/runel_remote_remote_command.txt"))
                            .takes_value(true)
                            .empty_values(false)
                            .requires_ifs(&[
                                ("set", "remote argument"),
                                ("mode", "remote argument"),
                                ("set", "remote value"),
                            ]),
                    )
                    .arg(
                        Arg::with_name("remote argument")
                            .validator(|s: String| -> Result<(), String> {
                                if s.contains(':') {
                                    Err("Argument can't contain a ':' char!".into())
                                } else {
                                    Ok(())
                                }
                            })
                            .value_name("RARG")
                            .help(include_str!("../help/runel_remote_remote_argument.txt"))
                            .takes_value(true)
                            .empty_values(false)
                            .required_ifs(&[("remote command", "set"), ("remote command", "mode")]),
                    )
                    .arg(
                        Arg::with_name("remote value")
                            .value_name("RVAL")
                            .help(include_str!("../help/runel_remote_remote_value.txt"))
                            .takes_value(true)
                            .empty_values(false)
                            .required_if("remote command", "set"),
                    ),
            )
    }
}

impl RunelCommand {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let get_value = move |s| matches.value_of(s).unwrap();

        match get_value("remote command") {
            "mode" => match get_value("remote argument") {
                "default" => RunelCommand::Mode {
                    mode: RunelMode::Default,
                    command: None,
                },
                mode => RunelCommand::Mode {
                    mode: RunelMode::Custom,
                    command: Some(mode.into()),
                },
            },

            "set" => RunelCommand::Set {
                widget: get_value("remote argument").parse().unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }),
                value: get_value("remote value").into(),
            },
            "reload" => RunelCommand::Reload,
            "quit" => RunelCommand::Quit,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::mode::RunelMode;
    use crate::widget::RunelWidget;

    impl Config {
        pub fn new_from_vec(v: Vec<&str>) -> Self {
            let matches = Self::new_app().get_matches_from(v);
            match matches.subcommand_name() {
                Some("daemon") => Self::new_daemon(&matches),
                Some("remote") => Self::new_remote(&matches),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn check_struct_set_long() {
        let my_struct = Config::new_from_vec(vec![
            "runel",
            "remote",
            "set",
            "title",
            "big,comma,separated,title banana",
        ]);

        if let Config::Remote(RunelCommand::Set { widget, value }) = my_struct {
            assert_eq!(widget, RunelWidget::Title);
            assert_eq!(value, "big,comma,separated,title banana");
        } else {
            panic!();
        }
    }

    #[test]
    fn check_struct_set() {
        let my_struct = Config::new_from_vec(vec!["runel", "remote", "set", "timer", "off"]);

        if let Config::Remote(RunelCommand::Set { widget, value }) = my_struct {
            assert_eq!(widget, RunelWidget::Timer);
            assert_eq!(value, "off");
        } else {
            panic!();
        }
    }

    #[test]
    fn check_struct_mode() {
        let my_struct = Config::new_from_vec(vec!["runel", "remote", "mode", "default"]);

        if let Config::Remote(RunelCommand::Mode {
            mode: RunelMode::Default,
            command: None,
        }) = my_struct
        {
        } else {
            panic!();
        }
    }

    #[test]
    fn check_struct_mode_custom() {
        let my_struct = Config::new_from_vec(vec!["runel", "remote", "mode", "wifi"]);

        if let Config::Remote(RunelCommand::Mode {
            mode: RunelMode::Custom,
            command: Some(mode),
        }) = my_struct
        {
            assert_eq!("wifi", mode);
        } else {
            panic!();
        }
    }

    #[test]
    fn check_struct_quit() {
        let my_struct = Config::new_from_vec(vec!["runel", "remote", "quit"]);

        if let Config::Remote(RunelCommand::Quit) = my_struct {
        } else {
            panic!();
        }
    }

    #[test]
    fn check_struct_reload() {
        let my_struct = Config::new_from_vec(vec!["runel", "remote", "reload"]);

        if let Config::Remote(RunelCommand::Reload) = my_struct {
        } else {
            panic!();
        }
    }

    #[test]
    fn set_to_string() {
        let my_struct = Config::new_from_vec(vec!["runel", "remote", "set", "title", "world"]);
        if let Config::Remote(command) = my_struct {
            assert_eq!("set:title:world", command.to_string());
        } else {
            panic!();
        }
    }

    #[test]
    fn check_env_struct() {
        std::env::set_var("LEMONBAR_ARGS", "-a,32,-f,lucy tewi-8,-u,1");
        let my_struct = Config::new_from_vec(vec!["runel", "daemon"]);
        if let Config::Daemon(lemonbar_args) = my_struct {
            assert_eq!(
                lemonbar_args,
                vec!["-a", "32", "-f", "lucy tewi-8", "-u", "1"]
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn check_config_struct() {
        std::env::set_var("LEMONBAR_ARGS", "-a,32,-f,lucy tewi-8,-u,1");
        let my_struct =
            Config::new_from_vec(vec!["runel", "daemon", "--", "--cool_flag", "cool_value"]);
        if let Config::Daemon(lemonbar_args) = my_struct {
            assert_eq!(
                lemonbar_args,
                vec![
                    "--cool_flag",
                    "cool_value",
                    "-a",
                    "32",
                    "-f",
                    "lucy tewi-8",
                    "-u",
                    "1"
                ]
            );
        } else {
            panic!();
        }
    }
}
