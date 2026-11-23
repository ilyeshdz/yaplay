use serde::{Deserialize, Serialize};

/// A floating panel that can be moved around the overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Panel {
    Controls,
    Roster,
    Notifications,
}

impl Panel {
    pub const ALL: [Self; 3] = [Self::Controls, Self::Roster, Self::Notifications];

    pub fn name(self) -> &'static str {
        match self {
            Self::Controls => "Controls",
            Self::Roster => "Voice panel",
            Self::Notifications => "Notifications",
        }
    }
}

/// Saved position of a panel, relative to the top-left corner of one display.
///
/// Storing the display keeps the layout stable when several monitors are
/// connected; positions are clamped back on screen when a display disappears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelPosition {
    pub display_id: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    pub controls: Option<PanelPosition>,
    pub roster: Option<PanelPosition>,
    pub notifications: Option<PanelPosition>,
}

impl Layout {
    pub fn get(&self, panel: Panel) -> Option<PanelPosition> {
        match panel {
            Panel::Controls => self.controls,
            Panel::Roster => self.roster,
            Panel::Notifications => self.notifications,
        }
    }

    pub fn set(&mut self, panel: Panel, position: PanelPosition) {
        match panel {
            Panel::Controls => self.controls = Some(position),
            Panel::Roster => self.roster = Some(position),
            Panel::Notifications => self.notifications = Some(position),
        }
    }

    pub fn clear(&mut self, panels: &[Panel]) {
        for panel in panels {
            match panel {
                Panel::Controls => self.controls = None,
                Panel::Roster => self.roster = None,
                Panel::Notifications => self.notifications = None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_round_trips_and_missing_fields_use_defaults() {
        let position = PanelPosition {
            display_id: 7,
            x: -20,
            y: 140,
        };
        let mut layout = Layout::default();
        layout.set(Panel::Roster, position);
        let restored: Layout =
            serde_json::from_slice(&serde_json::to_vec(&layout).unwrap()).unwrap();
        assert_eq!(restored, layout);
        assert_eq!(restored.get(Panel::Roster), Some(position));
        assert_eq!(restored.get(Panel::Controls), None);
        assert_eq!(
            serde_json::from_str::<Layout>("{}").unwrap(),
            Layout::default()
        );
    }

    #[test]
    fn clearing_panels_leaves_the_others_alone() {
        let mut layout = Layout::default();
        for panel in Panel::ALL {
            layout.set(
                panel,
                PanelPosition {
                    display_id: 1,
                    x: 0,
                    y: 0,
                },
            );
        }
        layout.clear(&[Panel::Roster, Panel::Notifications]);
        assert!(layout.get(Panel::Controls).is_some());
        assert!(layout.get(Panel::Roster).is_none());
        assert!(layout.get(Panel::Notifications).is_none());
    }
}
