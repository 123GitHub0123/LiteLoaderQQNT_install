use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use log::{error, info};
use simplelog::*;
use winreg::enums::*;
use winreg::RegKey;

// 获取QQNT安装路径
fn get_qq_install_path() -> io::Result<String> {
    // 打开注册表项
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let qq_key = match hklm.open_subkey("SOFTWARE\\WOW6432Node\\Tencent\\QQNT") {
        Ok(key) => key,
        Err(e) => {
            error!("无法打开注册表项: {}", e);
            return Err(e);
        }
    };

    // 读取安装路径
    match qq_key.get_value("Install") {
        Ok(path) => {
            info!("成功读取安装路径: {}", path);
            Ok(path)
        },
        Err(e) => {
            error!("无法读取安装路径: {}", e);
            Err(e)
        }
    }
}

fn move_dll(qqnt_path: &str) -> io::Result<()> {
    // 获取当前目录
    let current_dir = match env::current_dir() {
        Ok(dir) => {
            info!("当前工作目录: {:?}", dir);
            dir
        },
        Err(e) => {
            error!("获取当前工作目录失败: {}", e);
            return Err(e);
        }
    };

    // 获取 DLL 文件路径
    let dll_path = current_dir.join("dbghelp_x64.dll");
    info!("DLL 文件路径: {:?}", dll_path);

    // 获取 DLL 目标路径
    let dll_target_path = Path::new(qqnt_path).join("dbghelp.dll");
    info!("DLL 目标路径: {:?}", dll_target_path);

    // 移动并重命名文件
    match std::fs::rename(&dll_path, &dll_target_path) {
        Ok(_) => info!("{} 文件成功移动并重命名 {}",dll_path.display(),dll_target_path.display()),
        Err(e) => {
            error!("移动并重命名文件失败: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

// 主函数
fn main() -> io::Result<()> {
    // 初始化日志记录
    CombinedLogger::init(
        vec![
            TermLogger::new(
                LevelFilter::Info,
                ConfigBuilder::new()
                    .set_time_to_local(true)
                    .set_time_format_str("%Y-%m-%d %H:%M:%S")
                    .build(),
                TerminalMode::Mixed,
                ColorChoice::Auto
            ),
            WriteLogger::new(
                LevelFilter::Info,
                ConfigBuilder::new()
                    .set_time_to_local(true)
                    .set_time_format_str("%Y-%m-%d %H:%M:%S")
                    .build(),
                File::create("LiteLoaderQQNT_Installer.log").unwrap()
            ),
        ]
    ).unwrap();

    info!("程序启动");

    // 获取 QQ 安装路径
    let install_path = match get_qq_install_path() {
        Ok(path) =>{
            info!("QQNT 安装路径： {}", path);
            path
        },
        Err(e) => {
            error!("获取 QQ 安装路径失败: {}", e);
            return Err(e);
        }
    };
    info!("QQ 安装路径: {}", install_path);

    // 获取当前工作目录
    let to_require_path = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            error!("获取当前工作目录失败: {}", e);
            return Err(e);
        }
    };
    info!("当前工作目录: {}", to_require_path.display());

    // 构建要添加的 `require` 路径
    info!("需要添加的路径: {}", to_require_path.display());

    // 指定要修改的文件路径
    let file_path = Path::new(&install_path).join("resources\\app\\app_launcher\\index.js");
    info!("文件路径: {}", file_path.display());

    // 构建需要添加的 `require` 行
    let require_line = format!("require(String.raw`{}`);", to_require_path.display());

    // 读取原始文件内容
    let mut original_content = String::new();
    {
        let mut file = match File::open(&file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("无法打开文件: {}", e);
                return Err(e);
            }
        };
        file.read_to_string(&mut original_content)?;
    }
    info!("读取原始文件内容成功");

    // 创建新的内容
    let new_content = format!("{}\n{}", require_line, original_content);

    // 将新的内容写回文件
    {
        let mut file = match File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                error!("无法创建文件: {}", e);
                return Err(e);
            }
        };
        file.write_all(new_content.as_bytes())?;
    }
    info!("文件修改成功");

    move_dll(&install_path)?;

    Ok(())
}
