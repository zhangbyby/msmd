use crate::{SubCommands, Args};

pub fn init(args: &Args) {
    match &args.command {
        SubCommands::Init => {
            println!("开始初始化元数据");

            println!("初始化完成");
        }
        _ => {}
    }
}