use std::fs;
use std::path::Path;
use ratatui::style::Color;

// =================================================================
//  Config — 主配置结构体
// =================================================================

/// 全局应用配置，由 `dfree.toml` 加载，若无配置文件则使用内置默认值
pub struct Config {
    /// 调色板（所有颜色值）
    pub palette: Palette,
    /// 各组件间距配置
    pub spacing: Spacing,
    /// 界面文字标签（中英文对照）
    pub labels: Labels,
    /// 各分类文件详情的体积下限（低于此值只计总数，不记录文件路径）
    pub thresholds: Thresholds,
}

impl Config {
    /// 尝试从 `dfree.toml` 加载配置；文件不存在时返回内置默认值
    pub fn load() -> Self {
        let config_path = Path::new("dfree.toml");
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => {
                    match toml::from_str::<ConfigFile>(&content) {
                        Ok(cfg_file) => Config::from_file(cfg_file),
                        Err(e) => {
                            eprintln!("⚠  配置文件解析出错，使用默认配置: {}", e);
                            Config::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠  读取配置文件失败，使用默认配置: {}", e);
                    Config::default()
                }
            }
        } else {
            Config::default()
        }
    }

    /// 将 TOML 反序列化得到的中间结构体转换为类型化的 Config
    fn from_file(f: ConfigFile) -> Self {
        Config {
            palette: Palette::from_raw(f.palette),
            spacing: Spacing::from_raw(f.spacing),
            labels: Labels::from_raw(f.labels),
            thresholds: Thresholds::from_raw(f.thresholds),
        }
    }

    /// 内置默认配置（莫兰迪配色）
    pub fn default() -> Self {
        Config {
            palette: Palette::from_raw(PaletteRaw::default()),
            spacing: Spacing::from_raw(SpacingRaw::default()),
            labels: Labels::default(),
            thresholds: Thresholds::default(),
        }
    }
}

// =================================================================
//  调色板 (Palette)
// =================================================================

/// 颜色调色板，所有值均为莫兰迪低饱和色系
pub struct Palette {
    /// 弹窗背景色
    pub popup_bg: Color,
    /// 弹窗边框色
    pub popup_border: Color,
    /// 主文字颜色
    pub text_primary: Color,
    /// 辅助文字颜色
    pub text_secondary: Color,
    /// 强调文字颜色（如标题、数值）
    pub text_highlight: Color,

    // ---- 系统信息栏标签 ----
    /// CPU 标签颜色
    pub label_cpu: Color,
    /// Memory 标签颜色
    pub label_memory: Color,
    /// Swap 标签颜色
    pub label_swap: Color,
    /// Disk I/O 标签颜色
    pub label_disk_io: Color,
    /// Total 标签颜色
    pub label_total: Color,

    // ---- 快捷键 / 按钮 ----
    /// 快捷键按键颜色（如 [Enter], [q]）
    pub key_binding: Color,
    /// 快捷键说明文字颜色
    pub key_desc: Color,

    // ---- 存储分类（分析弹窗） ----
    /// 文档分类颜色
    pub cat_documents: Color,
    /// 图片分类颜色
    pub cat_pictures: Color,
    /// 音频分类颜色
    pub cat_audio: Color,
    /// 视频分类颜色
    pub cat_video: Color,
    /// 其他分类颜色
    pub cat_other: Color,
    /// 应用分类颜色
    pub cat_applications: Color,
    /// 系统分类颜色
    pub cat_system: Color,
    /// 缓存分类颜色
    pub cat_cache: Color,

    // ---- 仪表盘 (Gauge) ----
    /// 扫描完成时的仪表盘颜色
    pub gauge_done: Color,
    /// 扫描中的仪表盘颜色
    pub gauge_scanning: Color,
    /// 正常使用率颜色（< 70%）
    pub gauge_ok: Color,
    /// 警告使用率颜色（70% ~ 90%）
    pub gauge_warn: Color,
    /// 危险使用率颜色（>= 90%）
    pub gauge_danger: Color,

