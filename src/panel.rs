use crate::{
    colors::Colors,
    error::XDisplayError,
    mode::RunelMode,
    widget::RunelWidget::{self, *},
    widgets::{Widget, WidgetStatus},
};

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Display},
    rc::Rc,
};

type RWidget = Rc<RefCell<Widget>>;
type Widgets = HashMap<RunelWidget, RWidget>;
type Modes = HashMap<RunelMode, Vec<RWidget>>;
type RColors = Rc<RefCell<Colors>>;

pub struct Panel {
    colors: RColors,
    widgets: Widgets,
    modes: Modes,
    current_mode: RunelMode,
}

impl Panel {
    pub fn new(colors: Colors) -> Self {
        let colors = Rc::new(RefCell::new(colors));
        let widgets = rwidgets(&colors);

        Self {
            current_mode: RunelMode::Default,
            modes: rmodes(&widgets),
            widgets,
            colors,
        }
    }

    pub fn set(&mut self, widget: RunelWidget, value: String) -> WidgetStatus {
        self.widgets.get_mut(&widget).map_or_else(
            || WidgetStatus::NotUpdated,
            |wid| wid.borrow_mut().set(value),
        )
    }

    pub fn mode(&mut self, mode: RunelMode) -> WidgetStatus {
        if self.current_mode != mode {
            self.current_mode = mode;
            WidgetStatus::Updated
        } else {
            WidgetStatus::NotUpdated
        }
    }

    pub fn reload(&mut self) -> Result<WidgetStatus, XDisplayError> {
        self.colors
            .borrow_mut()
            .query_xresources()
            .map(|_| WidgetStatus::Updated)
    }
}

macro_rules! rcl {
    ($t:expr) => {
        Rc::clone(&$t)
    };
}

macro_rules! ins {
    ($ws:expr, $key:expr, $val:expr) => {
        $ws.insert($key, Rc::new(RefCell::new($val)))
    };
}

fn rwidgets(colors: &RColors) -> Widgets {
    let mut ws = HashMap::with_capacity(13);

    ins!(ws, Bspwm, Widget::new_bspwm(rcl!(colors)));

    ins!(ws, Title, Widget::new_title(rcl!(colors)));

    ins!(ws, Timer, Widget::new_info(rcl!(colors), "T"));
    ins!(ws, Keyboard, Widget::new_info(rcl!(colors), "K"));
    ins!(ws, Brightness, Widget::new_info(rcl!(colors), "L"));
    ins!(ws, Volume, Widget::new_info(rcl!(colors), "V"));
    ins!(ws, Battery, Widget::new_info(rcl!(colors), "B"));
    ins!(ws, Date, Widget::new_info(rcl!(colors), "D"));
    ins!(ws, Clock, Widget::new_info(rcl!(colors), "C"));

    for (w, t) in &[(Space, " "), (Left, "%{l}"), (Right, "%{r}"), (Custom, "")] {
        ins!(ws, *w, Widget::new_text(t));
    }
    ws
}

fn rmodes(ws: &Widgets) -> Modes {
    let mut modes = HashMap::with_capacity(2);

    modes.insert(RunelMode::Default, {
        make_mode_vec(
            ws,
            &[
                Left, Space, Bspwm, Space, Title, Right, Space, Timer, Space, Space, Keyboard,
                Space, Space, Brightness, Space, Space, Volume, Space, Space, Battery, Space,
                Space, Date, Space, Space, Clock, Space,
            ],
        )
    });

    modes.insert(RunelMode::Custom, {
        make_mode_vec(
            ws,
            &[
                Left, Space, Bspwm, Space, Title, Right, Space, Custom, Space,
            ],
        )
    });

    return modes;

    fn make_mode_vec(ws: &Widgets, ms: &[RunelWidget]) -> Vec<RWidget> {
        let mut vec = Vec::with_capacity(ms.len());
        ms.iter().for_each(|w| vec.push(rcl!(&ws[w])));
        vec
    }
}

impl Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.colors.borrow().bg)?;
        for wid in &self.modes[&self.current_mode] {
            write!(f, "{}", wid.borrow())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod panel_tests {
    use super::*;
    use crate::command::RunelCommand::*;

    #[test]
    fn panel_input() {
        let mut new_panel = Panel::new(Colors::new());
        let test_cmds = vec![
            Set {
                widget: RunelWidget::Bspwm,
                value: String::from("MeDP1:OH:oB:fV:oC:fR:fM:fT:oS:LT:TT:G"),
            },
            Set {
                widget: RunelWidget::Title,
                value: String::from("bananas"),
            },
            Set {
                widget: RunelWidget::Timer,
                value: String::from("off"),
            },
            Set {
                widget: RunelWidget::Keyboard,
                value: String::from("dvorak"),
            },
            Set {
                widget: RunelWidget::Keyboard,
                value: String::from("dvorak"),
            },
            Set {
                widget: RunelWidget::Brightness,
                value: String::from("89"),
            },
            Set {
                widget: RunelWidget::Volume,
                value: String::from("100 on"),
            },
            Set {
                widget: RunelWidget::Battery,
                value: String::from("Charging 77"),
            },
            Set {
                widget: RunelWidget::Date,
                value: String::from("19 August"),
            },
            Set {
                widget: RunelWidget::Clock,
                value: String::from("04:20"),
            },
            Set {
                widget: RunelWidget::Custom,
                value: String::from("yourmama vegana bitch fuck"),
            },
            Set {
                widget: RunelWidget::Clock,
                value: String::from("05:20"),
            },
            Mode {
                mode: RunelMode::Custom,
                command: Some(String::from("wifi")),
            },
            Mode {
                mode: RunelMode::Default,
                command: None,
            },
            Reload,
            Set {
                widget: RunelWidget::Title,
                value: String::from("reload bitech"),
            },
            Quit,
        ];

        for s in test_cmds {
            match s {
                Set { widget, value } => new_panel.set(widget, value),
                Mode { mode, .. } => new_panel.mode(mode),
                Reload => new_panel.reload().unwrap(),
                Quit => break,
            };
        }

        println!("{}", new_panel);
    }
}
