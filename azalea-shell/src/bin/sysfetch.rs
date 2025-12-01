fn main() {
    let os_name = ffetch::get_os_name();
    let kernel = ffetch::get_kernel_version();
    let username = ffetch::get_username();
    let hostname = ffetch::get_hostname();

    let cpu = ffetch::get_cpu_name();
    let gpu = ffetch::get_gpu();
    let memory = ffetch::get_memory();
    let arch = ffetch::get_cpu_arch();

    let de = ffetch::get_desktop_env();

    println!("{:?}@{:?}", username, hostname);
    println!("OS: {:?}", os_name);
    println!("Kernel: {:?}", kernel);

    println!("CPU: {:?}", cpu);
    println!("GPU: {:?}", gpu);
    println!("Memory: {:?}", memory);
    println!("Architecture: {:?}", arch);

    println!("DE: {:?}", de);
}
