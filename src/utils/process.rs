use sysinfo::{ProcessesToUpdate, System};

#[derive(Clone, Debug)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String
}

/// 列出所有进程，默认按pid升序排列
pub fn fetch_processes() -> Vec<ProcessEntry> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut p_vec = sys.processes()
        .iter()
        .map(|(pid, process)| ProcessEntry {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().into_owned()
        })
        .collect::<Vec<ProcessEntry>>();
    p_vec.sort_by(|a, b| {
        a.pid.cmp(&b.pid)
    });
    p_vec
}