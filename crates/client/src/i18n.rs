//! Localization. Every string goes through `t`, which falls back to English and
//! then to the key itself, so a missing translation never renders blank.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    English,
    German,
    Spanish,
    French,
}

impl Locale {
    pub fn tag(self) -> &'static str {
        match self {
            Locale::English => "en",
            Locale::German => "de",
            Locale::Spanish => "es",
            Locale::French => "fr",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Locale::English => "English",
            Locale::German => "Deutsch",
            Locale::Spanish => "Espanol",
            Locale::French => "Francais",
        }
    }

    pub fn parse(tag: &str) -> Locale {
        match tag.trim().to_ascii_lowercase().as_str() {
            "de" | "de-de" => Locale::German,
            "es" | "es-es" => Locale::Spanish,
            "fr" | "fr-fr" => Locale::French,
            _ => Locale::English,
        }
    }

    fn index(self) -> u8 {
        match self {
            Locale::English => 0,
            Locale::German => 1,
            Locale::Spanish => 2,
            Locale::French => 3,
        }
    }

    fn from_index(index: u8) -> Locale {
        match index {
            1 => Locale::German,
            2 => Locale::Spanish,
            3 => Locale::French,
            _ => Locale::English,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set_locale(locale: Locale) {
    CURRENT.store(locale.index(), Ordering::Relaxed);
}

pub fn locale() -> Locale {
    Locale::from_index(CURRENT.load(Ordering::Relaxed))
}

fn locale_path() -> Option<std::path::PathBuf> {
    let base = std::env::var("APPDATA").ok()?;
    Some(
        std::path::PathBuf::from(base)
            .join("Runecord")
            .join("locale.txt"),
    )
}

pub fn load_locale() -> Locale {
    locale_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|tag| Locale::parse(tag.trim()))
        .unwrap_or(Locale::English)
}

pub fn save_locale(locale: Locale) {
    if let Some(path) = locale_path() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, locale.tag());
    }
}

pub fn locales() -> [Locale; 4] {
    [
        Locale::English,
        Locale::German,
        Locale::Spanish,
        Locale::French,
    ]
}

/// `(key, German, Spanish, French)`; English is the key itself.
const STRINGS: &[(&str, &str, &str, &str)] = &[
    ("My Account", "Mein Konto", "Mi cuenta", "Mon compte"),
    ("Profile", "Profil", "Perfil", "Profil"),
    ("Application", "Anwendung", "Aplicacion", "Application"),
    ("Appearance", "Darstellung", "Apariencia", "Apparence"),
    (
        "Notifications",
        "Benachrichtigungen",
        "Notificaciones",
        "Notifications",
    ),
    (
        "Voice & Video",
        "Sprache & Video",
        "Voz y video",
        "Voix et video",
    ),
    ("Keybinds", "Tastenbelegung", "Atajos", "Raccourcis"),
    ("Bots & Apps", "Bots & Apps", "Bots y apps", "Bots et apps"),
    ("Log Out", "Abmelden", "Cerrar sesion", "Se deconnecter"),
    ("Events", "Termine", "Eventos", "Evenements"),
    ("Members", "Mitglieder", "Miembros", "Membres"),
    (
        "Channels & Roles",
        "Kanale & Rollen",
        "Canales y roles",
        "Salons et roles",
    ),
    ("Overview", "Ubersicht", "Resumen", "Vue d'ensemble"),
    ("Onboarding", "Einstieg", "Incorporacion", "Integration"),
    ("AutoMod", "AutoMod", "AutoMod", "AutoMod"),
    ("Templates", "Vorlagen", "Plantillas", "Modeles"),
    (
        "Integrations",
        "Integrationen",
        "Integraciones",
        "Integrations",
    ),
    ("Roles", "Rollen", "Roles", "Roles"),
    ("Emojis", "Emojis", "Emojis", "Emojis"),
    ("Bans", "Bans", "Expulsiones", "Bannissements"),
    (
        "Audit Log",
        "Audit-Protokoll",
        "Registro de auditoria",
        "Journal d'audit",
    ),
    (
        "Server Settings",
        "Servereinstellungen",
        "Ajustes del servidor",
        "Reglages du serveur",
    ),
    (
        "User Settings",
        "Benutzereinstellungen",
        "Ajustes de usuario",
        "Reglages utilisateur",
    ),
    (
        "Save Changes",
        "Anderungen speichern",
        "Guardar cambios",
        "Enregistrer",
    ),
    ("Cancel", "Abbrechen", "Cancelar", "Annuler"),
    ("Search", "Suchen", "Buscar", "Rechercher"),
    ("Message", "Nachricht", "Mensaje", "Message"),
    ("Online", "Online", "En linea", "En ligne"),
    ("Offline", "Offline", "Desconectado", "Hors ligne"),
    ("Friends", "Freunde", "Amigos", "Amis"),
    (
        "Direct Messages",
        "Direktnachrichten",
        "Mensajes directos",
        "Messages prives",
    ),
];

fn translate(locale: Locale, key: &str) -> Option<&'static str> {
    let entry = STRINGS.iter().find(|(name, ..)| *name == key)?;
    let text = match locale {
        Locale::English => return None,
        Locale::German => entry.1,
        Locale::Spanish => entry.2,
        Locale::French => entry.3,
    };
    (!text.is_empty()).then_some(text)
}

/// The localized string for `key`, falling back to English and then the key.
pub fn t(key: &str) -> String {
    translate(locale(), key)
        .map(str::to_string)
        .unwrap_or_else(|| key.to_string())
}

/// Number of strings that have a translation for the active locale.
pub fn coverage() -> (usize, usize) {
    let locale = locale();
    let done = STRINGS
        .iter()
        .filter(|(key, ..)| translate(locale, key).is_some())
        .count();
    (done, STRINGS.len())
}

#[cfg(test)]
mod tests {
    use super::{Locale, coverage, locale, set_locale, t};
    use std::sync::Mutex;

    /// The locale is process-global, so these two tests must not overlap.
    static LOCALE_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn translations_fall_back_to_english() {
        let _locale = LOCALE_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        set_locale(Locale::German);
        assert_eq!(locale(), Locale::German);
        assert_eq!(t("Members"), "Mitglieder");
        assert_eq!(t("Not a real key"), "Not a real key");
        set_locale(Locale::English);
        assert_eq!(t("Members"), "Members");
        let (done, total) = coverage();
        assert_eq!(done, 0);
        assert!(total > 20);
    }

    #[test]
    fn every_locale_covers_the_catalogue() {
        let _locale = LOCALE_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for locale in super::locales() {
            set_locale(locale);
            let (done, total) = coverage();
            if locale == Locale::English {
                assert_eq!(done, 0);
            } else {
                assert_eq!(done, total, "{} is missing strings", locale.tag());
            }
        }
        set_locale(Locale::English);
    }
}
