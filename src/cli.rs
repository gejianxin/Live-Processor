use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::config::{DeviceLayout, DEFAULT_START_MENU_GROUP};

#[derive(Parser, Debug)]
#[command(
    name = "live-processor-cli",
    author = "gejianxin",
    version = "2.4.4-rev1",
    about = "Live-Processor CLI - 专业的 ASIO Link Pro 音频驱动与机架系统管理工具 (默认中文)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// 使用英文语言界面 / Use English interface
    #[arg(short = 'e', long)]
    pub english: bool,

    /// 交互式控制台向导模式
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 安装或升级 Live-Processor / ASIO Link Pro 音频驱动
    Install(InstallArgs),

    /// 卸载 Live-Processor 并清理系统残留
    Uninstall(UninstallArgs),

    /// 查看驱动、服务、注册表与自愈计划任务的诊断报告
    Status,

    /// 应用代码完整性策略并安装自主自愈计划任务
    Fix,

    /// 切换 WDM 音频设备声道布局 (4x8 默认模式 或 16x2 立体声模式)
    SwitchLayout(SwitchLayoutArgs),

    /// 提取内置的驱动程序和补丁包到本地目录
    ExtractPayload(ExtractPayloadArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// 自定义安装目标目录
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// WDM 设备声道布局: '4x8' (默认模式) 或 '16x2' (16 个独立立体声设备)
    #[arg(short, long, default_value = "4x8")]
    pub layout: DeviceLayout,

    /// 不创建开始菜单快捷方式
    #[arg(long)]
    pub no_shortcuts: bool,

    /// 自定义开始菜单文件夹名称
    #[arg(long, default_value = DEFAULT_START_MENU_GROUP)]
    pub start_menu_group: String,

    /// 自定义外部驱动载荷目录 (覆盖内置载荷)
    #[arg(long)]
    pub payload_dir: Option<PathBuf>,

    /// 静默无人值守安装
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// 指定卸载目录
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// 清理所有用户配置、驱动数据与自启动注册表项
    #[arg(short = 'c', long)]
    pub clean: bool,

    /// 静默无人值守卸载
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args, Debug)]
pub struct SwitchLayoutArgs {
    /// 目标 WDM 设备声道布局: '4x8' 或 '16x2'
    #[arg(short, long)]
    pub layout: DeviceLayout,

    /// 静默执行
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args, Debug)]
pub struct ExtractPayloadArgs {
    /// 提取输出目录
    #[arg(short, long, default_value = "./installer_payload")]
    pub out: PathBuf,
}
