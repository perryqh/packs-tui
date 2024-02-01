use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,
    Next,
    Down,
    Up,
    Left,
    Right,
    UncontainedOutViolations,
    UncontainedInViolations,
    ContainedOutViolations,
    ContainedInViolations,
    NextTab,
    Escape,
    SortAscending,
    SortDescending,
}
