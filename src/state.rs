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

pub const NOTIFICATION_TTL: Duration = Duration::from_secs(8);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceMember {
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
    VoiceState {
        in_call: bool,
        members: Vec<VoiceMember>,
    },
    Notification { title: String, body: String },
    MicState { muted: bool },
    DeafenState { deafened: bool },
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct OverlaySnapshot {
    pub in_call: bool,
    pub voice_members: Vec<VoiceMember>,
    pub notifications: Vec<Notification>,
    pub mic_muted: bool,
    pub deafened: bool,
    pub controls_open: bool,
}

#[derive(Default)]
pub struct AppState {
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
    pub fn push_notification(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.next_notification_id += 1;
        self.notifications.push(Notification {
            id: self.next_notification_id,
            title: title.into(),
            body: body.into(),
            created: Instant::now(),
            ttl: NOTIFICATION_TTL,
        });
    }

    pub fn prune_notifications(&mut self) {
        self.notifications.retain(|notification| !notification.is_expired());
    }

    pub fn apply(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::VoiceState { in_call, members } => {
                self.in_call = in_call;
                self.voice_members = members;
            }
            ServerMessage::Notification { title, body } => self.push_notification(title, body),
            ServerMessage::MicState { muted } => self.mic_muted = muted,
            ServerMessage::DeafenState { deafened } => self.deafened = deafened,
        }
    }

    pub fn snapshot(&self) -> OverlaySnapshot {
        OverlaySnapshot {
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

    pub fn seed_fake_voice_state(&mut self) {
        self.in_call = true;
        self.voice_members = vec![
            VoiceMember {
                id: "self".to_string(),
                name: "You".to_string(),
                color: (103, 80, 164),
                speaking: false,
                muted: false,
                is_self: true,
            },
            VoiceMember {
                id: "u-mara".to_string(),
                name: "Mara".to_string(),
                color: (76, 175, 80),
                speaking: true,
                muted: false,
                is_self: false,
            },
            VoiceMember {
                id: "u-theo".to_string(),
                name: "Theo".to_string(),
                color: (255, 193, 7),
                speaking: false,
                muted: true,
                is_self: false,
            },
        ];
    }
}
