use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

use crate::public::download_file;

pub fn main() {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择设置")
        .default(0)
        .item("设置下载线程数量")
        .item("返回")
        .interact()
        .unwrap_or(1000);
    match selection {
        0 => set_num_threads(),
        1 => {}
        _ => {}
    }
}

fn set_num_threads() {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择下载线程数量")
        .default(0)
        .item("1")
        .item("2")
        .item("3")
        .item("4")
        .item("5")
        .item("6")
        .item("7")
        .item("8")
        .item("9")
        .item("10")
        .item("11")
        .item("12")
        .item("13")
        .interact()
        .unwrap_or(3);
    download_file::set_num_threads(selection + 1);
}
