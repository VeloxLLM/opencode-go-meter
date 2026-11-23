//! Multi-language support for the tray meter.
//!
//! The UI language can be selected from the tray menu or the usage window.
//! It is persisted locally and defaults to English. `OPENCODE_GO_LANG`
//! can still be used to force a language.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
    Ja,
    Ko,
    Es,
    Fr,
    De,
    Ru,
}

pub struct Strings {
    pub window_title: &'static str,
    pub subtitle: &'static str,
    pub used: &'static str,
    pub remaining: &'static str,
    pub status_ok: &'static str,
    pub status_limited: &'static str,
    pub reset_after: &'static str,
    pub last_updated: &'static str,
    pub reopen_hint: &'static str,
    pub open_console: &'static str,
    pub close: &'static str,
    pub menu_refresh: &'static str,
    pub menu_console: &'static str,
    pub menu_quit: &'static str,
    pub menu_language: &'static str,
    pub menu_5h: &'static str,
    pub menu_weekly: &'static str,
    pub menu_month: &'static str,
    pub error: &'static str,
    pub no_data: &'static str,
    pub tooltip_5h: &'static str,
    pub tooltip_month: &'static str,
    pub group_5h: &'static str,
    pub group_weekly: &'static str,
    pub group_month: &'static str,
}

impl Lang {
    /// Language code such as `en`, `zh`, `ja`.
    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
            Lang::Es => "es",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::Ru => "ru",
        }
    }

    /// Parse a language code, returning `None` when unknown.
    pub fn parse(code: &str) -> Option<Self> {
        match code.trim().to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Some(Lang::En),
            "zh" | "zh-cn" | "zh-hans" | "zh-sg" => Some(Lang::Zh),
            "ja" | "jp" => Some(Lang::Ja),
            "ko" | "kr" => Some(Lang::Ko),
            "es" => Some(Lang::Es),
            "fr" => Some(Lang::Fr),
            "de" => Some(Lang::De),
            "ru" => Some(Lang::Ru),
            _ => None,
        }
    }

    /// Native language name, used in the language picker.
    pub fn native_name(&self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "中文",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
            Lang::Es => "Español",
            Lang::Fr => "Français",
            Lang::De => "Deutsch",
            Lang::Ru => "Русский",
        }
    }

    pub fn strings(&self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::Zh => &ZH,
            Lang::Ja => &JA,
            Lang::Ko => &KO,
            Lang::Es => &ES,
            Lang::Fr => &FR,
            Lang::De => &DE,
            Lang::Ru => &RU,
        }
    }
}

const EN: Strings = Strings {
    window_title: "OpenCode Go usage",
    subtitle: "5-hour window  ·  Weekly window  ·  Monthly allowance",
    used: "Used",
    remaining: "Remaining",
    status_ok: "OK",
    status_limited: "Capped",
    reset_after: "until reset",
    last_updated: "Last updated",
    reopen_hint: "Double-click the tray icon to reopen",
    open_console: "Open Console",
    close: "Close",
    menu_refresh: "Refresh now",
    menu_console: "Open OpenCode Console",
    menu_quit: "Quit",
    menu_language: "Language",
    menu_5h: "5h",
    menu_weekly: "Weekly",
    menu_month: "Monthly",
    error: "Error",
    no_data: "No data",
    tooltip_5h: "5h",
    tooltip_month: "Mo",
    group_5h: "5-hour window",
    group_weekly: "Weekly window",
    group_month: "Monthly allowance",
};

const ZH: Strings = Strings {
    window_title: "OpenCode Go 用量",
    subtitle: "5 小时窗口  ·  每周窗口  ·  月度额度",
    used: "已用",
    remaining: "剩余",
    status_ok: "正常",
    status_limited: "已触顶",
    reset_after: "后重置",
    last_updated: "最近更新",
    reopen_hint: "双击托盘图标可重新打开",
    open_console: "打开 Console",
    close: "关闭",
    menu_refresh: "立即刷新",
    menu_console: "打开 OpenCode Console",
    menu_quit: "退出",
    menu_language: "语言",
    menu_5h: "5小时",
    menu_weekly: "每周",
    menu_month: "月度",
    error: "错误",
    no_data: "暂无数据",
    tooltip_5h: "5h",
    tooltip_month: "月",
    group_5h: "5 小时窗口",
    group_weekly: "每周窗口",
    group_month: "月度额度",
};