    // ---- 表格 ----
    /// 表头文字颜色
    pub table_header: Color,
    /// 表格行文字颜色
    pub table_row: Color,
    /// 选中行背景色
    pub table_selected_bg: Color,

    // ---- 窗口标题 ----
    /// 标题文字颜色
    pub title_text: Color,
    /// 标题边框颜色
    pub title_border: Color,
}

impl Palette {
    /// 从 TOML 反序列化的原始字符串字典构建调色板
    fn from_raw(r: PaletteRaw) -> Self {
        Palette {
            popup_bg:         hex_or(&r.popup_bg, "#2d2d2d"),
            popup_border:     hex_or(&r.popup_border, "#4a4a4a"),
            text_primary:     hex_or(&r.text_primary, "#d1d1c6"),
            text_secondary:   hex_or(&r.text_secondary, "#8e8e84"),
            text_highlight:   hex_or(&r.text_highlight, "#ffffff"),
            label_cpu:        hex_or(&r.label_cpu, "#b5a89a"),
            label_memory:     hex_or(&r.label_memory, "#a8b5a0"),
            label_swap:       hex_or(&r.label_swap, "#b59aa0"),
            label_disk_io:    hex_or(&r.label_disk_io, "#9aa8b5"),
            label_total:      hex_or(&r.label_total, "#8e8e84"),
            key_binding:      hex_or(&r.key_binding, "#b59a7a"),
            key_desc:         hex_or(&r.key_desc, "#8e8e84"),
            cat_documents:    hex_or(&r.cat_documents, "#9aa8b5"),
            cat_pictures:     hex_or(&r.cat_pictures, "#b5a89a"),
            cat_audio:        hex_or(&r.cat_audio, "#b59aa0"),
            cat_video:        hex_or(&r.cat_video, "#a8b5a0"),
            cat_other:        hex_or(&r.cat_other, "#a0b5a8"),
            cat_applications: hex_or(&r.cat_applications, "#8ea8b5"),
            cat_system:       hex_or(&r.cat_system, "#b5a0a0"),
            cat_cache:        hex_or(&r.cat_cache, "#b5a88a"),
            gauge_done:       hex_or(&r.gauge_done, "#8fa88f"),
            gauge_scanning:   hex_or(&r.gauge_scanning, "#8f9fa8"),
            gauge_ok:         hex_or(&r.gauge_ok, "#8aa88a"),
            gauge_warn:       hex_or(&r.gauge_warn, "#b59a7a"),
            gauge_danger:     hex_or(&r.gauge_danger, "#b5807a"),
            table_header:     hex_or(&r.table_header, "#8e8e84"),
            table_row:        hex_or(&r.table_row, "#d1d1c6"),
            table_selected_bg: hex_or(&r.table_selected_bg, "#3a3a4a"),
            title_text:       hex_or(&r.title_text, "#d1d1c6"),
            title_border:     hex_or(&r.title_border, "#4a4a4a"),
        }
    }

    /// 根据使用率百分比返回对应的仪表盘颜色
    pub fn usage_color(&self, pct: f64) -> Color {
        if pct >= 90.0 { self.gauge_danger }
        else if pct >= 70.0 { self.gauge_warn }
        else { self.gauge_ok }
    }
}

// =================================================================
//  间距配置 (Spacing)
// =================================================================

/// 间距配置 —— 按组件划分，每个组件有独立的间距值
pub struct Spacing {
    /// 窗口标题间距
    pub title: TitleSpacing,
    /// 系统信息栏间距
    pub sys_header: SysHeaderSpacing,
    /// 磁盘列表间距
    pub volume_table: VolumeTableSpacing,
    /// 底部快捷键栏间距
    pub footer: FooterSpacing,
    /// 弹窗通用间距
    pub popup: PopupSpacing,
    /// 分析弹窗专属间距
    pub analysis: AnalysisSpacing,
    /// 大文件弹窗专属间距
    pub large_files: LargeFilesSpacing,
    /// 详情弹窗专属间距
    pub detail: DetailSpacing,
}

