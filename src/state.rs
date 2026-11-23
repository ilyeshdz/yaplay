use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::data::configuration::Configuration;

pub type SharedState = Arc<RwLock<AppState>>;

static APP_STATE: OnceLock<SharedState> = OnceLock::new();

pub fn app_state() -> SharedState {
    APP_STATE
        .get_or_init(|| Arc::new(RwLock::new(AppState::default())))
        .clone()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceMember {
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub id: String,
    pub name: String,
    pub color: (u8, u8, u8),
    pub speaking: bool,
    pub muted: bool,
    pub is_self: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub created: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Hello {
        version: u8,
        controls: Option<bool>,
        name: Option<String>,
        client_version: Option<String>,
        actions: Option<Vec<String>>,
    },
    VoiceState {
        in_call: bool,
        members: Vec<VoiceMember>,
    },
    Notification {
        title: String,
        body: String,
    },
    MicState {
        muted: bool,
    },
    DeafenState {
        deafened: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlAction {
    SetMute { muted: bool },
    SetDeafen { deafened: bool },
    LeaveCall {},
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    SetMute,
    SetDeafen,
    LeaveCall,
}

impl Capability {
    pub const ALL: [Self; 3] = [Self::SetMute, Self::SetDeafen, Self::LeaveCall];

    pub fn from_wire(name: &str) -> Option<Self> {
        match name {
            "set_mute" => Some(Self::SetMute),
            "set_deafen" => Some(Self::SetDeafen),
            "leave_call" => Some(Self::LeaveCall),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SetMute => "Mute",
            Self::SetDeafen => "Deafen",
            Self::LeaveCall => "Leave call",
        }
    }
}

impl ControlAction {
    pub fn capability(&self) -> Capability {
        match self {
            Self::SetMute { .. } => Capability::SetMute,
            Self::SetDeafen { .. } => Capability::SetDeafen,
            Self::LeaveCall {} => Capability::LeaveCall,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Integration {
    pub name: String,
    pub version: Option<String>,
    pub actions: BTreeSet<Capability>,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct OverlaySnapshot {
    pub appearance: crate::data::appearance::Appearance,
    pub notification_style: crate::data::configuration::NotificationSettings,
    pub integration: Option<Integration>,
    pub legacy_source: bool,
    pub in_call: bool,
    pub voice_members: Vec<VoiceMember>,
    pub notifications: Vec<Notification>,
    pub mic_muted: bool,
    pub deafened: bool,
    pub controls_open: bool,
    /// Transient mode that makes the display-only panels draggable.
    pub arranging: bool,
}

pub struct ActiveIntegration {
    pub info: Integration,
    pub sender: tokio::sync::mpsc::Sender<ControlAction>,
}

#[derive(Default)]
pub struct AppState {
    pub active: Option<ActiveIntegration>,
    pub legacy_source: Option<tokio::sync::mpsc::Sender<ControlAction>>,
    pub startup_error: Option<String>,
    pub config: Configuration,
    pub in_call: bool,
    pub voice_members: Vec<VoiceMember>,
    pub notifications: Vec<Notification>,
    pub mic_muted: bool,
    pub deafened: bool,
    pub controls_open: bool,
    /// Transient mode that makes the display-only panels draggable.
    pub arranging: bool,
    stopped_speaking: HashMap<String, Instant>,
    next_notification_id: u64,
}

impl AppState {
    pub fn integration(&self) -> Option<&Integration> {
        self.active.as_ref().map(|client| &client.info)
    }

    pub fn permits(&self, action: &ControlAction) -> bool {
        self.in_call
            && self
                .integration()
                .is_some_and(|i| i.actions.contains(&action.capability()))
    }

    pub fn request_control(&self, action: ControlAction) {
        if let Some(client) = self.active.as_ref().filter(|_| self.permits(&action)) {
            let _ = client.sender.try_send(action);
        }
    }

    pub fn push_notification(&mut self, title: impl Into<String>, body: impl Into<String>) {
        if !self.config.notifications_enabled {
            return;
        }
        self.next_notification_id = self.next_notification_id.wrapping_add(1);
        self.notifications.push(Notification {
            id: self.next_notification_id,
            title: title.into(),
            body: body.into(),
            created: Instant::now(),
        });
        self.prune_notifications();
    }

    pub fn prune_notifications(&mut self) {
        if !self.config.notifications_enabled {
            self.notifications.clear();
            return;
        }
        let ttl = Duration::from_secs(self.config.notifications.seconds.into());
        self.notifications
            .retain(|note| note.created.elapsed() < ttl);
        let excess = self
            .notifications
            .len()
            .saturating_sub(self.config.notifications.limit.into());
        self.notifications.drain(..excess);
    }

    pub fn apply(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Hello { .. } => {}
            ServerMessage::VoiceState { in_call, members } => {
                let previous: HashSet<_> = self
                    .voice_members
                    .iter()
                    .filter(|m| m.speaking)
                    .map(|m| m.id.as_str())
                    .collect();
                let mut held = std::mem::take(&mut self.stopped_speaking);
                let now = Instant::now();
                if in_call {
                    for member in members.iter().filter(|m| !m.speaking) {
                        let since = if previous.contains(member.id.as_str()) {
                            Some(now)
                        } else {
                            held.remove(&member.id)
                        };
                        if let Some(since) =
                            since.filter(|t| now.duration_since(*t) < Duration::from_secs(10))
                        {
                            self.stopped_speaking.insert(member.id.clone(), since);
                        }
                    }
                }
                self.in_call = in_call;
                self.voice_members = if in_call { members } else { Vec::new() };
                if !in_call {
                    self.mic_muted = false;
                    self.deafened = false;
                }
            }
            ServerMessage::Notification { title, body } => self.push_notification(title, body),
            ServerMessage::MicState { muted } => self.mic_muted = muted,
            ServerMessage::DeafenState { deafened } => self.deafened = deafened,
        }
    }

    pub fn snapshot(&self) -> OverlaySnapshot {
        OverlaySnapshot {
            appearance: self.config.appearance.clone(),
            integration: self.integration().cloned(),
            legacy_source: self.legacy_source.is_some(),
            in_call: self.in_call,
            voice_members: self
                .voice_members
                .iter()
                .filter(|m| {
                    !self.config.appearance.speakers_only
                        || (self.config.appearance.show_self || !m.is_self)
                            && (m.speaking
                                || self.stopped_speaking.get(&m.id).is_some_and(|since| {
                                    since.elapsed()
                                        < Duration::from_secs(
                                            self.config.appearance.speaker_hold_seconds.into(),
                                        )
                                }))
                })
                .cloned()
                .collect(),
            notification_style: self.config.notifications.clone(),
            notifications: self.notifications.clone(),
            mic_muted: self.mic_muted,
            deafened: self.deafened,
            controls_open: self.controls_open,
            arranging: self.arranging,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving_call_clears_roster_and_local_state() {
        let mut state = AppState {
            mic_muted: true,
            deafened: true,
            ..AppState::default()
        };
        let member = VoiceMember {
            avatar_url: None,
            id: "self".into(),
            name: "You".into(),
            color: (103, 80, 164),
            speaking: false,
            muted: true,
            is_self: true,
        };
        state.apply(ServerMessage::VoiceState {
            in_call: false,
            members: vec![member],
        });
        assert!(!state.in_call);
        assert!(state.voice_members.is_empty());
        assert!(!state.mic_muted && !state.deafened);
    }

    #[test]
    fn speaker_hold_starts_on_silence_and_ends_on_expiry_or_departure() {
        let mut state = AppState::default();
        state.config.appearance.speakers_only = true;
        let mut member = VoiceMember {
            id: "a".into(),
            name: "A".into(),
            avatar_url: None,
            color: (0, 0, 0),
            is_self: false,
            speaking: false,
            muted: false,
        };
        let roster = |m: &VoiceMember| ServerMessage::VoiceState {
            in_call: true,
            members: vec![m.clone()],
        };
        state.apply(roster(&member));
        assert!(state.snapshot().voice_members.is_empty());
        member.speaking = true;
        state.apply(roster(&member));
        assert!(state.snapshot().voice_members[0].speaking);
        member.speaking = false;
        state.apply(roster(&member));
        let stopped = state.stopped_speaking["a"];
        state.apply(roster(&member));
        assert_eq!(state.stopped_speaking["a"], stopped);
        assert!(!state.snapshot().voice_members[0].speaking);
        *state.stopped_speaking.get_mut("a").unwrap() -= Duration::from_secs(3);
        assert!(state.snapshot().voice_members.is_empty());
        state.config.appearance.speaker_hold_seconds = 10;
        assert_eq!(state.snapshot().voice_members.len(), 1);
        state.apply(ServerMessage::VoiceState {
            in_call: true,
            members: vec![],
        });
        state.apply(roster(&member));
        assert!(state.snapshot().voice_members.is_empty());
    }

    #[test]
    fn notification_edits_apply_to_existing_cards() {
        let mut state = AppState::default();
        for title in ["oldest", "older", "newest"] {
            state.push_notification(title, "body");
        }
        state.config.notifications.limit = 2;
        state.prune_notifications();
        assert_eq!(state.notifications[0].title, "older");
        state.notifications[0].created -= Duration::from_secs(2);
        state.config.notifications.seconds = 1;
        state.prune_notifications();
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(state.notifications[0].title, "newest");
        state.config.notifications_enabled = false;
        state.prune_notifications();
        assert!(state.snapshot().notifications.is_empty());
    }

    #[test]
    fn notifications_respect_preferences_and_keep_only_latest() {
        let mut state = AppState::default();
        state.config.notifications_enabled = false;
        state.push_notification("hidden", "");
        assert!(state.notifications.is_empty());
        state.config.notifications_enabled = true;
        for n in 0..10 {
            state.push_notification(n.to_string(), "");
        }
        assert_eq!(state.notifications.len(), 5);
        assert_eq!(state.notifications[0].title, "5");
        state.notifications[0].created = Instant::now() - Duration::from_secs(8);
        state.prune_notifications();
        assert_eq!(state.notifications.len(), 5 - 1);
    }
}