const JA: Strings = Strings {
    window_title: "OpenCode Go 使用量",
    subtitle: "5時間ウィンドウ  ·  週間ウィンドウ  ·  月間枠",
    used: "使用済み",
    remaining: "残り",
    status_ok: "正常",
    status_limited: "上限到達",
    reset_after: "後にリセット",
    last_updated: "最終更新",
    reopen_hint: "トレイアイコンをダブルクリックで再表示",
    open_console: "Console を開く",
    close: "閉じる",
    menu_refresh: "今すぐ更新",
    menu_console: "OpenCode Console を開く",
    menu_quit: "終了",
    menu_language: "言語",
    menu_5h: "5時間",
    menu_weekly: "週間",
    menu_month: "月間",
    error: "エラー",
    no_data: "データなし",
    tooltip_5h: "5h",
    tooltip_month: "月",
    group_5h: "5時間ウィンドウ",
    group_weekly: "週間ウィンドウ",
    group_month: "月間枠",
};

const KO: Strings = Strings {
    window_title: "OpenCode Go 사용량",
    subtitle: "5시간 창  ·  주간 창  ·  월간 한도",
    used: "사용",
    remaining: "남음",
    status_ok: "정상",
    status_limited: "한도 초과",
    reset_after: "후 초기화",
    last_updated: "마지막 업데이트",
    reopen_hint: "트레이 아이콘을 두 번 클릭하면 다시 열립니다",
    open_console: "Console 열기",
    close: "닫기",
    menu_refresh: "지금 새로고침",
    menu_console: "OpenCode Console 열기",
    menu_quit: "종료",
    menu_language: "언어",
    menu_5h: "5시간",
    menu_weekly: "주간",
    menu_month: "월간",
    error: "오류",
    no_data: "데이터 없음",
    tooltip_5h: "5h",
    tooltip_month: "월",
    group_5h: "5시간 창",
    group_weekly: "주간 창",
    group_month: "월간 한도",
};

const ES: Strings = Strings {
    window_title: "Uso de OpenCode Go",
    subtitle: "Ventana de 5 h  ·  Ventana semanal  ·  Cuota mensual",
    used: "Usado",
    remaining: "Restante",
    status_ok: "OK",
    status_limited: "Límite alcanzado",
    reset_after: "hasta el reinicio",
    last_updated: "Última actualización",
    reopen_hint: "Doble clic en el icono de la bandeja para reabrir",
    open_console: "Abrir Console",
    close: "Cerrar",
    menu_refresh: "Actualizar ahora",
    menu_console: "Abrir OpenCode Console",
    menu_quit: "Salir",
    menu_language: "Idioma",
    menu_5h: "5 h",
    menu_weekly: "Semanal",
    menu_month: "Mensual",
    error: "Error",
    no_data: "Sin datos",
    tooltip_5h: "5 h",
    tooltip_month: "Mes",
    group_5h: "Ventana de 5 h",
    group_weekly: "Ventana semanal",
    group_month: "Cuota mensual",
};

const FR: Strings = Strings {
    window_title: "Utilisation d'OpenCode Go",
    subtitle: "Fenêtre de 5 h  ·  Fenêtre hebdomadaire  ·  Quota mensuel",
    used: "Utilisé",
    remaining: "Restant",
    status_ok: "OK",
    status_limited: "Limite atteinte",
    reset_after: "avant réinitialisation",
    last_updated: "Dernière mise à jour",
    reopen_hint: "Double-cliquez sur l'icône pour rouvrir",
    open_console: "Ouvrir Console",
    close: "Fermer",
    menu_refresh: "Actualiser",
    menu_console: "Ouvrir OpenCode Console",
    menu_quit: "Quitter",
    menu_language: "Langue",
    menu_5h: "5 h",
    menu_weekly: "Semaine",
    menu_month: "Mois",
    error: "Erreur",
    no_data: "Aucune donnée",
    tooltip_5h: "5 h",
    tooltip_month: "Mois",
    group_5h: "Fenêtre de 5 h",
    group_weekly: "Fenêtre hebdomadaire",
    group_month: "Quota mensuel",
};