impl Spacing { fn from_raw(r: SpacingRaw) -> Self { Spacing { title: TitleSpacing::from_raw(r.components.title), sys_header: SysHeaderSpacing::from_raw(r.components.sys_header), volume_table: VolumeTableSpacing::from_raw(r.components.volume_table), footer: FooterSpacing::from_raw(r.components.footer), popup: PopupSpacing::from_raw(r.components.popup), analysis: AnalysisSpacing::from_raw(r.components.analysis), large_files: LargeFilesSpacing::from_raw(r.components.large_files), detail: DetailSpacing::from_raw(r.components.detail), } } fn default() -> Self { Spacing::from_raw(SpacingRaw::default()) } }

/// 窗口标题 —— 标题文字左右两侧的空白字符数
pub struct TitleSpacing {
    /// 标题左侧空格数（默认 1）
    pub left: u16,
    /// 标题右侧空格数（默认 1）
    pub right: u16,
}
impl TitleSpacing { fn from_raw(r: TitleSpacingRaw) -> Self { TitleSpacing { left: r.left.unwrap_or(1), right: r.right.unwrap_or(1) } } }

/// 系统信息栏 —— CPU / Memory / Swap / Disk I/O 所在的两行
pub struct SysHeaderSpacing {
    /// 标签和数值之间的空格数，如 "CPU:" 和 "12.5%" 之间（默认 1）
    pub label_value_gap: u16,
    /// 不同指标组之间的空格数（默认 2）
    pub item_gap: u16,
    /// 两行系统信息之间的空行数（默认 0）
    pub line_gap: u16,
    /// 组与组之间的分隔符字符串（默认 "  │  "）
    pub separator: String,
}
impl SysHeaderSpacing { fn from_raw(r: SysHeaderSpacingRaw) -> Self { SysHeaderSpacing { label_value_gap: r.label_value_gap.unwrap_or(1), item_gap: r.item_gap.unwrap_or(2), line_gap: r.line_gap.unwrap_or(0), separator: r.separator.unwrap_or_else(|| "  │  ".to_string()), } } }

/// 磁盘列表 —— 表头和每一行的前缀空格
pub struct VolumeTableSpacing {
    /// 表头文字前的空格数，如 "Volumes" 前面（默认 1）
    pub header_prefix: u16,
    /// 每行文字前的空格数（默认 1）
    pub row_prefix: u16,
    /// 行与行之间的空行数（默认 0）
    pub row_gap: u16,
}
impl VolumeTableSpacing { fn from_raw(r: VolumeTableSpacingRaw) -> Self { VolumeTableSpacing { header_prefix: r.header_prefix.unwrap_or(1), row_prefix: r.row_prefix.unwrap_or(1), row_gap: r.row_gap.unwrap_or(0), } } }

/// 底部快捷键栏 —— 底部操作提示行
pub struct FooterSpacing {
    /// 底部栏左端空格数（默认 1）
    pub prefix: u16,
    /// 快捷键按键和说明文字之间的空格，如 "[↑↓] 选择"（默认 1）
    pub key_desc_gap: u16,
    /// 不同快捷键组之间的空格数（默认 2）
    pub group_gap: u16,
}
impl FooterSpacing { fn from_raw(r: FooterSpacingRaw) -> Self { FooterSpacing { prefix: r.prefix.unwrap_or(1), key_desc_gap: r.key_desc_gap.unwrap_or(1), group_gap: r.group_gap.unwrap_or(2), } } }

