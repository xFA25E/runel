use crate::colors::Colors;

use std::{
    cell::RefCell,
    fmt::{self, Display},
    rc::Rc,
};

type RColors = Rc<RefCell<Colors>>;

pub enum WidgetStatus {
    Updated,
    NotUpdated,
}

impl WidgetStatus {
    pub fn is_updated(&self) -> bool {
        match self {
            WidgetStatus::Updated => true,
            WidgetStatus::NotUpdated => false,
        }
    }
}

pub enum Widget {
    Bspwm {
        v: String,
        c: RColors,
    },
    Title {
        v: String,
        c: RColors,
    },
    Info {
        v: String,
        c: RColors,
        h: &'static str,
    },
    Text {
        v: String,
    },
}

impl Widget {
    pub fn new_bspwm(c: RColors) -> Self {
        Widget::Bspwm {
            c,
            v: String::new(),
        }
    }

    pub fn new_title(c: RColors) -> Self {
        Widget::Title {
            c,
            v: String::new(),
        }
    }

    pub fn new_info(c: RColors, h: &'static str) -> Self {
        Widget::Info {
            c,
            h,
            v: String::new(),
        }
    }

    pub fn new_text(v: &str) -> Self {
        Widget::Text { v: String::from(v) }
    }

    pub fn set(&mut self, v: String) -> WidgetStatus {
        let val = self.v();
        if val != &v {
            *val = v;
            val.retain(|c| c != '\n');
            WidgetStatus::Updated
        } else {
            WidgetStatus::NotUpdated
        }
    }

    fn v(&mut self) -> &mut String {
        use Widget::*;

        match self {
            Bspwm { v, .. } | Title { v, .. } | Info { v, .. } | Text { v, .. } => v,
        }
    }
}

impl Display for Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Widget::*;

        match self {
            Bspwm { v, c } => {
                fn split(s: &str) -> Option<(char, &str)> {
                    if s.len() > 1 {
                        Some((s.as_bytes()[0] as char, &s[1..]))
                    } else {
                        None
                    }
                }
                let c = c.borrow();

                if let Some(s) = v.get(1..) {
                    for (start, text) in s.split(':').filter_map(split) {
                        match start {
                            'm' => write!(f, "{} {}  ", c.monitor, text)?,
                            'M' => write!(f, "{}-{}- ", c.monitor, text)?,
                            'f' => write!(f, "{} {}  ", c.free, text)?,
                            'F' => write!(f, "{}-{}- ", c.free, text)?,
                            'o' => write!(f, "{} {}  ", c.occupied, text)?,
                            'O' => write!(f, "{}-{}- ", c.occupied, text)?,
                            'u' => write!(f, "{} {}  ", c.urgent, text)?,
                            'U' => write!(f, "{}-{}- ", c.urgent, text)?,
                            'L' | 'T' | 'G' => write!(f, " {}{}", c.state, text)?,
                            _ => continue,
                        }
                    }
                }
                Ok(())
            }
            Title { v, c } => write!(f, "{}{}", c.borrow().title, v),
            Info { v, c, h } => {
                let c = c.borrow();
                write!(f, "{}{}  {}{}", c.info_head, h, c.info, v)
            }
            Text { v } => write!(f, "{}", v),
        }
    }
}

#[cfg(test)]
mod widget_tests {
    use super::*;

    static BSPWM_S: &str = "WMeDP1:OH:oB:fV:FC:uR:UM:LT:TT:GSPL";
    static BSPWM_R: &str = "-eDP1- -H-  B   V  -C-  R  -M-  T T SPL";

    #[test]
    fn bspwm_widget() {
        let cl = Rc::new(RefCell::new(Colors::new()));
        let mut wid = Widget::new_bspwm(cl);
        wid.set(BSPWM_S.into());
        assert_eq!(wid.v(), BSPWM_S);
        assert_eq!(BSPWM_R, wid.to_string());
    }

    #[test]
    fn info_widget() {
        let cl = Rc::new(RefCell::new(Colors::new()));
        let mut wid = Widget::new_info(cl, "Head");
        let expect = "bonono good";
        wid.set(expect.into());
        assert_eq!(wid.v(), expect);

        assert_eq!("Head  bonono good", wid.to_string());
    }

    #[test]
    fn title_widget() {
        let cl = Rc::new(RefCell::new(Colors::new()));
        let mut wid = Widget::new_title(cl);
        let expect = "tutle";
        wid.set(expect.into());
        assert_eq!(wid.v(), expect);

        assert_eq!("tutle", wid.to_string());
    }

    #[test]
    fn text_widget() {
        let _cl = Rc::new(RefCell::new(Colors::new()));
        let expect = "really simple";
        let mut wid = Widget::new_text(expect);
        assert_eq!(wid.v(), expect);
        let expect = "hello";
        wid.set(expect.into());

        assert_eq!(expect, wid.to_string());
    }
}
