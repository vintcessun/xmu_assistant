mod course_downloader;
mod login;
mod public;
mod setting;

use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
fn main() {
    public::main();
    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择功能")
            .default(0)
            .item("下载文件")
            .item("登录账号")
            .item("设置")
            .item("重试失败任务")
            .item("退出")
            .interact()
            .unwrap_or(1000);
        match selection {
            0 => course_downloader::main(),
            1 => login::main(),
            2 => setting::main(),
            3 => public::download_file::retry_error_tasks(),
            _ => break,
        }
    }
}