/// 弹窗通用间距 —— 所有弹窗共享的间距
pub struct PopupSpacing {
    /// 弹窗标题左右两端的空格数（默认 1）
    pub title_padding_x: u16,
    /// 弹窗内容区左侧空格数（默认 1）
    pub content_left: u16,
    /// 弹窗内容区右侧空格数（默认 1）
    pub content_right: u16,
    /// 弹窗内容区顶部空行数（默认 0）
    pub content_top: u16,
    /// 弹窗内容区底部空行数（默认 0）
    pub content_bottom: u16,
    /// 帮助文字上方的空行数（默认 1）
    pub footer_gap_top: u16,
}
impl PopupSpacing { fn from_raw(r: PopupSpacingRaw) -> Self { PopupSpacing { title_padding_x: r.title_padding_x.unwrap_or(1), content_left: r.content_left.unwrap_or(1), content_right: r.content_right.unwrap_or(1), content_top: r.content_top.unwrap_or(0), content_bottom: r.content_bottom.unwrap_or(0), footer_gap_top: r.footer_gap_top.unwrap_or(1), } } }

/// 分析弹窗专属间距
pub struct AnalysisSpacing {
    /// 分类列表行之间的空行数（默认 0）
    pub category_gap: u16,
    /// 仪表盘上方空行数（默认 0）
    pub gauge_margin_top: u16,
    /// 仪表盘下方空行数（默认 0）
    pub gauge_margin_bottom: u16,
    /// 仪表盘高度（行数），默认 1
    pub gauge_height: u16,
    /// 仪表盘下方空行数（默认 1）
    pub gauge_gap: u16,
}
impl AnalysisSpacing { fn from_raw(r: AnalysisSpacingRaw) -> Self { AnalysisSpacing { category_gap: r.category_gap.unwrap_or(0), gauge_margin_top: r.gauge_margin_top.unwrap_or(0), gauge_margin_bottom: r.gauge_margin_bottom.unwrap_or(0), gauge_height: r.gauge_height.unwrap_or(1), gauge_gap: r.gauge_gap.unwrap_or(1), } } }

/// 大文件弹窗专属间距
pub struct LargeFilesSpacing {
    /// 文件列表行之间的空行数（默认 0）
    pub item_gap: u16,
    /// 列表上方空行数（默认 0）
    pub list_margin_top: u16,
}
impl LargeFilesSpacing { fn from_raw(r: LargeFilesSpacingRaw) -> Self { LargeFilesSpacing { item_gap: r.item_gap.unwrap_or(0), list_margin_top: r.list_margin_top.unwrap_or(0), } } }

/// 详情弹窗专属间距
pub struct DetailSpacing {
    /// 信息行之间的空行数（默认 0）
    pub line_gap: u16,
    /// 仪表盘上方空行数（默认 1）
    pub gauge_margin_top: u16,
    /// 仪表盘下方空行数（默认 0）
    pub gauge_margin_bottom: u16,
}
impl DetailSpacing { fn from_raw(r: DetailSpacingRaw) -> Self { DetailSpacing { line_gap: r.line_gap.unwrap_or(0), gauge_margin_top: r.gauge_margin_top.unwrap_or(1), gauge_margin_bottom: r.gauge_margin_bottom.unwrap_or(0), } } }

// =================================================================
//  体积下限配置 (Thresholds)
// =================================================================

/// 各分类收录文件详情的体积下限（字节），低于此值的文件只累加分类总数，不记录路径
#[derive(Clone)]
pub struct Thresholds {
    pub documents: u64,
    pub pictures: u64,
    pub audio: u64,
    pub video: u64,
    pub other: u64,
    pub applications: u64,
    pub cache: u64,
}

