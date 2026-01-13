//! 交互式 Prompt 模块 Demo
//!
//! 演示新的交互式 prompt 模块的各种功能

use color_eyre::eyre;
use workflow::base::interactive::*;

fn main() -> Result<()> {
    // 1. 消息输出演示
    let msg = Message::global();
    msg.break_line()?;
    msg.info("=== 消息输出演示 ===")?;
    msg.info("这是一条信息消息")?;
    msg.success("操作成功完成！")?;
    msg.warning("这是一条警告消息")?;
    msg.error("这是一条错误消息")?;
    msg.separator('-', 50)?;
    msg.break_line()?;

    // 2. 输入提示演示
    msg.break_line()?;
    msg.info("=== 输入提示演示 ===")?;
    let name = input("请输入您的姓名")
        .default("John Doe")
        .placeholder("Please enter your name")
        .prompt()?;
    msg.info(format!("您输入的姓名是: {}", name))?;

    let email = input("请输入邮箱地址")
        .default("user@example.com")
        .placeholder("Please enter your email")
        .validator(validators::email())
        .prompt()?;
    msg.info(format!("您输入的邮箱是: {}", email))?;

    // 3. 密码输入演示
    msg.break_line()?;
    msg.info("=== 密码输入演示 ===")?;
    let password = input("请输入密码")
        .password()
        .validator(|input: &str| {
            // 组合验证：先检查是否为空，再检查长度
            // 使用字符数量而不是字节长度，以正确处理 Unicode 字符
            if input.trim().is_empty() {
                Err("密码不能为空".to_string())
            } else if input.chars().count() < 8 {
                Err("密码长度至少为 8 个字符".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()?;
    msg.info(format!("密码已设置（长度: {}）", password.len()))?;

    // 4. 确认提示演示
    msg.break_line()?;
    msg.info("=== 确认提示演示 ===")?;
    let _confirmed1 = confirm("是否继续操作？").default(true).prompt()?;
    msg.info("操作已继续")?;

    let _confirmed2 = confirm("这个操作不可撤销，确定要继续吗？").default(false).prompt()?;
    msg.info("操作是否继续？")?;

    // 5. 带验证的输入演示
    msg.break_line()?;
    msg.info("=== 带验证的输入演示 ===")?;
    let age = input("请输入年龄（必须是数字）")
        .validator(|input: &str| {
            input.parse::<u32>().map(|_| ()).map_err(|_| "请输入有效的数字".to_string())
        })
        .prompt()?;
    msg.info(format!("您输入的年龄是: {}", age))?;

    // 6. 选择提示演示
    msg.break_line()?;
    msg.info("=== 选择提示演示 ===")?;
    let options = vec!["选项 1", "选项 2", "选项 3", "选项 4", "选项 5"];
    let selected = select("请选择一个选项", options.clone()).default(0).prompt()?;
    msg.info(format!("您选择的选项是: {}", selected))?;

    let selected2 = select("请选择另一个选项", vec!["红色", "绿色", "蓝色", "黄色"])
        .default(2)
        .prompt()?;
    msg.info(format!("您选择的颜色是: {}", selected2))?;

    // 7. 多选提示演示
    msg.break_line()?;
    msg.info("=== 多选提示演示 ===")?;
    let multi_options = vec!["功能 A", "功能 B", "功能 C", "功能 D", "功能 E"];
    let selected_items = multiselect("请选择多个功能", multi_options.clone())
        .default(vec![0, 2])
        .prompt()?;
    msg.info(format!("您选择的功能: {}", selected_items.join(", ")))?;

    let selected_tags =
        multiselect("请选择标签", vec!["重要", "紧急", "待办", "已完成"]).prompt()?;
    if selected_tags.is_empty() {
        msg.info("未选择任何标签")?;
    } else {
        msg.info(format!("您选择的标签: {}", selected_tags.join(", ")))?;
    }

    // 8. 表格演示
    msg.break_line()?;
    msg.info("=== 表格演示 ===")?;
    table(vec!["姓名", "年龄", "城市"])
        .add_row(vec!["张三", "25", "北京"])
        .add_row(vec!["李四", "30", "上海"])
        .add_row(vec!["王五", "28", "广州"])
        .with_border(true)
        .with_row_line(true)
        .with_alignment(Alignment::Left)
        .render();

    msg.break_line()?;
    msg.info("无边框表格:")?;
    table(vec!["项目", "状态", "进度"])
        .add_row(vec!["任务 1", "完成", "100%"])
        .add_row(vec!["任务 2", "进行中", "50%"])
        .add_row(vec!["任务 3", "待开始", "0%"])
        .with_border(false)
        .render();

    // 9. Spinner 演示
    msg.break_line()?;
    msg.info("=== Spinner 演示 ===")?;

    // 方式 1: 手动管理
    let spinner1 = spinner("正在处理数据...").start();
    std::thread::sleep(std::time::Duration::from_millis(2000));
    spinner1.stop();
    msg.success("数据处理完成！")?;

    // 方式 2: 使用 do_work
    spinner("正在加载配置...")
        .start()
        .do_work(|| -> std::result::Result<(), String> {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            Ok(())
        })
        .map_err(|e| eyre::eyre!("{}", e))?;
    msg.success("配置加载完成！")?;

    // 方式 3: 使用 with_success/with_error
    let spinner3 = spinner("正在验证...").start();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    spinner3.with_success("验证成功");

    let spinner4 = spinner("正在测试...").start();
    std::thread::sleep(std::time::Duration::from_millis(800));
    spinner4.with_info("测试完成");

    // 方式 4: 自定义动画帧和间隔
    let spinner5 = spinner("自定义动画...")
        .with_frames(vec!["-", "\\", "|", "/"])
        .with_interval(std::time::Duration::from_millis(200))
        .start();
    std::thread::sleep(std::time::Duration::from_millis(2000));
    spinner5.stop();

    // 10. Progress 演示
    msg.break_line()?;
    msg.info("=== Progress 演示 ===")?;

    // 方式 1: 已知总数的进度条（使用 progress_bar 构建器）
    msg.info("方式 1: 已知总数的进度条")?;
    let pb1 = progress_bar("正在处理文件...").with_total(100).start();
    for _i in 0..100 {
        pb1.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    pb1.finish_with_message("文件处理完成！");

    // 方式 2: 使用 Progress::new（已知总数）
    msg.break_line()?;
    msg.info("方式 2: 使用 Progress::new")?;
    let progress2 = Progress::new(50, "正在下载数据...");
    for _i in 0..50 {
        progress2.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    progress2.finish_with_message("数据下载完成！");

    // 方式 3: 下载模式（显示速度和 ETA）
    msg.break_line()?;
    msg.info("方式 3: 下载模式（显示速度和 ETA）")?;
    let total_bytes = 1024 * 1024; // 1MB
    let progress3 = Progress::new_download(total_bytes, "正在下载文件...");
    let chunk_size = 64 * 1024; // 64KB per chunk
    for i in 0..(total_bytes / chunk_size) {
        progress3.set_position(i * chunk_size);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    progress3.set_position(total_bytes);
    progress3.finish_with_message("文件下载完成！");

    // 方式 4: 未知总数（使用 spinner 模式）
    msg.break_line()?;
    msg.info("方式 4: 未知总数的进度条（spinner 模式）")?;
    let progress4 = Progress::new_unknown("正在搜索...");
    for _i in 0..20 {
        progress4.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    progress4.finish_with_message("搜索完成！");

    // 方式 5: 使用 progress_bar 构建器的自定义配置
    msg.break_line()?;
    msg.info("方式 5: 自定义配置的进度条")?;
    let pb5 = progress_bar("正在处理任务...")
        .with_total(30)
        .with_bar_width(40)
        .with_progress_chars("█░")
        .with_interval(std::time::Duration::from_millis(50))
        .start();
    for _i in 0..30 {
        pb5.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    pb5.with_success("所有任务已完成！");

    // 方式 6: 动态更新消息和总数
    msg.break_line()?;
    msg.info("方式 6: 动态更新进度条")?;
    let pb6 = progress_bar("初始化...").with_total(100).start();
    for i in 0..100 {
        if i == 25 {
            pb6.update_message("处理中...");
        } else if i == 50 {
            pb6.update_message("优化中...");
        } else if i == 75 {
            pb6.update_message("完成中...");
        }
        pb6.inc(1);
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    pb6.with_info("处理完成！");

    msg.break_line()?;
    msg.info("=== Demo 完成 ===")?;
    msg.success("所有演示已完成！")?;

    Ok(())
}