const DE: Strings = Strings {
    window_title: "OpenCode Go-Nutzung",
    subtitle: "5-Stunden-Fenster  ·  Wochenfenster  ·  Monatskontingent",
    used: "Verbraucht",
    remaining: "Verbleibend",
    status_ok: "OK",
    status_limited: "Limit erreicht",
    reset_after: "bis zum Reset",
    last_updated: "Zuletzt aktualisiert",
    reopen_hint: "Doppelklick auf das Tray-Symbol zum erneuten Öffnen",
    open_console: "Console öffnen",
    close: "Schließen",
    menu_refresh: "Jetzt aktualisieren",
    menu_console: "OpenCode Console öffnen",
    menu_quit: "Beenden",
    menu_language: "Sprache",
    menu_5h: "5 h",
    menu_weekly: "Woche",
    menu_month: "Monat",
    error: "Fehler",
    no_data: "Keine Daten",
    tooltip_5h: "5 h",
    tooltip_month: "Monat",
    group_5h: "5-Stunden-Fenster",
    group_weekly: "Wochenfenster",
    group_month: "Monatskontingent",
};

const RU: Strings = Strings {
    window_title: "Использование OpenCode Go",
    subtitle: "5-часовое окно  ·  Недельное окно  ·  Месячная квота",
    used: "Использовано",
    remaining: "Осталось",
    status_ok: "ОК",
    status_limited: "Достигнут лимит",
    reset_after: "до сброса",
    last_updated: "Последнее обновление",
    reopen_hint: "Дважды щёлкните по значку, чтобы открыть снова",
    open_console: "Открыть Console",
    close: "Закрыть",
    menu_refresh: "Обновить",
    menu_console: "Открыть OpenCode Console",
    menu_quit: "Выход",
    menu_language: "Язык",
    menu_5h: "5 ч",
    menu_weekly: "Неделя",
    menu_month: "Месяц",
    error: "Ошибка",
    no_data: "Нет данных",
    tooltip_5h: "5 ч",
    tooltip_month: "Мес",
    group_5h: "5-часовое окно",
    group_weekly: "Недельное окно",
    group_month: "Месячная квота",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_provides_every_key() {
        // The struct guarantees field presence at compile time; this check
        // just makes sure no entry is accidentally left empty.
        let all = [Lang::En, Lang::Zh, Lang::Ja, Lang::Ko, Lang::Es, Lang::Fr, Lang::De, Lang::Ru];
        for lang in all {
            let s = lang.strings();
            assert!(!s.window_title.is_empty());
            assert!(!s.subtitle.is_empty());
            assert!(!s.used.is_empty());
            assert!(!s.remaining.is_empty());
            assert!(!s.status_ok.is_empty());
            assert!(!s.status_limited.is_empty());
            assert!(!s.reset_after.is_empty());
            assert!(!s.last_updated.is_empty());
            assert!(!s.reopen_hint.is_empty());
            assert!(!s.open_console.is_empty());
            assert!(!s.close.is_empty());
            assert!(!s.menu_refresh.is_empty());
            assert!(!s.menu_console.is_empty());
            assert!(!s.menu_quit.is_empty());
            assert!(!s.menu_language.is_empty());
            assert!(!s.menu_5h.is_empty());
            assert!(!s.menu_weekly.is_empty());
            assert!(!s.menu_month.is_empty());
            assert!(!s.error.is_empty());
            assert!(!s.no_data.is_empty());
            assert!(!s.tooltip_5h.is_empty());
            assert!(!s.tooltip_month.is_empty());
            assert!(!s.group_5h.is_empty());
            assert!(!s.group_weekly.is_empty());
            assert!(!s.group_month.is_empty());
        }
    }

    #[test]
    fn language_codes_round_trip() {
        for lang in [Lang::En, Lang::Zh, Lang::Ja, Lang::Ko, Lang::Es, Lang::Fr, Lang::De, Lang::Ru] {
            assert_eq!(Lang::parse(lang.code()), Some(lang));
        }
    }
}