impl Thresholds {
    pub fn get(&self, cat: u8) -> u64 {
        match cat {
            0 => self.documents,
            1 => self.pictures,
            2 => self.audio,
            3 => self.video,
            4 => self.other,
            5 => self.applications,
            6 => u64::MAX,      // System 永不收录文件详情
            7 => self.cache,
            _ => 0,
        }
    }
    fn default() -> Self {
        Thresholds {
            documents:    1 * 1024 * 1024,
            pictures:     5 * 1024 * 1024,
            audio:        5 * 1024 * 1024,
            video:       50 * 1024 * 1024,
            other:        5 * 1024 * 1024,
            applications:10 * 1024 * 1024,
            cache:        1 * 1024 * 1024,
        }
    }
    fn from_raw(r: ThresholdsRaw) -> Self {
        Thresholds {
            documents:    mb(r.documents).unwrap_or(1),
            pictures:     mb(r.pictures).unwrap_or(5),
            audio:        mb(r.audio).unwrap_or(5),
            video:        mb(r.video).unwrap_or(50),
            other:        mb(r.other).unwrap_or(5),
            applications: mb(r.applications).unwrap_or(10),
            cache:        mb(r.cache).unwrap_or(1),
        }
    }
}

fn mb(v: Option<u64>) -> Option<u64> { v.map(|m| m * 1024 * 1024) }

// =================================================================
//  界面文字标签 (Labels)
// =================================================================

/// 界面文字标签 —— 所有界面显示的文本，默认中英文对照
pub struct Labels {
    // ---- 系统信息栏 ----
    pub cpu: String,
    pub memory: String,
    pub swap: String,
    pub disk_io: String,
    pub total: String,

    // ---- 分析弹窗 ----
    pub storage_analysis: String,
    pub category: String,
    pub size: String,
    pub pct: String,
    pub documents: String,
    pub pictures: String,
    pub audio: String,
    pub video: String,
    pub other: String,
    pub applications: String,
    pub system: String,
    pub cache: String,
    pub cat_total: String,
    pub scan_starting: String,
    pub scan_scanning: String,
    pub scan_complete: String,
    pub dirs_scanned: String,

    // ---- 大文件弹窗 ----
    pub large_files: String,
    pub largest_files_prefix: String,
    pub num: String,
    pub file_size: String,
    pub path: String,

    // ---- 详情弹窗 ----
    pub volume_details: String,
    pub volume_label: String,
    pub file_system: String,
    pub capacity: String,
    pub available: String,

    // ---- 分类文件弹窗 ----
    pub modified: String,
    pub sort_by_size: String,
    pub sort_by_name: String,
    pub sort_by_time: String,
    pub file_detail_title: String,
    pub path_label: String,
    pub mtime_label: String,
    pub size_label: String,
    pub cat_label: String,

    // ---- 帮助菜单 ----
    pub help_title: String,
    pub help_general: String,
    pub help_main: String,
    pub help_analysis: String,
    pub help_cat_files: String,
    pub help_detail: String,
    pub help_exit: String,

    // ---- 通用 ----
    pub back: String,
    pub large_files_btn: String,
}

