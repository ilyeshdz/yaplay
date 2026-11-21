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

pub const MAX_NOTIFICATIONS: usize = 5;

pub const NOTIFICATION_TTL: Duration = Duration::from_secs(8);

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
    pub ttl: Duration,
}

impl Notification {
    pub fn is_expired(&self) -> bool {
        self.created.elapsed() >= self.ttl
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Hello {
        version: u8,
        controls: bool,
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

#[derive(Clone, PartialEq, Eq, Default)]
pub struct OverlaySnapshot {
    pub appearance: crate::data::appearance::Appearance,
    pub controls_available: bool,
    pub in_call: bool,
    pub voice_members: Vec<VoiceMember>,
    pub notifications: Vec<Notification>,
    pub mic_muted: bool,
    pub deafened: bool,
    pub controls_open: bool,
}

#[derive(Default)]
pub struct AppState {
    pub control_client: Option<tokio::sync::mpsc::Sender<ControlAction>>,
    pub startup_error: Option<String>,
    pub config: Configuration,
    pub in_call: bool,
    pub voice_members: Vec<VoiceMember>,
    pub notifications: Vec<Notification>,
    pub mic_muted: bool,
    pub deafened: bool,
    pub controls_open: bool,
    next_notification_id: u64,
}

impl AppState {
    pub fn request_control(&self, action: ControlAction) {
        if let Some(client) = self.control_client.as_ref().filter(|_| self.in_call) {
            let _ = client.try_send(action);
        }
    }

    pub fn push_notification(&mut self, title: impl Into<String>, body: impl Into<String>) {
        if !self.config.notifications_enabled {
            return;
        }
        self.prune_notifications();
        if self.notifications.len() >= MAX_NOTIFICATIONS {
            self.notifications.remove(0);
        }
        self.next_notification_id = self.next_notification_id.wrapping_add(1);
        self.notifications.push(Notification {
            id: self.next_notification_id,
            title: title.into(),
            body: body.into(),
            created: Instant::now(),
            ttl: NOTIFICATION_TTL,
        });
    }

    pub fn prune_notifications(&mut self) {
        self.notifications
            .retain(|notification| !notification.is_expired());
    }

    pub fn apply(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Hello { .. } => {}
            ServerMessage::VoiceState { in_call, members } => {
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
            controls_available: self
                .control_client
                .as_ref()
                .is_some_and(|client| !client.is_closed()),
            in_call: self.in_call,
            voice_members: self.voice_members.clone(),
            notifications: self
                .notifications
                .iter()
                .filter(|notification| !notification.is_expired())
                .cloned()
                .collect(),
            mic_muted: self.mic_muted,
            deafened: self.deafened,
            controls_open: self.controls_open,
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
    fn notifications_respect_preferences_and_keep_only_latest() {
        let mut state = AppState::default();
        state.config.notifications_enabled = false;
        state.push_notification("hidden", "");
        assert!(state.notifications.is_empty());
        state.config.notifications_enabled = true;
        for n in 0..10 {
            state.push_notification(n.to_string(), "");
        }
        assert_eq!(state.notifications.len(), MAX_NOTIFICATIONS);
        assert_eq!(state.notifications[0].title, "5");
        state.notifications[0].created = Instant::now() - NOTIFICATION_TTL;
        state.prune_notifications();
        assert_eq!(state.notifications.len(), MAX_NOTIFICATIONS - 1);
    }
}
