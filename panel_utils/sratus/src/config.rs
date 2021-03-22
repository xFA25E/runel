pub const FIELDS: &'static [(
    &'static str,
    &'static str,
    Option<&'static [&'static str]>,
    &'static str,
)] = &[
    // field, head, command, default value
    ("keyseq", "S", None, ""),
    ("timer", "T", None, "off"),
    ("keyboard", "K", Some(&["lkeyboard"]), ""),
    ("light", "L", Some(&["lbrightness"]), ""),
    ("volume", "V", Some(&["lvolume"]), ""),
    ("battery", "B", Some(&["lbattery"]), ""),
    ("date", "D", Some(&["lclock", "60", "%d %b, %a"]), ""),
    ("clock", "C", Some(&["lclock", "3", "%R"]), ""),
];

pub const MQUEUE: &'static str = "/sratus";
pub const MAX_MSG_LEN: usize = 100;
pub const CAPACITY: usize = 10;
