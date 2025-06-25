use serde::{Deserialize, Serialize};

use crate::enums::*;

/// Theme represents the UI theme preference.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Theme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "system")]
    System,
}

/// UpdateUserPrivateParams represents the parameters for updating a user's private profile.
#[derive(Serialize, Default, Clone, Debug)]
pub struct UpdateUserPrivateParams {
    /// Profile description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    
    /// Main platform you are playing on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,
    
    /// Is crossplay enabled for your WF account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossplay: Option<bool>,
    
    /// UI locale and preferable communication language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<Language>,
    
    /// UI theme preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    
    /// Should we sync locale across devices.
    #[serde(rename = "syncLocale", skip_serializing_if = "Option::is_none")]
    pub sync_locale: Option<bool>,
    
    /// Should we sync theme across devices.
    #[serde(rename = "syncTheme", skip_serializing_if = "Option::is_none")]
    pub sync_theme: Option<bool>,
}

impl UpdateUserPrivateParams {
    pub fn new() -> Self {
        UpdateUserPrivateParams::default()
    }

    pub fn with_about<S: Into<String>>(mut self, about: S) -> Self {
        self.about = Some(about.into());
        self
    }

    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn with_crossplay(mut self, crossplay: bool) -> Self {
        self.crossplay = Some(crossplay);
        self
    }

    pub fn with_locale(mut self, locale: Language) -> Self {
        self.locale = Some(locale);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn with_sync_locale(mut self, sync_locale: bool) -> Self {
        self.sync_locale = Some(sync_locale);
        self
    }

    pub fn with_sync_theme(mut self, sync_theme: bool) -> Self {
        self.sync_theme = Some(sync_theme);
        self
    }
}
