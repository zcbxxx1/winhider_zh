use eframe::egui;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    English,
    ChineseSimplified,
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

impl Language {
    pub const ALL: [Self; 2] = [Self::English, Self::ChineseSimplified];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::ChineseSimplified => "中文（简体）",
        }
    }
}

pub fn install_locale_fonts(ctx: &egui::Context) {
    let Some(font_bytes) = [
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simsun.ttc",
    ]
    .iter()
    .find_map(|path| std::fs::read(path).ok()) else {
        return;
    };

    let font_name = "winhider_cjk".to_owned();
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert(font_name.clone(), egui::FontData::from_owned(font_bytes));

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push(font_name.clone());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(font_name);

    ctx.set_fonts(fonts);
}

pub fn tr(language: Language, english: &'static str) -> &'static str {
    match language {
        Language::English => english,
        Language::ChineseSimplified => zh_cn(english).unwrap_or(english),
    }
}

// Keep upstream English UI text as the lookup key. If upstream adds or changes a
// string, the app falls back to English instead of failing to build or render.
pub fn tr_format(
    language: Language,
    english_template: &'static str,
    args: &[(&str, String)],
) -> String {
    let mut translated = tr(language, english_template).to_owned();
    for (name, value) in args {
        translated = translated.replace(&format!("{{{name}}}"), value);
    }
    translated
}

fn zh_cn(english: &'static str) -> Option<&'static str> {
    Some(match english {
        "Ready." => "就绪。",
        "Language changed." => "语言已切换。",
        "Failed to start Graphics Capture" => "启动图形捕获失败",
        "Ctrl+S: Toggled capture for {count} windows" => {
            "Ctrl+S：已切换 {count} 个窗口的截图隐藏状态"
        }
        "Ctrl+S: Failed to toggle capture" => "Ctrl+S：切换截图隐藏失败",
        "Ctrl+S: No windows selected" => "Ctrl+S：未选择窗口",
        "Ctrl+T: Toggled taskbar for {count} windows" => {
            "Ctrl+T：已切换 {count} 个窗口的任务栏隐藏状态"
        }
        "Ctrl+T: Failed to toggle taskbar" => "Ctrl+T：切换任务栏隐藏失败",
        "Ctrl+T: No windows selected" => "Ctrl+T：未选择窗口",
        "File" => "文件",
        "Launch CLI" => "启动 CLI",
        "Failed to launch CLI: {error}" => "启动 CLI 失败：{error}",
        "CLI launched successfully." => "CLI 已成功启动。",
        "Clear Temp Files" => "清理临时文件",
        "Temporary injection files cleaned." => "临时注入文件已清理。",
        "Exit" => "退出",
        "Settings" => "设置",
        "Enable Auto-Updates" => "启用自动更新",
        "Language" => "语言",
        "Preview Quality" => "预览质量",
        "Low (Fastest)" => "低（最快）",
        "Medium (Balanced)" => "中（平衡）",
        "High (Best)" => "高（最佳）",
        "Help" => "帮助",
        "Check for Updates" => "检查更新",
        "About" => "关于",
        "Show Preview" => "显示预览",
        "Monitor {index}" => "显示器 {index}",
        "Waiting for WGC Stream..." => "正在等待 WGC 画面流……",
        "Auto-Hide on Start" => "启动时自动隐藏",
        "{count} apps configured" => "已配置 {count} 个应用",
        "Edit List" => "编辑列表",
        "Local Stealth" => "本机隐身",
        "Hide Self from Capture" => "在截图中隐藏自身",
        "Hide Self from Taskbar" => "从任务栏隐藏自身",
        "Target Applications" => "目标应用",
        "🔄 Force Refresh" => "🔄 强制刷新",
        "Reset Hidden States" => "重置隐藏状态",
        "Restore all windows currently marked as hidden by WinHider." => {
            "恢复所有当前被 WinHider 标记为隐藏的窗口。"
        }
        "This will restore all windows currently marked as hidden by WinHider." => {
            "这会恢复所有当前被 WinHider 标记为隐藏的窗口。"
        }
        "The startup auto-hide list will not be changed." => "启动时自动隐藏列表不会被修改。",
        "Reset Now" => "立即重置",
        "Reset completed: restored {count} hidden states." => {
            "重置完成：已恢复 {count} 个隐藏状态。"
        }
        "Reset completed with {errors} errors; restored {count} hidden states." => {
            "重置完成，但有 {errors} 个错误；已恢复 {count} 个隐藏状态。"
        }
        "No hidden states to reset." => "没有需要重置的隐藏状态。",
        "List refreshed." => "列表已刷新。",
        "Hotkeys: Ctrl+S=Toggle Capture, Ctrl+T=Toggle Taskbar (select windows first, Ctrl+click for multi-select)" => {
            "快捷键：Ctrl+S=切换截图隐藏，Ctrl+T=切换任务栏隐藏（请先选择窗口，Ctrl+点击可多选）"
        }
        "Selected: {count} windows" => "已选择：{count} 个窗口",
        "← Selected" => "← 已选择",
        "Add to Startup Auto-Hide" => "加入启动隐藏",
        "Already in startup auto-hide list." => "已在启动自动隐藏列表中。",
        "Add this window title to startup auto-hide list." => {
            "把这个窗口标题加入启动自动隐藏列表。"
        }
        "Added to startup auto-hide: {title}" => "已加入启动自动隐藏：{title}",
        "Already in startup auto-hide: {title}" => "已在启动自动隐藏中：{title}",
        "Hide Taskbar" => "隐藏任务栏",
        "Hide Capture" => "隐藏截图",
        "Error: {error}" => "错误：{error}",
        "Capture state updated: {title}" => "截图隐藏状态已更新：{title}",
        "Checking GitHub for updates..." => "正在从 GitHub 检查更新……",
        "✓ You are up to date!" => "✓ 当前已是最新版本！",
        "Current version: {version}" => "当前版本：{version}",
        "Close" => "关闭",
        "New Version Available!" => "发现新版本！",
        "Current: {version}" => "当前：{version}",
        "Latest: {version}" => "最新：{version}",
        "⬇ Download Installer" => "⬇ 下载安装程序",
        "⚠ Check Failed" => "⚠ 检查失败",
        "About {app}" => "关于 {app}",
        "Version {version}" => "版本 {version}",
        "Latest: Checking..." => "最新版本：检查中……",
        "Professional Window Visibility Controller" => "专业窗口可见性控制器",
        "WinHider allows advanced control over how application windows appear to the system and screen capture software." => {
            "WinHider 可高级控制应用窗口在系统与截图/录屏软件中的呈现方式。"
        }
        "• Hide windows from screen capture (OBS, Teams, Zoom)" => {
            "• 在截图/录屏中隐藏窗口（OBS、Teams、Zoom）"
        }
        "• Remove windows from Taskbar and Alt-Tab" => "• 从任务栏和 Alt-Tab 中移除窗口",
        "• Maintain normal usability while hidden" => "• 隐藏时仍保持正常可用",
        "• Auto-Hide on startup based on custom list" => "• 按自定义列表在启动时自动隐藏",
        "Auto-Hide Applications" => "自动隐藏应用",
        "Applications to automatically hide on startup:" => "启动时要自动隐藏的应用：",
        "Remove" => "移除",
        "Add application:" => "添加应用：",
        "Add" => "添加",
        "Save & Close" => "保存并关闭",
        "Cancel" => "取消",
        "Failed to save auto-hide list: {error}" => "保存自动隐藏列表失败：{error}",
        "Auto-hide list saved." => "自动隐藏列表已保存。",
        _ => return None,
    })
}
