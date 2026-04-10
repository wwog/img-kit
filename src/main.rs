use img_kit::transcode::transcode_image;
use std::env;
use std::process::ExitCode;

fn print_usage(binary_name: &str) {
    eprintln!(
        "用法:\n  {binary_name} transcode <input_path> <output_dir>\n\n示例:\n  {binary_name} transcode assets/bmp_1.bmp ./output"
    );
}

fn run() -> Result<(), String> {
    let mut arguments = env::args();
    let binary_name = arguments.next().unwrap_or_else(|| "img-kit".to_owned());
    let Some(command) = arguments.next() else {
        print_usage(&binary_name);
        return Err("缺少子命令".to_owned());
    };

    match command.as_str() {
        "transcode" => {
            let Some(input_path) = arguments.next() else {
                print_usage(&binary_name);
                return Err("缺少 input_path 参数".to_owned());
            };
            let Some(output_dir) = arguments.next() else {
                print_usage(&binary_name);
                return Err("缺少 output_dir 参数".to_owned());
            };
            if arguments.next().is_some() {
                print_usage(&binary_name);
                return Err("参数过多".to_owned());
            }

            let output_path = transcode_image(&input_path, &output_dir)?;
            // stdout 只输出路径，便于调用方程序解析；人读信息写到 stderr
            eprintln!("转码成功: {output_path}");
            println!("{output_path}");
            Ok(())
        }
        "-h" | "--help" | "help" => {
            print_usage(&binary_name);
            Ok(())
        }
        _ => {
            print_usage(&binary_name);
            Err(format!("未知子命令: {command}"))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("错误: {error}");
            ExitCode::from(1)
        }
    }
}