impl Labels {
    fn default() -> Self { Labels::from_raw(LabelsRaw::default()) }
    fn from_raw(r: LabelsRaw) -> Self {
        Labels {
            cpu:            r.cpu.unwrap_or_else(|| "CPU (处理器)".into()),
            memory:         r.memory.unwrap_or_else(|| "Memory (内存)".into()),
            swap:           r.swap.unwrap_or_else(|| "Swap (交换)".into()),
            disk_io:        r.disk_io.unwrap_or_else(|| "Disk I/O (磁盘)".into()),
            total:          r.total.unwrap_or_else(|| "Total (总计)".into()),
            storage_analysis: r.storage_analysis.unwrap_or_else(|| "Storage Analysis (存储分析)".into()),
            category:       r.category.unwrap_or_else(|| "Category (分类)".into()),
            size:           r.size.unwrap_or_else(|| "Size (大小)".into()),
            pct:            r.pct.unwrap_or_else(|| "%".into()),
            documents:      r.documents.unwrap_or_else(|| "Documents (文档)".into()),
            pictures:       r.pictures.unwrap_or_else(|| "Pictures (图片)".into()),
            audio:          r.audio.unwrap_or_else(|| "Audio (音频)".into()),
            video:          r.video.unwrap_or_else(|| "Video (视频)".into()),
            other:          r.other.unwrap_or_else(|| "Other (其他)".into()),
            applications:   r.applications.unwrap_or_else(|| "Applications (应用)".into()),
            system:         r.system.unwrap_or_else(|| "System (系统)".into()),
            cache:          r.cache.unwrap_or_else(|| "Cache (缓存)".into()),
            cat_total:      r.cat_total.unwrap_or_else(|| "Total (总计)".into()),
            scan_starting:  r.scan_starting.unwrap_or_else(|| "Starting scan... (开始扫描...)".into()),
            scan_scanning:  r.scan_scanning.unwrap_or_else(|| "Scanning (扫描中)".into()),
            scan_complete:  r.scan_complete.unwrap_or_else(|| "Complete (完成)".into()),
            dirs_scanned:   r.dirs_scanned.unwrap_or_else(|| "directories scanned (已扫描目录)".into()),
            large_files:    r.large_files.unwrap_or_else(|| "Large Files (大文件)".into()),
            largest_files_prefix: r.largest_files_prefix.unwrap_or_else(|| "Top (前)".into()),
            num:            r.num.unwrap_or_else(|| "#".into()),
            file_size:      r.file_size.unwrap_or_else(|| "Size (大小)".into()),
            path:           r.path.unwrap_or_else(|| "Path (路径)".into()),
            volume_details: r.volume_details.unwrap_or_else(|| "Volume Details (卷详情)".into()),
            volume_label:   r.volume_label.unwrap_or_else(|| "Volume (卷标)".into()),
            file_system:    r.file_system.unwrap_or_else(|| "File System (文件系统)".into()),
            capacity:       r.capacity.unwrap_or_else(|| "Capacity (容量)".into()),
            available:      r.available.unwrap_or_else(|| "Available (可用)".into()),
            modified:       r.modified.unwrap_or_else(|| "Modified (修改时间)".into()),
            sort_by_size:   r.sort_by_size.unwrap_or_else(|| "Size (大小)".into()),
            sort_by_name:   r.sort_by_name.unwrap_or_else(|| "Name (名称)".into()),
            sort_by_time:   r.sort_by_time.unwrap_or_else(|| "Time (时间)".into()),
            file_detail_title: r.file_detail_title.unwrap_or_else(|| "File Details (文件详情)".into()),
            path_label:     r.path_label.unwrap_or_else(|| "Path (路径)".into()),
            mtime_label:    r.mtime_label.unwrap_or_else(|| "Modified (修改时间)".into()),
            size_label:     r.size_label.unwrap_or_else(|| "Size (大小)".into()),
            cat_label:      r.cat_label.unwrap_or_else(|| "Category (分类)".into()),
            help_title:     r.help_title.unwrap_or_else(|| "Help (帮助)".into()),
            help_general:   r.help_general.unwrap_or_else(|| "General (通用)".into()),
            help_main:      r.help_main.unwrap_or_else(|| "Main (主界面)".into()),
            help_analysis:  r.help_analysis.unwrap_or_else(|| "Analysis (分析弹窗)".into()),
            help_cat_files: r.help_cat_files.unwrap_or_else(|| "Category Files (分类文件)".into()),
            help_detail:    r.help_detail.unwrap_or_else(|| "Volume Details (卷详情)".into()),
            help_exit:      r.help_exit.unwrap_or_else(|| "Exit (退出)".into()),
            back:           r.back.unwrap_or_else(|| "back (返回)".into()),
            large_files_btn: r.large_files_btn.unwrap_or_else(|| "large files (大文件)".into()),
        }
    }
}

// =================================================================
//  TOML 反序列化中间结构体（私有，不对外暴露）
// =================================================================

use serde::Deserialize;

#[derive(Deserialize)]
struct ConfigFile {
    palette: PaletteRaw,
    spacing: SpacingRaw,
    #[serde(default)]
    labels: LabelsRaw,
    #[serde(default)]
    thresholds: ThresholdsRaw,
}

