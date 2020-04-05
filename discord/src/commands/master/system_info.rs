use crate::commands::prelude::*;
use crate::Result;
use magic::report_kb;
use sys_info::*;

#[command]
#[aliases("systeminfo", "sys_info", "sysinfo")]
#[max_args(1)]
#[owners_only]
#[example = "--all"]
#[usage = "?[-a | --all]"]
/// Get the information of the system that I'm running on.
/// Passing __-a__ or __--all__ to show more information
/// **TODO**: show more useful information in *--all* mode
async fn system_info(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let get_all = args
        .iter::<String>()
        .find(|v| v.as_ref().ok().map_or(false, |s| s == "-a" || s == "--all"));

    let mut fields = vec![
        ("OS", os()?, true),
        ("CPU", cpu()?, true),
        ("Disk", disk()?, true),
        ("Memory", memory()?, true),
    ];

    if get_all.is_some() {
        let mut addition = vec![("Hostname", hostname()?, true)];

        if cfg!(target_os = "linux") {
            addition.push(("Processes", proc_total()?.to_string(), true));
        }

        fields.extend(addition);
    }
    
    let config = crate::read_config().await;
    let color = config.color.information;
    drop(config);

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.title("System Information");
            embed.color(color);
            embed.fields(fields);
            
            embed
        })
    }).await?;

    Ok(())
}

fn os() -> Result<String> {
    let mut s = os_type()?;

    let release: String = if cfg!(target_os = "linux") {
        linux_os_release()?
            .pretty_name
            .unwrap_or_else(|| String::from("Unknown"))
    } else {
        os_release()?
    };

    s.push_str(" @ ");
    s.push_str(&release);

    Ok(s)
}

fn cpu() -> Result<String> {
    Ok(format!(
        "{} cores @ {}MHz\nLoad: {}%",
        cpu_num()?,
        cpu_speed()?,
        (loadavg()?.five * 100f64) as u8
    ))
}

fn disk() -> Result<String> {
    let DiskInfo { free, total } = disk_info()?;

    let used = total - free;
    let free_p = free / (total / 100);

    let res = format!(
        "{} / {}\nFree: {} ({}%)",
        report_kb(used),
        report_kb(total),
        report_kb(free),
        free_p
    );

    Ok(res)
}

fn memory() -> Result<String> {
    let MemInfo {
        free,
        total,
        buffers,
        cached,
        ..
    } = mem_info()?;

    let used = total - free;
    let free_p = free / (total / 100);

    let mut res = format!(
        "{} / {}\nFree: {} ({}%)",
        report_kb(used),
        report_kb(total),
        report_kb(free),
        free_p
    );

    if cfg!(target_os = "linux") {
        res.push_str(&format!("\nBuffers: {}", report_kb(buffers)));
        res.push_str(&format!("\nCached: {}", report_kb(cached)));
    }

    Ok(res)
}
