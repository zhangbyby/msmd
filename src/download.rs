use crate::SubCommands;
use crate::Args;

pub fn download(args: &Args) {
    match &args.command {
        SubCommands::Download { thread_count } => {
            let thread_count = thread_count.unwrap_or(1);
            let thread_count = if thread_count > 16 {
                eprintln!("指定线程数大于16, 重置为16");
                16
            } else if thread_count < 1 {
                eprintln!("指定线程数小于1, 重置为1");
                1
            } else {
                thread_count
            };
            println!("开始下载音乐({}个线程)", thread_count);


            println!("下载结束");
        }
        _ => {}
    }
}