#[derive(Default, Deserialize)]
struct PaletteRaw {
    #[serde(default)] popup_bg: String,
    #[serde(default)] popup_border: String,
    #[serde(default)] text_primary: String,
    #[serde(default)] text_secondary: String,
    #[serde(default)] text_highlight: String,
    #[serde(default)] label_cpu: String,
    #[serde(default)] label_memory: String,
    #[serde(default)] label_swap: String,
    #[serde(default)] label_disk_io: String,
    #[serde(default)] label_total: String,
    #[serde(default)] key_binding: String,
    #[serde(default)] key_desc: String,
    #[serde(default)] cat_documents: String,
    #[serde(default)] cat_pictures: String,
    #[serde(default)] cat_audio: String,
    #[serde(default)] cat_video: String,
    #[serde(default)] cat_other: String,
    #[serde(default)] cat_applications: String,
    #[serde(default)] cat_system: String,
    #[serde(default)] cat_cache: String,
    #[serde(default)] gauge_done: String,
    #[serde(default)] gauge_scanning: String,
    #[serde(default)] gauge_ok: String,
    #[serde(default)] gauge_warn: String,
    #[serde(default)] gauge_danger: String,
    #[serde(default)] table_header: String,
    #[serde(default)] table_row: String,
    #[serde(default)] table_selected_bg: String,
    #[serde(default)] title_text: String,
    #[serde(default)] title_border: String,
}

#[derive(Default, Deserialize)]
struct SpacingRaw {
    #[serde(default)]
    components: ComponentsRaw,
}

#[derive(Default, Deserialize)]
struct ComponentsRaw {
    #[serde(default)]
    title: TitleSpacingRaw,
    #[serde(default)]
    sys_header: SysHeaderSpacingRaw,
    #[serde(default)]
    volume_table: VolumeTableSpacingRaw,
    #[serde(default)]
    footer: FooterSpacingRaw,
    #[serde(default)]
    popup: PopupSpacingRaw,
    #[serde(default)]
    analysis: AnalysisSpacingRaw,
    #[serde(default)]
    large_files: LargeFilesSpacingRaw,
    #[serde(default)]
    detail: DetailSpacingRaw,
}

