use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Japanese,
}

impl Language {
    pub fn variants() -> &'static [Language] {
        &[Language::English, Language::Japanese]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Japanese => "日本語",
        }
    }
}

pub fn localize(key: &str, lang: Language) -> String {
    match lang {
        Language::English => key.to_string(), // Default fallback is the key itself for English usually, but we'll map explicitly if keys are identifiers
        Language::Japanese => localize_ja(key),
    }
}

pub fn localize_enum(key: &str, lang: Language) -> String {
     match lang {
        Language::English => key.to_string(),
        Language::Japanese => localize_ja(key),
    }
}


fn localize_ja(key: &str) -> String {
    match key {
        // Settings Dialog
        "Settings" => "設定".to_string(),
        "Language" => "言語".to_string(),
        
        // Menu
        "File" => "ファイル".to_string(),
        "Open File" => "ファイルを開く".to_string(),
        "Open Directory" => "ディレクトリを開く".to_string(),
        "Quit" => "終了".to_string(),
        "About" => "バージョン情報".to_string(),
        "Preferences" => "設定".to_string(),
        "Directory Defaults (Applied to new directories)" => "ディレクトリのデフォルト設定 (新規ディレクトリに適用)".to_string(),
        "Default Spread View" => "見開き表示をデフォルトにする".to_string(),
        "Default Right to Left" => "右開きをデフォルトにする".to_string(),
        "Default Dir Sort:" => "デフォルトのディレクトリ並び順:".to_string(),
        "Default Image Sort:" => "デフォルトの画像並び順:".to_string(),
        "Application Settings" => "アプリケーション設定".to_string(),
        "Dark Mode (Requires Restart)" => "ダークモード (再起動が必要)".to_string(),
        "Loop Images (at end of list)" => "画像をループする (リストの最後で)".to_string(),
        "Single Page for First Image (Spread View)" => "最初の画像を単ページ表示 (見開き表示時)".to_string(),
        "Input Configuration" => "入力設定".to_string(),
        "Reset to Defaults" => "デフォルトに戻す".to_string(),
        "Keyboard Shortcuts" => "キーボードショートカット".to_string(),
        "Mouse Configuration" => "マウス設定".to_string(),
        "Cancel" => "キャンセル".to_string(),
        "Save" => "保存".to_string(),
        "Press a key (Esc to cancel)" => "キーを押してください (Escでキャンセル)".to_string(),
        
        // Sort Types
        "Name Asc" => "名前 (昇順)".to_string(),
        "Name Desc" => "名前 (降順)".to_string(),
        "Date Asc" => "日付 (昇順)".to_string(),
        "Date Desc" => "日付 (降順)".to_string(),
        "Size Asc" => "サイズ (昇順)".to_string(),
        "Size Desc" => "サイズ (降順)".to_string(),

        // Mouse Inputs
        "Right Click" => "右クリック".to_string(),
        "Middle Click" => "中クリック".to_string(),
        "Scroll Up" => "スクロール上".to_string(),
        "Scroll Down" => "スクロール下".to_string(),
        "Left Double Click" => "左ダブルクリック".to_string(),

        // Actions
        "Previous Directory / Archive" => "前のディレクトリ/アーカイブ".to_string(),
        "Next Directory / Archive" => "次のディレクトリ/アーカイブ".to_string(),
        "Previous Image" => "前の画像".to_string(),
        "Next Image" => "次の画像".to_string(),
        "Toggle Fullscreen" => "フルスクリーン切り替え".to_string(),
        "Zoom In" => "ズームイン".to_string(),
        "Zoom Out" => "ズームアウト".to_string(),
        "Reset Zoom" => "ズームリセット".to_string(),
        "Toggle Spread View" => "見開き表示切り替え".to_string(),
        "Toggle Right-to-Left" => "右開き切り替え".to_string(),
        "Previous Image (Single Step)" => "前の画像 (1ページ)".to_string(),
        "Next Image (Single Step)" => "次の画像 (1ページ)".to_string(),
        
        "None" => "なし".to_string(),

        _ => key.to_string(),
    }
}