#[derive(Default, Deserialize)]
struct TitleSpacingRaw { #[serde(default)] left: Option<u16>, #[serde(default)] right: Option<u16> }

#[derive(Default, Deserialize)]
struct SysHeaderSpacingRaw { #[serde(default)] label_value_gap: Option<u16>, #[serde(default)] item_gap: Option<u16>, #[serde(default)] line_gap: Option<u16>, #[serde(default)] separator: Option<String> }

#[derive(Default, Deserialize)]
struct VolumeTableSpacingRaw { #[serde(default)] header_prefix: Option<u16>, #[serde(default)] row_prefix: Option<u16>, #[serde(default)] row_gap: Option<u16> }

#[derive(Default, Deserialize)]
struct FooterSpacingRaw { #[serde(default)] prefix: Option<u16>, #[serde(default)] key_desc_gap: Option<u16>, #[serde(default)] group_gap: Option<u16> }

#[derive(Default, Deserialize)]
struct PopupSpacingRaw { #[serde(default)] title_padding_x: Option<u16>, #[serde(default)] content_left: Option<u16>, #[serde(default)] content_right: Option<u16>, #[serde(default)] content_top: Option<u16>, #[serde(default)] content_bottom: Option<u16>, #[serde(default)] footer_gap_top: Option<u16> }

#[derive(Default, Deserialize)]
struct AnalysisSpacingRaw { #[serde(default)] category_gap: Option<u16>, #[serde(default)] gauge_margin_top: Option<u16>, #[serde(default)] gauge_margin_bottom: Option<u16>, #[serde(default)] gauge_height: Option<u16>, #[serde(default)] gauge_gap: Option<u16> }

#[derive(Default, Deserialize)]
struct LargeFilesSpacingRaw { #[serde(default)] item_gap: Option<u16>, #[serde(default)] list_margin_top: Option<u16> }

#[derive(Default, Deserialize)]
struct DetailSpacingRaw { #[serde(default)] line_gap: Option<u16>, #[serde(default)] gauge_margin_top: Option<u16>, #[serde(default)] gauge_margin_bottom: Option<u16> }

#[derive(Default, Deserialize)]
struct ThresholdsRaw {
    #[serde(default)] documents: Option<u64>,
    #[serde(default)] pictures: Option<u64>,
    #[serde(default)] audio: Option<u64>,
    #[serde(default)] video: Option<u64>,
    #[serde(default)] other: Option<u64>,
    #[serde(default)] applications: Option<u64>,
    #[serde(default)] cache: Option<u64>,
}

#[derive(Default, Deserialize)]
struct LabelsRaw {
    #[serde(default)] cpu: Option<String>,
    #[serde(default)] memory: Option<String>,
    #[serde(default)] swap: Option<String>,
    #[serde(default)] disk_io: Option<String>,
    #[serde(default)] total: Option<String>,
    #[serde(default)] storage_analysis: Option<String>,
    #[serde(default)] category: Option<String>,
    #[serde(default)] size: Option<String>,
    #[serde(default)] pct: Option<String>,
    #[serde(default)] documents: Option<String>,
    #[serde(default)] pictures: Option<String>,
    #[serde(default)] audio: Option<String>,
    #[serde(default)] video: Option<String>,
    #[serde(default)] other: Option<String>,
    #[serde(default)] applications: Option<String>,
    #[serde(default)] system: Option<String>,
    #[serde(default)] cache: Option<String>,
    #[serde(default)] cat_total: Option<String>,
    #[serde(default)] scan_starting: Option<String>,
    #[serde(default)] scan_scanning: Option<String>,
    #[serde(default)] scan_complete: Option<String>,
    #[serde(default)] dirs_scanned: Option<String>,
    #[serde(default)] large_files: Option<String>,
    #[serde(default)] largest_files_prefix: Option<String>,
    #[serde(default)] num: Option<String>,
    #[serde(default)] file_size: Option<String>,
    #[serde(default)] path: Option<String>,
    #[serde(default)] volume_details: Option<String>,
    #[serde(default)] volume_label: Option<String>,
    #[serde(default)] file_system: Option<String>,
    #[serde(default)] capacity: Option<String>,
    #[serde(default)] available: Option<String>,
    #[serde(default)] modified: Option<String>,
    #[serde(default)] sort_by_size: Option<String>,
    #[serde(default)] sort_by_name: Option<String>,
    #[serde(default)] sort_by_time: Option<String>,
    #[serde(default)] file_detail_title: Option<String>,
    #[serde(default)] path_label: Option<String>,
    #[serde(default)] mtime_label: Option<String>,
    #[serde(default)] size_label: Option<String>,
    #[serde(default)] cat_label: Option<String>,
    #[serde(default)] help_title: Option<String>,
    #[serde(default)] help_general: Option<String>,
    #[serde(default)] help_main: Option<String>,
    #[serde(default)] help_analysis: Option<String>,
    #[serde(default)] help_cat_files: Option<String>,
    #[serde(default)] help_detail: Option<String>,
    #[serde(default)] help_exit: Option<String>,
    #[serde(default)] back: Option<String>,
    #[serde(default)] large_files_btn: Option<String>,
}

// =================================================================
//  工具函数
// =================================================================

/// 将 "#rrggbb" 格式的十六进制颜色字符串解析为 ratatui 的 Color
fn hex_color(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Reset
    }
}

/// 解析十六进制颜色，若字符串为空则返回默认值
fn hex_or(s: &str, default: &str) -> Color {
    if s.is_empty() { hex_color(default) } else { hex_color(s) }
}
